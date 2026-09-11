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

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tantivy::schema::{Field, INDEXED, STORED, STRING, Schema, TEXT};
use tantivy::{Index as Tantivy, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};

use super::bundle::Bundle;
use super::concept::{Concept, Status};
use super::embed::Embedder;
use crate::error::{Error, Result};
use crate::lock::sha256_hex;

mod direct;
mod search;
pub use direct::MatchKind;
#[cfg(test)]
mod direct_tests;
#[cfg(test)]
mod tests;

/// Layout version of the index. Bump it when the tantivy schema or the vector
/// file changes shape; a stored index at another version is rebuilt.
pub const SCHEMA_VERSION: u32 = 3;

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

/// The `lifecycle` values that mean live work: a plan's or an issue's `open`
/// and a contract's or an ADR's `active` (ADR-050). Any other value —
/// `done`, `wontfix`, `abandoned`, `deprecated` — marks the document
/// settled.
const LIVE_LIFECYCLES: [&str; 2] = ["open", "active"];

/// Multiplier applied to a settled section's fused score, chosen so settled
/// work sorts below live knowledge but never disappears from results.
const DOWNRANK_FACTOR: f32 = 0.25;

/// Longest snippet, in characters.
const SNIPPET_CHARS: usize = 200;

/// Where the index lives: `.superdev/cache/sokf-index` in normal use.
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
    /// Fused score within a retrieval tier, not a confidence or probability.
    pub score: f32,
    /// Why this section was returned. Direct identity or path matches sort
    /// ahead of hybrid relevance, independent of the fused score.
    pub match_kind: MatchKind,
}

/// Search sections and recoverable query-parser diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchReport {
    /// Matching sections in retrieval order.
    pub hits: Vec<Hit>,
    /// Query syntax recovered by the lenient lexical parser.
    pub warnings: Vec<String>,
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
    /// Keep only concepts whose `lifecycle` is one of these; empty means
    /// every lifecycle, the concepts carrying none included.
    pub lifecycle: Vec<String>,
}

impl Default for SearchOpts {
    fn default() -> Self {
        SearchOpts {
            limit: DEFAULT_LIMIT,
            kinds: Vec::new(),
            tags: Vec::new(),
            lifecycle: Vec::new(),
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
    status: String,
    lifecycle: Option<String>,
}

impl SectionDoc {
    /// Settled work: a `lifecycle` value that is not a live one, or a
    /// deprecated concept. `status` stays read because documents outside the
    /// lifecycle-governed directories still use it.
    fn settled(&self) -> bool {
        self.lifecycle
            .as_deref()
            .is_some_and(|value| !LIVE_LIFECYCLES.contains(&value))
            || self.status == "deprecated"
    }
}

impl SectionDoc {
    fn hit(&self, score: f32, match_kind: MatchKind) -> Hit {
        Hit {
            path: self.path.clone(),
            concept_id: self.concept_id.clone(),
            heading_path: self.heading_path.clone(),
            start_line: self.start_line,
            end_line: self.end_line,
            snippet: self.snippet.clone(),
            score,
            match_kind,
        }
    }
}

/// An open index: a tantivy reader over the section documents, plus the
/// section vectors held in memory.
pub struct Index {
    reader: IndexReader,
    vectors: Vec<VectorRecord>,
    manifest: IndexManifest,
    // Current bundle identities, not a persisted second index. Sync already
    // loads these concepts; exact lookup need not search body prose.
    identities: BTreeMap<String, Option<String>>,
    bundle_root: PathBuf,
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
    lifecycle: Field,
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
        lifecycle: builder.add_text_field("lifecycle", STRING | STORED),
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
        identities: identities(bundle),
        bundle_root: bundle.root.clone(),
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
        identities: identities(bundle),
        bundle_root: bundle.root.clone(),
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
        if let Some(lifecycle) = &concept.lifecycle {
            doc.add_text(fields.lifecycle, lifecycle);
        }
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

fn identities(bundle: &Bundle) -> BTreeMap<String, Option<String>> {
    bundle
        .concepts
        .iter()
        .map(|concept| (concept.path.clone(), concept.id.clone()))
        .collect()
}

fn index_error(e: impl std::fmt::Display) -> Error {
    Error::Index {
        message: e.to_string(),
    }
}
