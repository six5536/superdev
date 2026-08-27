//! index.rs — the on-disk search index: tantivy for lexical retrieval, a flat
//! file of section vectors for semantic retrieval, and a manifest of per-file
//! hashes that makes a sync incremental.
//!
//! Everything lives under one directory, treated as a cache: when anything
//! about it is unusable or out of date, it is rebuilt from the bundle rather
//! than repaired.
//!
//! Retrieval runs both stores and fuses their rankings; [`Index::search`] is
//! the whole of the read side.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tantivy::collector::{DocSetCollector, TopDocs};
use tantivy::query::{BooleanQuery, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, INDEXED, IndexRecordOption, STORED, STRING, Schema, TEXT};
use tantivy::{
    DocAddress, Index as Tantivy, IndexReader, IndexWriter, ReloadPolicy, Searcher,
    TantivyDocument, Term,
};

use super::bundle::Bundle;
use super::concept::{Concept, Status};
use super::embed::Embedder;
use crate::error::{Error, Result};
use crate::lock::sha256_hex;

/// Layout version of the index. Bump it when the tantivy schema or the vector
/// file changes shape; a stored index at another version is rebuilt.
pub const SCHEMA_VERSION: u32 = 2;

/// Tantivy's indexing heap, shared across its writer threads.
const WRITER_HEAP_BYTES: usize = 50_000_000;

const TANTIVY_SUBDIR: &str = "tantivy";
const VECTORS_FILE: &str = "vectors.bin";
const MANIFEST_FILE: &str = "manifest.json";

/// Hits [`SearchOpts::limit`] defaults to.
const DEFAULT_LIMIT: usize = 8;

/// How many candidates each retrieval list offers the fusion, as a multiple
/// of the caller's limit. A section ranked low in one list can still win on
/// agreement, but only if it is in the list at all.
const CANDIDATE_FACTOR: usize = 4;

/// Reciprocal rank fusion's damping constant, at the value the method was
/// published with. It sets how much a top rank is worth: at 60 the gap
/// between rank 0 and rank 1 is small, so agreement between the two lists
/// outweighs either one's ordering.
const RRF_K: f32 = 60.0;

/// Tags that mark settled work — a finished plan or map (`done`), a resolved
/// wayfinder ticket, a rejected issue. Sections carrying one are down-ranked.
const DOWNRANK_TAGS: [&str; 3] = ["done", "resolved", "wontfix"];

/// Multiplier applied to a settled section's fused score, chosen so settled
/// work sorts below live knowledge but never disappears from results.
const DOWNRANK_FACTOR: f32 = 0.25;

/// Longest snippet, in characters.
const SNIPPET_CHARS: usize = 200;

/// Where the index lives: `.superdev/cache/aokf-index` in normal use.
///
/// The directory belongs to the index alone — a rebuild deletes it wholesale.
#[derive(Debug, Clone)]
pub struct IndexDir(pub PathBuf);

/// What one sync did, for the caller to report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncStats {
    /// Concept files parsed and written to the index this run.
    pub reindexed: usize,
    /// Concept files dropped because they left the bundle. Always 0 after a
    /// full rebuild, which starts from nothing.
    pub removed: usize,
    /// The whole index was rebuilt rather than updated in place.
    pub full_rebuild: bool,
    /// No embedder, so the index carries no vectors and search is lexical.
    pub lexical_only: bool,
}

/// One search result: where the section is, and how well it answered.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// Bundle-relative path of the concept file.
    pub path: String,
    /// The concept's frontmatter `id`, when it has one.
    pub concept_id: Option<String>,
    /// Headings from the document root down to this section; empty for the
    /// root section.
    pub heading_path: Vec<String>,
    /// First line of the section, 1-based and inclusive.
    pub start_line: usize,
    /// Last line of the section, 1-based and inclusive.
    pub end_line: usize,
    /// The section's opening text, whitespace collapsed onto one line and cut
    /// to roughly 200 characters.
    pub snippet: String,
    /// Fused score. Comparable within one result list and nowhere else.
    pub score: f32,
}

/// What to retrieve, beyond the query text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOpts {
    /// Most hits to return.
    pub limit: usize,
    /// Keep only these concept `type`s; empty means every type.
    pub kinds: Vec<String>,
    /// Keep only concepts carrying one of these tags; empty means every tag.
    pub tags: Vec<String>,
}

impl Default for SearchOpts {
    fn default() -> Self {
        SearchOpts {
            limit: DEFAULT_LIMIT,
            kinds: Vec::new(),
            tags: Vec::new(),
        }
    }
}

/// Identifies one section across both stores: a tantivy document carries the
/// path and the ordinal, a vector record the path's hash and the same
/// ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DocKey {
    path_hash: u64,
    ordinal: u32,
}

/// The stored fields of one section document, read once and reused.
#[derive(Debug, Clone)]
struct SectionDoc {
    path: String,
    concept_id: Option<String>,
    heading_path: Vec<String>,
    start_line: usize,
    end_line: usize,
    snippet: String,
    tags: Vec<String>,
    status: String,
}

impl SectionDoc {
    /// Settled work: deprecated, or tagged as done/resolved/wontfix.
    fn settled(&self) -> bool {
        self.status == "deprecated"
            || self
                .tags
                .iter()
                .any(|tag| DOWNRANK_TAGS.contains(&tag.as_str()))
    }
}

impl SectionDoc {
    fn hit(&self, score: f32) -> Hit {
        Hit {
            path: self.path.clone(),
            concept_id: self.concept_id.clone(),
            heading_path: self.heading_path.clone(),
            start_line: self.start_line,
            end_line: self.end_line,
            snippet: self.snippet.clone(),
            score,
        }
    }
}

/// An open index: a tantivy reader over the section documents, plus the
/// section vectors held in memory.
pub struct Index {
    reader: IndexReader,
    vectors: Vec<VectorRecord>,
    manifest: IndexManifest,
}

impl Index {
    /// Open or create the index at `dir`, then bring it up to date against
    /// `bundle`.
    ///
    /// Each concept file is hashed and compared with the stored manifest:
    /// new and changed files are re-indexed, files that left the bundle are
    /// deleted, and everything else is left alone. A [`SCHEMA_VERSION`] or
    /// embedder change — including gaining or losing the embedder — forces a
    /// full rebuild, as does an index directory that cannot be read.
    ///
    /// Nothing here touches the network: the caller constructs the embedder.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] when a bundle file or the index directory cannot
    /// be read or written, [`Error::Index`] when tantivy fails, and whatever
    /// the embedder returns.
    pub fn open_and_sync(
        dir: &IndexDir,
        bundle: &Bundle,
        embedder: Option<&dyn Embedder>,
    ) -> Result<(Index, SyncStats)> {
        sync(dir, bundle, embedder, false)
    }

    /// Discard the index at `dir` and build it again from `bundle`.
    ///
    /// # Errors
    ///
    /// As [`Index::open_and_sync`].
    pub fn force_rebuild(
        dir: &IndexDir,
        bundle: &Bundle,
        embedder: Option<&dyn Embedder>,
    ) -> Result<(Index, SyncStats)> {
        sync(dir, bundle, embedder, true)
    }

