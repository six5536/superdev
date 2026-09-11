//! Retrieval, rank fusion and section provenance over the index.

use std::collections::{HashMap, HashSet};

use tantivy::collector::{DocSetCollector, TopDocs};
use tantivy::query::{BooleanQuery, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption};
use tantivy::{DocAddress, Searcher, TantivyDocument, Term};

use super::{
    CANDIDATE_FACTOR, DOWNRANK_FACTOR, DocKey, Fields, Hit, Index, RRF_K, SNIPPET_CHARS,
    SearchOpts, SectionDoc, index_error, path_hash, schema,
};
use crate::error::Result;
use crate::sokf::embed::Embedder;

impl Index {
    /// Hybrid search over the indexed sections.
    ///
    /// Lexical retrieval always runs: BM25 over the section text, terms ANDed,
    /// re-run as a disjunction only when the conjunction finds nothing.
    /// Semantic retrieval — cosine over the section vectors — joins it when
    /// the index has vectors and `embedder` is the one that built them; pass
    /// the embedder the sync used. Anything else, including `None`, leaves
    /// search lexical.
    ///
    /// [`SearchOpts::kinds`], [`SearchOpts::tags`] and
    /// [`SearchOpts::lifecycle`] filter both lists before they are fused by
    /// reciprocal rank fusion, so a filtered concept cannot re-enter through
    /// the other list.
    ///
    /// Sections of settled work — a `lifecycle` value that is not a live
    /// one, or a `deprecated` concept — are down-ranked after fusion, so
    /// finished plans, issues and maps sort below live knowledge without
    /// leaving the results.
    ///
    /// Exact IDs, unambiguous numbered shorthands and physical paths lead
    /// hybrid relevance, still subject to every explicit filter. Other hits
    /// retain fused order. The caller groups the flat list by concept.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::Index`] when tantivy fails, and whatever the embedder
    /// returns for the query.
    pub fn search(
        &self,
        query: &str,
        embedder: Option<&dyn Embedder>,
        opts: &SearchOpts,
    ) -> Result<Vec<Hit>> {
        Ok(self.search_report(query, embedder, opts)?.hits)
    }

    /// Search as [`Self::search`], retaining recoverable lexical syntax errors.
    /// A malformed query still searches for the words instead of being refused.
    ///
    /// # Errors
    /// Returns the same index and embedding errors as [`Self::search`].
    pub fn search_report(
        &self,
        query: &str,
        embedder: Option<&dyn Embedder>,
        opts: &SearchOpts,
    ) -> Result<super::SearchReport> {
        if opts.limit == 0 {
            return Ok(super::SearchReport {
                hits: Vec::new(),
                warnings: Vec::new(),
            });
        }
        let searcher = self.reader.searcher();
        let (_, fields) = schema();
        // Every pass that reads a document keeps it here, so the last step
        // re-reads only the sections semantic retrieval found on its own.
        let mut docs: HashMap<DocKey, SectionDoc> = HashMap::new();

        let (lexical, warnings) = lexical_keys(&searcher, &fields, query, opts, &mut docs)?;
        let semantic = self.semantic_keys(&searcher, &fields, query, embedder, opts, &mut docs)?;
        let mut provenance = HashMap::new();
        for key in &lexical {
            provenance.insert(*key, super::MatchKind::Lexical);
        }
        for key in &semantic {
            provenance
                .entry(*key)
                .and_modify(|kind| *kind = super::MatchKind::LexicalAndSemantic)
                .or_insert(super::MatchKind::Semantic);
        }
        let mut fused = rrf(&[lexical, semantic]);
        self.read_missing(&searcher, &fields, &fused, &mut docs)?;
        downrank(&mut fused, &docs);
        let direct = super::direct::matches(query, &self.identities, &self.bundle_root);
        let mut direct_keys = HashSet::new();
        for (path, kind) in direct {
            let mut clauses = vec![term_query(Term::from_field_text(fields.path, path))];
            if let Some(filter) = filter_query(&fields, opts) {
                clauses.push(filter);
            }
            let mut keys: Vec<_> = collect_keys(
                &searcher,
                &fields,
                &BooleanQuery::intersection(clauses),
                &mut docs,
            )?
            .into_iter()
            .collect();
            keys.sort();
            for key in keys {
                direct_keys.insert(key);
                if provenance.insert(key, kind).is_none() {
                    fused.push((key, 0.0));
                }
            }
        }
        // Stable: preserve relevance among already-retrieved direct sections;
        // newly admitted sections use deterministic path/ordinal order.
        fused.sort_by_key(|(key, _)| !direct_keys.contains(key));
        fused.truncate(opts.limit);
        Ok(super::SearchReport {
            hits: fused
                .into_iter()
                .filter_map(|(key, score)| {
                    docs.get(&key).map(|doc| doc.hit(score, provenance[&key]))
                })
                .collect(),
            warnings,
        })
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
) -> Result<(Vec<DocKey>, Vec<String>)> {
    let filter = filter_query(fields, opts);
    let mut conjunction = QueryParser::for_index(searcher.index(), vec![fields.text]);
    conjunction.set_conjunction_by_default();
    let disjunction = QueryParser::for_index(searcher.index(), vec![fields.text]);

    let mut warnings = Vec::new();
    for parser in [&conjunction, &disjunction] {
        // Keep the parser's actual diagnostics, not a syntax-debris heuristic.
        // Report at most three distinct recoveries, even after the OR retry.
        let (parsed, errors) = parser.parse_query_lenient(query);
        for error in errors {
            let message = error.to_string();
            if warnings.len() < 3 && !warnings.contains(&message) {
                warnings.push(message);
            }
        }
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
        return Ok((keys, warnings));
    }
    Ok((Vec::new(), warnings))
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

/// The `kinds`/`tags`/`lifecycle` filter as a query: any of the kinds, any of
/// the tags, and any of the lifecycle values. `None` when nothing was
/// filtered.
fn filter_query(fields: &Fields, opts: &SearchOpts) -> Option<Box<dyn Query>> {
    let any = |field: Field, values: &[String]| -> Option<Box<dyn Query>> {
        let clauses: Vec<Box<dyn Query>> = values
            .iter()
            .map(|value| term_query(Term::from_field_text(field, value)))
            .collect();
        (!clauses.is_empty()).then(|| Box::new(BooleanQuery::union(clauses)) as Box<dyn Query>)
    };
    let groups: Vec<Box<dyn Query>> = [
        any(fields.kind, &opts.kinds),
        any(fields.tags, &opts.tags),
        any(fields.lifecycle, &opts.lifecycle),
    ]
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
        status: text(fields.status).unwrap_or("stable").to_string(),
        lifecycle: text(fields.lifecycle).map(str::to_string),
        path,
    };
    Ok((key, section))
}

/// A section's opening text on one line, cut to [`SNIPPET_CHARS`].
pub(super) fn snippet(text: &str) -> String {
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