    /// Number of indexed sections.
    pub fn section_count(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// Number of section vectors; 0 when the index is lexical-only.
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }

    /// The embedder the vectors were built with; `None` when lexical-only.
    pub fn model_id(&self) -> Option<&str> {
        self.manifest.model_id.as_deref()
    }

    /// Hybrid search over the indexed sections.
    ///
    /// Lexical retrieval always runs: BM25 over the section text, terms ANDed,
    /// re-run as a disjunction only when the conjunction finds nothing.
    /// Semantic retrieval — cosine over the section vectors — joins it when
    /// the index has vectors and `embedder` is the one that built them; pass
    /// the embedder the sync used. Anything else, including `None`, leaves
    /// search lexical.
    ///
    /// [`SearchOpts::kinds`] and [`SearchOpts::tags`] filter both lists before
    /// they are fused by reciprocal rank fusion, so a filtered concept cannot
    /// re-enter through the other list.
    ///
    /// Sections of settled work — a `deprecated` concept, or one tagged
    /// `done`, `resolved` or `wontfix` — are down-ranked after fusion, so
    /// finished plans, issues and maps sort below live knowledge without
    /// leaving the results.
    ///
    /// The result is a flat list, best first; grouping by concept belongs to
    /// the caller.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Index`] when tantivy fails, and whatever the embedder
    /// returns for the query.
    pub fn search(
        &self,
        query: &str,
        embedder: Option<&dyn Embedder>,
        opts: &SearchOpts,
    ) -> Result<Vec<Hit>> {
        if opts.limit == 0 {
            return Ok(Vec::new());
        }
        let searcher = self.reader.searcher();
        let (_, fields) = schema();
        // Every pass that reads a document keeps it here, so the last step
        // re-reads only the sections semantic retrieval found on its own.
        let mut docs: HashMap<DocKey, SectionDoc> = HashMap::new();

        let lexical = lexical_keys(&searcher, &fields, query, opts, &mut docs)?;
        let semantic = self.semantic_keys(&searcher, &fields, query, embedder, opts, &mut docs)?;
        let mut fused = rrf(&[lexical, semantic]);
        self.read_missing(&searcher, &fields, &fused, &mut docs)?;
        downrank(&mut fused, &docs);
        fused.truncate(opts.limit);
        Ok(fused
            .into_iter()
            .filter_map(|(key, score)| docs.get(&key).map(|doc| doc.hit(score)))
            .collect())
    }

    /// The semantic candidates: sections whose vector points the same way as
    /// the query's, best first.
    ///
    /// Empty — leaving search lexical — without an embedder, without vectors,
    /// or when the embedder is not the one the vectors were built with, since
    /// two models' vectors say nothing about each other.
    fn semantic_keys(
        &self,
        searcher: &Searcher,
        fields: &Fields,
        query: &str,
        embedder: Option<&dyn Embedder>,
        opts: &SearchOpts,
        docs: &mut HashMap<DocKey, SectionDoc>,
    ) -> Result<Vec<DocKey>> {
        let Some(embedder) = embedder else {
            return Ok(Vec::new());
        };
        if self.vectors.is_empty() || self.manifest.model_id != Some(embedder.model_id()) {
            return Ok(Vec::new());
        }
        let embedded = embedder.embed(&[query.to_string()])?;
        let Some(query_vector) = embedded.first() else {
            return Ok(Vec::new());
        };
        // Only built when something is filtered, because it reads every
        // document the filter matches.
        let allowed = match filter_query(fields, opts) {
            Some(filter) => Some(collect_keys(searcher, fields, filter.as_ref(), docs)?),
            None => None,
        };

        let mut scored: Vec<(DocKey, f32)> = self
            .vectors
            .iter()
            .filter(|record| record.vector.len() == query_vector.len())
            .map(|record| {
                let key = DocKey {
                    path_hash: record.path_hash,
                    ordinal: record.ordinal,
                };
                (key, cosine(&record.vector, query_vector))
            })
            // A similarity at or below zero is not evidence of anything; such
            // a record would only add noise to the fusion.
            .filter(|(key, score)| {
                *score > 0.0 && allowed.as_ref().is_none_or(|allowed| allowed.contains(key))
            })
            .collect();
        // The key breaks ties, so the ranking never depends on the order the
        // vector file happens to be in.
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        scored.truncate(candidates(opts));
        Ok(scored.into_iter().map(|(key, _)| key).collect())
    }

    /// Read the documents behind `fused` that no earlier pass read — the
    /// sections only semantic retrieval found — the lexical pass reads its
    /// own. A vector record knows its
    /// path's hash and its ordinal, which together name exactly one document.
    fn read_missing(
        &self,
        searcher: &Searcher,
        fields: &Fields,
        fused: &[(DocKey, f32)],
        docs: &mut HashMap<DocKey, SectionDoc>,
    ) -> Result<()> {
        let missing: Vec<&DocKey> = fused
            .iter()
            .map(|(key, _)| key)
            .filter(|key| !docs.contains_key(key))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        let by_hash: HashMap<u64, &String> = self
            .manifest
            .files
            .keys()
            .map(|path| (path_hash(path), path))
            .collect();
        let clauses: Vec<Box<dyn Query>> = missing
            .into_iter()
            .filter_map(|key| {
                let path = by_hash.get(&key.path_hash)?;
                Some(Box::new(BooleanQuery::intersection(vec![
                    term_query(Term::from_field_text(fields.path, path)),
                    term_query(Term::from_field_u64(fields.ordinal, u64::from(key.ordinal))),
                ])) as Box<dyn Query>)
            })
            .collect();
        if clauses.is_empty() {
            return Ok(());
        }
        collect_keys(searcher, fields, &BooleanQuery::union(clauses), docs)?;
        Ok(())
    }
}

// Hand-written because tantivy's reader is not `Debug`; the counts are what a
// caller would want to see anyway.
impl std::fmt::Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index")
            .field("sections", &self.section_count())
            .field("vectors", &self.vectors.len())
            .field("model_id", &self.manifest.model_id)
            .finish()
    }
}

/// One section's embedding. The path hash keeps the record small and fixed
/// width; the ordinal is the section's position in its concept.
#[derive(Debug, Clone)]
struct VectorRecord {
    path_hash: u64,
    ordinal: u32,
    vector: Vec<f32>,
}

/// `manifest.json`: what the stored index was built from.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexManifest {
    schema_version: u32,
    model_id: Option<String>,
    files: BTreeMap<String, String>,
}

/// The tantivy fields, resolved once when the schema is built.
struct Fields {
    path: Field,
    concept_id: Field,
    heading_path: Field,
    start_line: Field,
    end_line: Field,
    ordinal: Field,
    kind: Field,
    tags: Field,
    status: Field,
    text: Field,
}

/// One tantivy document per section. `ordinal` is indexed so a vector record,
/// which stores only a path hash and an ordinal, can be looked up as a
/// document.
fn schema() -> (Schema, Fields) {
    let mut builder = Schema::builder();
    let fields = Fields {
        path: builder.add_text_field("path", STRING | STORED),
        concept_id: builder.add_text_field("concept_id", STRING | STORED),
        heading_path: builder.add_text_field("heading_path", STRING | STORED),
        start_line: builder.add_u64_field("start_line", STORED),
        end_line: builder.add_u64_field("end_line", STORED),
        ordinal: builder.add_u64_field("ordinal", STORED | INDEXED),
        kind: builder.add_text_field("kind", STRING | STORED),
        tags: builder.add_text_field("tags", STRING | STORED),
        status: builder.add_text_field("status", STRING | STORED),
        text: builder.add_text_field("text", TEXT | STORED),
    };
    (builder.build(), fields)
}

/// Update in place when the stored index still matches, rebuild otherwise.
fn sync(
    dir: &IndexDir,
    bundle: &Bundle,
    embedder: Option<&dyn Embedder>,
    force: bool,
) -> Result<(Index, SyncStats)> {
    let files = file_hashes(bundle);
    let model_id = embedder.map(|e| e.model_id());
    if !force
        && let Some(stored) = read_manifest(&dir.0)
        && stored.schema_version == SCHEMA_VERSION
        && stored.model_id == model_id
        && let Some(synced) = update(dir, bundle, embedder, &stored, files.clone())?
    {
        return Ok(synced);
    }
    rebuild(dir, bundle, embedder, files, model_id)
}

/// Build the index from scratch, discarding whatever was there.
fn rebuild(
    dir: &IndexDir,
    bundle: &Bundle,
    embedder: Option<&dyn Embedder>,
    files: BTreeMap<String, String>,
    model_id: Option<String>,
) -> Result<(Index, SyncStats)> {
    let root = &dir.0;
    if root.exists() {
        fs::remove_dir_all(root).map_err(|source| Error::Io {
            path: root.clone(),
            source,
        })?;
    }
    let tantivy_dir = root.join(TANTIVY_SUBDIR);
    fs::create_dir_all(&tantivy_dir).map_err(|source| Error::Io {
        path: tantivy_dir.clone(),
        source,
    })?;

    let (schema, fields) = schema();
    let tantivy = Tantivy::create_in_dir(&tantivy_dir, schema).map_err(index_error)?;
    let writer: IndexWriter = tantivy.writer(WRITER_HEAP_BYTES).map_err(index_error)?;
    let concepts: Vec<&Concept> = bundle.concepts.iter().collect();
    for concept in &concepts {
        add_concept(&writer, &fields, concept)?;
    }
    commit(writer)?;

    let vectors = embed_concepts(embedder, &concepts)?;
    write_vectors(&root.join(VECTORS_FILE), &vectors)?;
    let manifest = IndexManifest {
        schema_version: SCHEMA_VERSION,
        model_id,
        files,
    };
    write_manifest(root, &manifest)?;

    let index = Index {
        reader: reader(&tantivy)?,
        vectors,
        manifest,
    };
    let stats = SyncStats {
        reindexed: concepts.len(),
        removed: 0,
        full_rebuild: true,
        lexical_only: embedder.is_none(),
    };
    Ok((index, stats))
}

/// Bring the stored index up to date. `Ok(None)` means the stored index is
/// unusable — a missing or foreign tantivy directory, an unreadable vector
/// file — and the caller should rebuild.
fn update(
    dir: &IndexDir,
    bundle: &Bundle,
    embedder: Option<&dyn Embedder>,
    stored: &IndexManifest,
    files: BTreeMap<String, String>,
) -> Result<Option<(Index, SyncStats)>> {
    let root = &dir.0;
    let (schema, fields) = schema();
    let Ok(tantivy) = Tantivy::open_in_dir(root.join(TANTIVY_SUBDIR)) else {
        return Ok(None);
    };
    if tantivy.schema() != schema {
        return Ok(None);
    }
    let Some(mut vectors) = read_vectors(&root.join(VECTORS_FILE)) else {
        return Ok(None);
    };
    // One vector per indexed section, or none at all when the index is
    // lexical-only. Any other count means the two stores have drifted apart,
    // which only a rebuild can fix.
    let reader = reader(&tantivy)?;
    let expected = if stored.model_id.is_some() {
        reader.searcher().num_docs()
    } else {
        0
    };
    if vectors.len() as u64 != expected {
        return Ok(None);
    }

    let changed: Vec<&Concept> = bundle
        .concepts
        .iter()
        .filter(|c| stored.files.get(&c.path) != files.get(&c.path))
        .collect();
    let removed: Vec<&String> = stored
        .files
        .keys()
        .filter(|path| !files.contains_key(*path))
        .collect();

    if !changed.is_empty() || !removed.is_empty() {
        let touched: HashSet<u64> = changed
            .iter()
            .map(|c| c.path.as_str())
            .chain(removed.iter().map(|path| path.as_str()))
            .map(path_hash)
            .collect();
        // The vector file has no delete, so it is rewritten whole: keep the
        // records of untouched files, embed the changed ones afresh.
        let fresh = embed_concepts(embedder, &changed)?;
        vectors.retain(|record| !touched.contains(&record.path_hash));
        // Vectors of two widths cannot share one file, and the write would
        // fail after the tantivy commit — past the point where the caller
        // could still fall back. Rebuild instead, before writing anything.
        if let (Some(kept), Some(new)) = (vectors.first(), fresh.first())
            && kept.vector.len() != new.vector.len()
        {
            return Ok(None);
        }

        let writer: IndexWriter = tantivy.writer(WRITER_HEAP_BYTES).map_err(index_error)?;
        for path in changed
            .iter()
            .map(|c| &c.path)
            .chain(removed.iter().copied())
        {
            writer.delete_term(Term::from_field_text(fields.path, path));
        }
        for concept in &changed {
            add_concept(&writer, &fields, concept)?;
        }
        commit(writer)?;

        vectors.extend(fresh);
        vectors.sort_by_key(|record| (record.path_hash, record.ordinal));
        write_vectors(&root.join(VECTORS_FILE), &vectors)?;
        // The reader only reloads when told to, and it was built before the
        // commit above.
        reader.reload().map_err(index_error)?;
    }

    let manifest = IndexManifest {
        schema_version: SCHEMA_VERSION,
        model_id: stored.model_id.clone(),
        files,
    };
    // Nothing changed means the stored manifest already says this; a server
    // that syncs on every request should not rewrite it every request.
    if manifest.files != stored.files {
        write_manifest(root, &manifest)?;
    }

    let index = Index {
        reader,
        vectors,
        manifest,
    };
    let stats = SyncStats {
        reindexed: changed.len(),
        removed: removed.len(),
        full_rebuild: false,
        lexical_only: embedder.is_none(),
    };
    Ok(Some((index, stats)))
}

/// The indexed form of a concept's status.
fn status_str(status: &Status) -> &'static str {
    match status {
        Status::Draft => "draft",
        Status::Stable => "stable",
        Status::Deprecated => "deprecated",
    }
}

/// Add one document per section of `concept`.
fn add_concept(writer: &IndexWriter, fields: &Fields, concept: &Concept) -> Result<()> {
    for (ordinal, section) in concept.sections.iter().enumerate() {
        let mut doc = TantivyDocument::default();
        doc.add_text(fields.path, &concept.path);
        if let Some(id) = &concept.id {
            doc.add_text(fields.concept_id, id);
        }
        doc.add_text(fields.heading_path, section.heading_path.join(" > "));
        doc.add_u64(fields.start_line, section.start_line as u64);
        doc.add_u64(fields.end_line, section.end_line as u64);
        doc.add_u64(fields.ordinal, ordinal as u64);
        doc.add_text(fields.kind, &concept.kind);
        for tag in &concept.tags {
            doc.add_text(fields.tags, tag);
        }
        doc.add_text(fields.status, status_str(&concept.status));
        doc.add_text(fields.text, &section.text);
        writer.add_document(doc).map_err(index_error)?;
    }
    Ok(())
}

/// Commit, then wait for the merge threads: the index directory is deleted on
/// the next rebuild, and a live merge would still be writing into it.
fn commit(mut writer: IndexWriter) -> Result<()> {
    writer.commit().map_err(index_error)?;
    writer.wait_merging_threads().map_err(index_error)
}

/// A reader that reloads only when told to. Nothing else writes to the index
/// while it is open, so the searcher taken at creation stays current.
fn reader(tantivy: &Tantivy) -> Result<IndexReader> {
    tantivy
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()
        .map_err(index_error)
}

/// Embed every section of `concepts` in one batch.
fn embed_concepts(
    embedder: Option<&dyn Embedder>,
    concepts: &[&Concept],
) -> Result<Vec<VectorRecord>> {
    let Some(embedder) = embedder else {
        return Ok(Vec::new());
    };
    let mut keys: Vec<(u64, u32)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for concept in concepts {
        let hash = path_hash(&concept.path);
        for (ordinal, section) in concept.sections.iter().enumerate() {
            keys.push((hash, ordinal as u32));
            texts.push(section.text.clone());
        }
    }
    let vectors = embedder.embed(&texts)?;
    if vectors.len() != keys.len() {
        return Err(Error::Index {
            message: format!(
                "embedder returned {} vectors for {} sections",
                vectors.len(),
                keys.len()
            ),
        });
    }
    Ok(keys
        .into_iter()
        .zip(vectors)
        .map(|((path_hash, ordinal), vector)| VectorRecord {
            path_hash,
            ordinal,
            vector,
        })
        .collect())
}

/// The first 64 bits of the path's sha256. Collisions are not a correctness
/// risk here: the path itself is stored in tantivy, and the hash only joins a
/// vector back to its section.
fn path_hash(path: &str) -> u64 {
    let hex = sha256_hex(path.as_bytes());
    u64::from_str_radix(&hex[..16], 16).unwrap_or_default()
}

/// The hash of every concept as parsed, keyed by bundle-relative path.
///
/// These come from the load, not from a fresh read: re-reading here would let
/// a write between load and sync record the new hash against the old content,
/// which no later sync could ever notice.
fn file_hashes(bundle: &Bundle) -> BTreeMap<String, String> {
    bundle
        .concepts
        .iter()
        .map(|c| (c.path.clone(), c.content_hash.clone()))
        .collect()
}

/// Read `manifest.json`. A missing or unreadable manifest reads as absent,
/// which sends the caller down the rebuild path.
fn read_manifest(root: &Path) -> Option<IndexManifest> {
    let text = fs::read_to_string(root.join(MANIFEST_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_manifest(root: &Path, manifest: &IndexManifest) -> Result<()> {
    let path = root.join(MANIFEST_FILE);
    let json = serde_json::to_string_pretty(manifest).map_err(index_error)?;
    fs::write(&path, format!("{json}\n")).map_err(|source| Error::Io { path, source })
}

/// `vectors.bin`: `dim: u32`, `count: u32`, then `count` records of
/// `path_hash: u64`, `ordinal: u32`, `dim` little-endian `f32`s.
fn write_vectors(path: &Path, records: &[VectorRecord]) -> Result<()> {
    let dim = records.first().map_or(0, |r| r.vector.len());
    if records.iter().any(|r| r.vector.len() != dim) {
        return Err(Error::Index {
            message: "embedder returned vectors of differing widths".into(),
        });
    }
    let mut bytes = Vec::with_capacity(8 + records.len() * (12 + dim * 4));
    bytes.extend_from_slice(&(dim as u32).to_le_bytes());
    bytes.extend_from_slice(&(records.len() as u32).to_le_bytes());
    for record in records {
        bytes.extend_from_slice(&record.path_hash.to_le_bytes());
        bytes.extend_from_slice(&record.ordinal.to_le_bytes());
        for value in &record.vector {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    fs::write(path, bytes).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Read `vectors.bin`. Anything that does not add up reads as `None`: the
/// file is a cache, so a truncated or foreign one is rebuilt, not diagnosed.
fn read_vectors(path: &Path) -> Option<Vec<VectorRecord>> {
    let bytes = fs::read(path).ok()?;
    let dim = u32::from_le_bytes(bytes.get(0..4)?.try_into().ok()?) as usize;
    let count = u32::from_le_bytes(bytes.get(4..8)?.try_into().ok()?) as usize;
    let stride = dim.checked_mul(4)?.checked_add(12)?;
    if bytes.len() != stride.checked_mul(count)?.checked_add(8)? {
        return None;
    }
    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let at = 8 + i * stride;
        let path_hash = u64::from_le_bytes(bytes[at..at + 8].try_into().ok()?);
        let ordinal = u32::from_le_bytes(bytes[at + 8..at + 12].try_into().ok()?);
        let vector = (0..dim)
            .map(|d| {
                let offset = at + 12 + d * 4;
                // The length check above guarantees these four bytes exist.
                f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
            })
            .collect();
        records.push(VectorRecord {
            path_hash,
            ordinal,
            vector,
        });
    }
    Some(records)
}

/// The lexical candidates: BM25 over the section text, best first.
///
/// The query is parsed twice at most — all terms required, then any term —
/// because a user's phrasing usually names more than one section's worth of
/// words, and an empty answer is worse than a loose one.
fn lexical_keys(
    searcher: &Searcher,
    fields: &Fields,
    query: &str,
    opts: &SearchOpts,
    docs: &mut HashMap<DocKey, SectionDoc>,
) -> Result<Vec<DocKey>> {
    let filter = filter_query(fields, opts);
    let mut conjunction = QueryParser::for_index(searcher.index(), vec![fields.text]);
    conjunction.set_conjunction_by_default();
    let disjunction = QueryParser::for_index(searcher.index(), vec![fields.text]);

    for parser in [&conjunction, &disjunction] {
        // Lenient, because the query is prose a user typed, not a query
        // language: an unbalanced quote should still search for the words.
        let (parsed, _) = parser.parse_query_lenient(query);
        let query: Box<dyn Query> = match &filter {
            Some(filter) => Box::new(BooleanQuery::intersection(vec![parsed, filter.box_clone()])),
            None => parsed,
        };
        let top = searcher
            .search(
                &query,
                &TopDocs::with_limit(candidates(opts)).order_by_score(),
            )
            .map_err(index_error)?;
        if top.is_empty() {
            continue;
        }
        let mut keys = Vec::with_capacity(top.len());
        for (_, address) in top {
            let (key, doc) = read_doc(searcher, fields, address)?;
            docs.insert(key, doc);
            keys.push(key);
        }
        return Ok(keys);
    }
    Ok(Vec::new())
}

/// Reciprocal rank fusion: each list gives a section `1 / (k + rank)`, and the
/// scores add up.
///
/// Rank is what fuses, never score: BM25 and cosine are on unrelated scales,
/// so the only comparable thing the two lists produce is their ordering.
/// Scale down the fused score of settled sections and re-sort. Stable, so
/// sections on equal scores keep their fused order.
fn downrank(fused: &mut [(DocKey, f32)], docs: &HashMap<DocKey, SectionDoc>) {
    let mut touched = false;
    for (key, score) in fused.iter_mut() {
        if docs.get(key).is_some_and(SectionDoc::settled) {
            *score *= DOWNRANK_FACTOR;
            touched = true;
        }
    }
    if touched {
        fused.sort_by(|a, b| b.1.total_cmp(&a.1));
    }
}

pub(crate) fn rrf(lists: &[Vec<DocKey>]) -> Vec<(DocKey, f32)> {
    let mut scores: HashMap<DocKey, f32> = HashMap::new();
    let mut order: Vec<DocKey> = Vec::new();
    for list in lists {
        for (rank, key) in list.iter().enumerate() {
            let score = 1.0 / (RRF_K + rank as f32);
            scores
                .entry(*key)
                .and_modify(|total| *total += score)
                .or_insert_with(|| {
                    order.push(*key);
                    score
                });
        }
    }
    let mut scored: Vec<(DocKey, f32)> = order.iter().map(|key| (*key, scores[key])).collect();
    // A stable sort, so sections on equal scores keep the order the lists
    // first offered them in.
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored
}

/// The `kinds`/`tags` filter as a query: any of the kinds, and any of the
/// tags. `None` when nothing was filtered.
fn filter_query(fields: &Fields, opts: &SearchOpts) -> Option<Box<dyn Query>> {
    let any = |field: Field, values: &[String]| -> Option<Box<dyn Query>> {
        let clauses: Vec<Box<dyn Query>> = values
            .iter()
            .map(|value| term_query(Term::from_field_text(field, value)))
            .collect();
        (!clauses.is_empty()).then(|| Box::new(BooleanQuery::union(clauses)) as Box<dyn Query>)
    };
    let groups: Vec<Box<dyn Query>> = [any(fields.kind, &opts.kinds), any(fields.tags, &opts.tags)]
        .into_iter()
        .flatten()
        .collect();
    (!groups.is_empty()).then(|| Box::new(BooleanQuery::intersection(groups)) as Box<dyn Query>)
}

/// Every document `query` matches, keyed and cached. Unscored: the caller
/// wants the set, not a ranking.
fn collect_keys(
    searcher: &Searcher,
    fields: &Fields,
    query: &dyn Query,
    docs: &mut HashMap<DocKey, SectionDoc>,
) -> Result<HashSet<DocKey>> {
    let mut keys = HashSet::new();
    for address in searcher
        .search(query, &DocSetCollector)
        .map_err(index_error)?
    {
        let (key, doc) = read_doc(searcher, fields, address)?;
        docs.insert(key, doc);
        keys.insert(key);
    }
    Ok(keys)
}

/// Read one section document's stored fields.
fn read_doc(
    searcher: &Searcher,
    fields: &Fields,
    address: DocAddress,
) -> Result<(DocKey, SectionDoc)> {
    // Scoped: the trait's `as_str` would otherwise shadow `String::as_str`
    // for the rest of the module.
    use tantivy::schema::Value as _;

    let doc: TantivyDocument = searcher.doc(address).map_err(index_error)?;
    let text = |field| doc.get_first(field).and_then(|value| value.as_str());
    let number = |field| doc.get_first(field).and_then(|value| value.as_u64());
    let path = text(fields.path).unwrap_or_default().to_string();
    let key = DocKey {
        path_hash: path_hash(&path),
        ordinal: number(fields.ordinal).unwrap_or_default() as u32,
    };
    let section = SectionDoc {
        concept_id: text(fields.concept_id).map(str::to_string),
        heading_path: text(fields.heading_path)
            .filter(|joined| !joined.is_empty())
            .map(|joined| joined.split(" > ").map(str::to_string).collect())
            .unwrap_or_default(),
        start_line: number(fields.start_line).unwrap_or_default() as usize,
        end_line: number(fields.end_line).unwrap_or_default() as usize,
        snippet: snippet(text(fields.text).unwrap_or_default()),
        tags: doc
            .get_all(fields.tags)
            .filter_map(|value| value.as_str())
            .map(str::to_string)
            .collect(),
        status: text(fields.status).unwrap_or("stable").to_string(),
        path,
    };
    Ok((key, section))
}

/// A section's opening text on one line, cut to [`SNIPPET_CHARS`].
fn snippet(text: &str) -> String {
    let single = text.split_whitespace().collect::<Vec<&str>>().join(" ");
    match single.char_indices().nth(SNIPPET_CHARS) {
        Some((at, _)) => format!("{}…", single[..at].trim_end()),
        None => single,
    }
}

/// Cosine similarity. The embedders normalise, which makes this a dot
/// product, but nothing enforces that, so the norms are computed.
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm = |v: &[f32]| v.iter().map(|x| x * x).sum::<f32>().sqrt();
    let scale = norm(a) * norm(b);
    if scale == 0.0 { 0.0 } else { dot / scale }
}

/// How many candidates one retrieval list may offer.
fn candidates(opts: &SearchOpts) -> usize {
    opts.limit.saturating_mul(CANDIDATE_FACTOR)
}

fn term_query(term: Term) -> Box<dyn Query> {
    Box::new(TermQuery::new(term, IndexRecordOption::Basic))
}

fn index_error(e: impl std::fmt::Display) -> Error {
    Error::Index {
        message: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;
    use crate::sokf::bundle::{Bundle, load_bundle};
    use crate::sokf::embed::FakeEmbedder;

    /// Two headings, so this concept contributes three sections after the edit
    /// in `edited_file_reindexes_only_that_file` and two before it.
    const ALPHA: &str = "---\ntype: Module\nid: alpha\ntags: [core]\n---\nPlanning, before any heading.\n\n# Details\n\nThe planning stage never writes.\n";
    const BETA: &str = "---\ntype: Spec\nid: beta\n---\nBeta has no headings at all.\n";

    /// A tempdir holding `bundle/` (two concepts, three sections) and room for
    /// `idx/` beside it.
    fn fixture() -> (TempDir, Bundle) {
        let dir = tempfile::tempdir().unwrap();
        let bundle_dir = dir.path().join("bundle");
        fs::create_dir(&bundle_dir).unwrap();
        fs::write(bundle_dir.join("alpha.md"), ALPHA).unwrap();
        fs::write(bundle_dir.join("beta.md"), BETA).unwrap();
        let bundle = load_bundle(&bundle_dir).unwrap();
        (dir, bundle)
    }

    /// Re-read the bundle after a test edits it on disk.
    fn reload(dir: &Path) -> Bundle {
        load_bundle(&dir.join("bundle")).unwrap()
    }

    /// Two concepts with no shared vocabulary: `release` owns "tag-driven
    /// pipeline", `testing` owns "nextest".
    const RELEASE: &str = "---\ntype: Reference\nid: release\ntags: [process]\n---\nReleases are cut from main.\n\n# Pipeline\n\nThe release pipeline is tag-driven: pushing a tag triggers the publish.\n";
    const TESTING: &str = "---\ntype: Spec\nid: testing\ntags: [quality]\n---\nTests run under nextest.\n\n# Layers\n\nUnit tests and end-to-end tests, run on every commit.\n";

    /// An index over [`RELEASE`] and [`TESTING`]. The tempdir comes back with
    /// it: dropping it would delete the index under the reader.
    fn search_fixture(embedder: Option<&dyn Embedder>) -> (TempDir, Index) {
        let dir = tempfile::tempdir().unwrap();
        let bundle_dir = dir.path().join("bundle");
        fs::create_dir(&bundle_dir).unwrap();
        fs::write(bundle_dir.join("release.md"), RELEASE).unwrap();
        fs::write(bundle_dir.join("testing.md"), TESTING).unwrap();
        let bundle = load_bundle(&bundle_dir).unwrap();
        let (index, _) =
            Index::open_and_sync(&IndexDir(dir.path().join("idx")), &bundle, embedder).unwrap();
        (dir, index)
    }

    #[test]
    fn lexical_search_finds_the_right_section() {
        let (_dir, idx) = search_fixture(None);
        let hits = idx
            .search("tag-driven release pipeline", None, &SearchOpts::default())
            .unwrap();
        // Only the Pipeline section carries every term, so the AND pass
        // answers the query on its own.
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].concept_id.as_deref(), Some("release"));
        assert_eq!(hits[0].path, "release.md");
        assert_eq!(hits[0].heading_path, vec!["Pipeline".to_string()]);
        assert!(hits[0].start_line > 0 && hits[0].end_line >= hits[0].start_line);
        // The heading line through to the end of the file.
        assert_eq!((hits[0].start_line, hits[0].end_line), (8, 10));
        assert!(hits[0].snippet.contains("tag-driven"));
        assert!(!hits[0].snippet.contains('\n'));

        // A caller that asks for nothing gets nothing.
        let none = SearchOpts {
            limit: 0,
            ..SearchOpts::default()
        };
        assert!(idx.search("release", None, &none).unwrap().is_empty());
    }

    /// Body-only concepts, so a section's text is exactly its one body line
    /// and a query can be made byte-identical to it.
    const EXACT: &str = "---\ntype: Note\nid: exact\n---\nzephyr quartz lantern meadow\n";
    const DENSE: &str = "---\ntype: Note\nid: dense\n---\nzephyr zephyr zephyr quartz quartz quartz lantern lantern lantern meadow meadow meadow\n";

    #[test]
    fn semantic_contributes_when_vectors_exist() {
        let dir = tempfile::tempdir().unwrap();
        let bundle_dir = dir.path().join("bundle");
        fs::create_dir(&bundle_dir).unwrap();
        fs::write(bundle_dir.join("exact.md"), EXACT).unwrap();
        fs::write(bundle_dir.join("dense.md"), DENSE).unwrap();
        let bundle = load_bundle(&bundle_dir).unwrap();
        let (idx, _) = Index::open_and_sync(
            &IndexDir(dir.path().join("idx")),
            &bundle,
            Some(&FakeEmbedder),
        )
        .unwrap();

        // The embedder hashes whole texts, so cosine here is an exact-text
        // test: the query is `exact`'s only section verbatim, and `dense` —
        // the same words three times over — points somewhere else entirely.
        let query = "zephyr quartz lantern meadow";
        let dense_text = &bundle.concepts[0].sections[0].text;
        assert_eq!(bundle.concepts[0].id.as_deref(), Some("dense"));
        let vectors = FakeEmbedder
            .embed(&[query.to_string(), dense_text.clone()])
            .unwrap();
        let cosine: f32 = vectors[0].iter().zip(&vectors[1]).map(|(a, b)| a * b).sum();
        assert!(
            cosine <= 0.0,
            "fixture assumes `dense` is far off: {cosine}"
        );

        // `dense` repeats every term, so it leads BM25; `exact` trails it
        // there but tops the semantic list. Two ranks beat one.
        let hits = idx
            .search(query, Some(&FakeEmbedder), &SearchOpts::default())
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].concept_id.as_deref(), Some("exact"));
        assert_eq!(hits[1].concept_id.as_deref(), Some("dense"));
        assert!(hits[0].score > hits[1].score);
        assert!((hits[0].score - (1.0 / 60.0 + 1.0 / 61.0)).abs() < 1e-6);
        assert!((hits[1].score - 1.0 / 60.0).abs() < 1e-6);

        // Lexical alone reverses them, which is what the semantic list had to
        // overturn.
        let lexical = idx.search(query, None, &SearchOpts::default()).unwrap();
        assert_eq!(lexical[0].concept_id.as_deref(), Some("dense"));
        // An embedder the index was not built with is ignored: its vectors
        // cannot be compared with the stored ones.
        let mismatched = idx
            .search(query, Some(&RaggedEmbedder), &SearchOpts::default())
            .unwrap();
        assert_eq!(mismatched, lexical);
    }

    /// Three concepts sharing one vocabulary; only their settledness differs.
    const LIVE: &str = "---\ntype: Note\nid: live\n---\nquartz lantern meadow guide\n";
    const FINISHED: &str =
        "---\ntype: Plan\nid: finished\ntags: [done]\n---\nquartz lantern meadow guide\n";
    const RETIRED: &str =
        "---\ntype: Spec\nid: retired\nstatus: deprecated\n---\nquartz lantern meadow guide\n";

    #[test]
    fn settled_work_is_downranked_not_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let bundle_dir = dir.path().join("bundle");
        fs::create_dir(&bundle_dir).unwrap();
        fs::write(bundle_dir.join("finished.md"), FINISHED).unwrap();
        fs::write(bundle_dir.join("retired.md"), RETIRED).unwrap();
        fs::write(bundle_dir.join("live.md"), LIVE).unwrap();
        let bundle = load_bundle(&bundle_dir).unwrap();
        let (idx, _) =
            Index::open_and_sync(&IndexDir(dir.path().join("idx")), &bundle, None).unwrap();

        let hits = idx.search("quartz", None, &SearchOpts::default()).unwrap();
        // Identical text, so ranking is decided by settledness alone: the
        // live concept first, the done-tagged plan and the deprecated spec
        // down-ranked behind it — but still present.
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].concept_id.as_deref(), Some("live"));
        assert!(hits[1].score < hits[0].score);
        let trailing: HashSet<_> = hits[1..]
            .iter()
            .map(|hit| hit.concept_id.clone().unwrap())
            .collect();
        assert_eq!(
            trailing,
            HashSet::from(["finished".to_string(), "retired".to_string()])
        );
    }

    #[test]
    fn filters_restrict_kinds_and_tags() {
        let (_dir, idx) = search_fixture(Some(&FakeEmbedder));
        let opts = |kinds: &[&str], tags: &[&str]| SearchOpts {
            kinds: kinds.iter().map(|s| (*s).to_string()).collect(),
            tags: tags.iter().map(|s| (*s).to_string()).collect(),
            ..SearchOpts::default()
        };
        let search = |opts: &SearchOpts| {
            idx.search("release pipeline tests layers", Some(&FakeEmbedder), opts)
                .unwrap()
        };

        // Unfiltered, both concepts answer.
        let all = search(&SearchOpts::default());
        assert!(all.iter().any(|h| h.path == "release.md"));
        assert!(all.iter().any(|h| h.path == "testing.md"));

        // Filters apply to the semantic list as well as the lexical one, so a
        // vectored index cannot smuggle an excluded concept back in.
        let spec = search(&opts(&["Spec"], &[]));
        assert!(!spec.is_empty());
        assert!(spec.iter().all(|h| h.path == "testing.md"));
        let process = search(&opts(&[], &["process"]));
        assert!(!process.is_empty());
        assert!(process.iter().all(|h| h.path == "release.md"));

        // Kinds and tags are ANDed: no Spec is tagged `process`.
        assert!(search(&opts(&["Spec"], &["process"])).is_empty());
        // An unknown value matches nothing rather than everything.
        assert!(search(&opts(&["Nope"], &[])).is_empty());
    }

    #[test]
    fn fusion_maths() {
        let key = |path_hash| DocKey {
            path_hash,
            ordinal: 0,
        };
        let (both, single, tail) = (key(1), key(2), key(3));
        let scored = rrf(&[vec![both, tail], vec![single, both]]);
        assert_eq!(scored.len(), 3);
        // Ranks 0 and 1 across the two lists.
        assert_eq!(scored[0].0, both);
        assert!((scored[0].1 - (1.0 / 60.0 + 1.0 / 61.0)).abs() < 1e-6);
        // Rank 0 of one list only.
        assert_eq!(scored[1].0, single);
        assert!((scored[1].1 - 1.0 / 60.0).abs() < 1e-6);
        assert_eq!(scored[2].0, tail);
        assert!((scored[2].1 - 1.0 / 61.0).abs() < 1e-6);
        assert!(rrf(&[]).is_empty());
    }

    #[test]
    fn a_query_no_section_answers_in_full_falls_back_to_or() {
        let (_dir, idx) = search_fixture(None);
        // No section carries both terms, so the AND pass finds nothing and the
        // OR pass answers.
        let hits = idx
            .search("pipeline nextest", None, &SearchOpts::default())
            .unwrap();
        assert_eq!(hits.len(), 2);
        let mut ids: Vec<&str> = hits
            .iter()
            .filter_map(|h| h.concept_id.as_deref())
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, vec!["release", "testing"]);
        // A word in neither concept finds nothing rather than everything.
        assert!(
            idx.search("kryptonite", None, &SearchOpts::default())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_semantic_hit_joins_back_to_its_section() {
        let (_dir, idx) = search_fixture(Some(&FakeEmbedder));
        // No section carries this word, so lexical retrieval returns nothing
        // and every hit comes from the vector store — each one looked up by
        // its path hash and ordinal.
        assert!(
            idx.search("kryptonite", None, &SearchOpts::default())
                .unwrap()
                .is_empty()
        );
        let hits = idx
            .search("kryptonite", Some(&FakeEmbedder), &SearchOpts::default())
            .unwrap();

        // Three of the four sections; the fourth's vector points away from the
        // query, and a cosine at or below zero is not a hit.
        assert_eq!(hits.len(), 3);
        assert!(
            hits.iter()
                .all(|h| h.heading_path != vec!["Pipeline".to_string()])
        );
        // Every field of a semantic-only hit comes from the joined document.
        assert_eq!(hits[0].path, "testing.md");
        assert_eq!(hits[0].concept_id.as_deref(), Some("testing"));
        assert_eq!(hits[0].heading_path, vec!["Layers".to_string()]);
        assert_eq!((hits[0].start_line, hits[0].end_line), (8, 10));
        assert!(hits[0].snippet.contains("Unit tests"));
        // A root section has no heading path.
        assert_eq!(hits[1].heading_path, Vec::<String>::new());
        assert_eq!(hits[1].start_line, 1);
        // One list, so the scores are the bare reciprocal ranks.
        assert!((hits[0].score - 1.0 / 60.0).abs() < 1e-6);
        assert!((hits[2].score - 1.0 / 62.0).abs() < 1e-6);
    }

    #[test]
    fn snippets_are_one_line_and_bounded() {
        let long = format!("# Heading\n\n{}", "word ".repeat(80));
        let short = snippet(&long);
        assert!(short.chars().count() <= 201);
        assert!(short.ends_with('…'));
        assert!(!short.contains('\n'));
        // Whitespace of every kind collapses to single spaces.
        assert_eq!(snippet("  a\n\n\tb  "), "a b");
    }

    #[test]
    fn first_open_indexes_everything() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        assert!(stats.full_rebuild);
        assert_eq!(stats.reindexed, 2);
        assert_eq!(stats.removed, 0);
        assert!(!stats.lexical_only);
        assert_eq!(index.section_count(), 3);
        assert_eq!(index.vector_count(), 3);
        assert_eq!(index.model_id(), Some("fake:8"));
        assert_eq!(
            format!("{index:?}"),
            "Index { sections: 3, vectors: 3, model_id: Some(\"fake:8\") }"
        );
    }

    #[test]
    fn stored_vectors_round_trip_intact() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();

        // Read the file back through the codec and check one known record end
        // to end: a transposed field or a flipped byte order would show here
        // and nowhere else.
        let records = read_vectors(&idx.0.join("vectors.bin")).unwrap();
        assert_eq!(records.len(), 3);
        let beta = &bundle.concepts[1];
        assert_eq!(beta.path, "beta.md");
        let record = records
            .iter()
            .find(|r| r.path_hash == path_hash("beta.md") && r.ordinal == 0)
            .expect("beta's only section");
        let expected = FakeEmbedder
            .embed(&[beta.sections[0].text.clone()])
            .unwrap();
        assert_eq!(record.vector, expected[0]);
        // Two files, three sections: alpha owns the other two ordinals.
        let mut alpha: Vec<u32> = records
            .iter()
            .filter(|r| r.path_hash == path_hash("alpha.md"))
            .map(|r| r.ordinal)
            .collect();
        alpha.sort_unstable();
        assert_eq!(alpha, vec![0, 1]);
    }

    #[test]
    fn unchanged_bundle_syncs_nothing() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        drop(first);

        let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        assert!(!stats.full_rebuild);
        assert_eq!(stats.reindexed, 0);
        assert_eq!(stats.removed, 0);
        assert_eq!(index.section_count(), 3);
        assert_eq!(index.vector_count(), 3);
    }

    #[test]
    fn edited_file_reindexes_only_that_file() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        drop(first);

        // A second heading, so the file goes from two sections to three: the
        // old documents must be deleted, not merely joined by the new ones.
        fs::write(
            dir.path().join("bundle/alpha.md"),
            format!("{ALPHA}\n# Extra\n\nA newly written section.\n"),
        )
        .unwrap();
        let edited = reload(dir.path());
        let (index, stats) = Index::open_and_sync(&idx, &edited, Some(&FakeEmbedder)).unwrap();
        assert!(!stats.full_rebuild);
        assert_eq!(stats.reindexed, 1);
        assert_eq!(stats.removed, 0);
        assert_eq!(index.section_count(), 4);
        assert_eq!(index.vector_count(), 4);
    }

    #[test]
    fn deleted_file_is_removed() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        drop(first);

        fs::remove_file(dir.path().join("bundle/beta.md")).unwrap();
        let smaller = reload(dir.path());
        let (index, stats) = Index::open_and_sync(&idx, &smaller, Some(&FakeEmbedder)).unwrap();
        assert!(!stats.full_rebuild);
        assert_eq!(stats.reindexed, 0);
        assert_eq!(stats.removed, 1);
        // Only alpha's two sections survive, in both stores.
        assert_eq!(index.section_count(), 2);
        assert_eq!(index.vector_count(), 2);
        // And beta's text no longer answers a query made of it.
        assert!(
            index
                .search("Beta has no headings at all", None, &SearchOpts::default())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn force_rebuild_reindexes_everything() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        drop(first);

        let (index, stats) = Index::force_rebuild(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        assert!(stats.full_rebuild);
        assert_eq!(stats.reindexed, 2);
        assert_eq!(index.section_count(), 3);
    }

    #[test]
    fn an_unusable_cache_is_rebuilt() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));

        // Each of these leaves the manifest intact, so only the damaged part
        // can send the sync down the rebuild path. The last is a well-formed
        // but empty vector file: no longer one vector per section.
        let damage: [&dyn Fn(&Path); 3] = [
            &|root| fs::remove_dir_all(root.join("tantivy")).unwrap(),
            &|root| fs::write(root.join("vectors.bin"), b"short").unwrap(),
            &|root| fs::write(root.join("vectors.bin"), [0u8; 8]).unwrap(),
        ];
        for damage in damage {
            let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
            drop(first);
            damage(&idx.0);
            let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
            assert!(stats.full_rebuild);
            assert_eq!(index.section_count(), 3);
            assert_eq!(index.vector_count(), 3);
        }

        // A manifest that is not an index manifest reads as no index at all.
        fs::write(idx.0.join("manifest.json"), "{ not json").unwrap();
        let (_, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        assert!(stats.full_rebuild);
    }

    /// Names itself as [`FakeEmbedder`] does, so the manifest gate lets an
    /// incremental sync through, but embeds twice as wide.
    struct WiderEmbedder;

    impl Embedder for WiderEmbedder {
        fn model_id(&self) -> String {
            "fake:8".into()
        }

        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| vec![0.25; 16]).collect())
        }
    }

    #[test]
    fn a_change_of_vector_width_rebuilds_rather_than_failing() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        drop(first);

        // One changed file, so the sync would mix kept 8-wide records with
        // fresh 16-wide ones.
        fs::write(
            dir.path().join("bundle/alpha.md"),
            format!("{ALPHA}\nA newly written line.\n"),
        )
        .unwrap();
        let edited = reload(dir.path());
        let (index, stats) = Index::open_and_sync(&idx, &edited, Some(&WiderEmbedder)).unwrap();
        assert!(stats.full_rebuild);
        assert_eq!(stats.reindexed, 2);
        assert_eq!(index.vector_count(), 3);
        // Every record is the new width, so nothing of the old index survived.
        let records = read_vectors(&idx.0.join("vectors.bin")).unwrap();
        assert!(records.iter().all(|r| r.vector.len() == 16));
    }

    #[test]
    fn gaining_an_embedder_forces_a_full_rebuild() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (first, stats) = Index::open_and_sync(&idx, &bundle, None).unwrap();
        assert!(stats.lexical_only);
        assert_eq!(first.vector_count(), 0);
        drop(first);

        let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        assert!(stats.full_rebuild);
        assert!(!stats.lexical_only);
        assert_eq!(stats.reindexed, 2);
        assert_eq!(index.vector_count(), 3);
        assert_eq!(index.model_id(), Some("fake:8"));
    }

    #[test]
    fn an_index_from_another_schema_version_is_rebuilt() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        drop(first);

        let manifest = idx.0.join("manifest.json");
        let text = fs::read_to_string(&manifest).unwrap();
        fs::write(
            &manifest,
            text.replace(
                &format!("\"schema_version\": {SCHEMA_VERSION}"),
                "\"schema_version\": 99",
            ),
        )
        .unwrap();
        assert!(fs::read_to_string(&manifest).unwrap().contains("99"));

        let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        assert!(stats.full_rebuild);
        assert_eq!(stats.reindexed, 2);
        assert_eq!(index.section_count(), 3);
        assert_eq!(
            read_manifest(&idx.0).unwrap().schema_version,
            SCHEMA_VERSION
        );
    }

    /// Returns one vector per text, but of two different widths.
    struct RaggedEmbedder;

    impl Embedder for RaggedEmbedder {
        fn model_id(&self) -> String {
            "ragged".into()
        }

        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            Ok(texts
                .iter()
                .enumerate()
                .map(|(i, _)| vec![1.0; i + 1])
                .collect())
        }
    }

    /// Returns fewer vectors than it was given texts.
    struct ShortEmbedder;

    impl Embedder for ShortEmbedder {
        fn model_id(&self) -> String {
            "short".into()
        }

        fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
            Ok(vec![vec![1.0]])
        }
    }

    #[test]
    fn a_misbehaving_embedder_is_an_index_error() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let ragged = Index::open_and_sync(&idx, &bundle, Some(&RaggedEmbedder)).unwrap_err();
        assert!(matches!(ragged, Error::Index { .. }));
        assert!(ragged.to_string().contains("differing widths"));

        let short = Index::open_and_sync(&idx, &bundle, Some(&ShortEmbedder)).unwrap_err();
        assert_eq!(
            short.to_string(),
            "index: embedder returned 1 vectors for 3 sections"
        );
    }

    #[test]
    fn model_change_forces_full_rebuild() {
        let (dir, bundle) = fixture();
        let idx = IndexDir(dir.path().join("idx"));
        let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        drop(first);

        let (index, stats) = Index::open_and_sync(&idx, &bundle, None).unwrap();
        assert!(stats.full_rebuild);
        assert!(stats.lexical_only);
        assert_eq!(stats.reindexed, 2);
        assert_eq!(index.section_count(), 3);
        // No embedder, so no vectors — and the manifest says so.
        assert_eq!(index.vector_count(), 0);
        assert_eq!(index.model_id(), None);
    }
}
