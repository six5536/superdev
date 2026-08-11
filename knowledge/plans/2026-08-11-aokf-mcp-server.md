---
type: Plan
id: plan-aokf-mcp-server
title: AOKF MCP Server — Implementation Plan
description: Task-by-task implementation plan for the AOKF MCP server spec. Ephemeral — deleted in the commit that completes it.
status: draft
links:
  - rel: implements
    to: spec-aokf-mcp-server
    note: Edge declared plan-side only, so deleting this plan leaves no dangling references.
---

# AOKF MCP Server Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `superdev mcp aokf` (read-side MCP server: hybrid search, read, graph, overview), `superdev aokf validate`/`index`, full Python-validator replacement, and the search-first AGENTS.md switchover — per the
[AOKF MCP Server spec](../specs/2026-08-11-aokf-mcp-server-design.md).

**Architecture:** New `aokf` subsystem in superdev-core: one parser (frontmatter + heading sections with line ranges) feeds the validator, the tantivy+vector index, and the rmcp stdio server. Freshness is lazy per call via a content-hash manifest. The binary stays thin.

**Tech Stack:** rmcp 3.1.2 (features `server`, `transport-io`; tokio runtime), tantivy 0.26.1, model2vec-rs 0.2.1 (features `fancy-regex`, `hf-hub` — NOT default `onig`), serde_yaml_ng 0.10.0, pulldown-cmark 0.13.4 (no default features), serde_json (already transitive via rmcp, declared directly for `.mcp.json` merging).

## Global Constraints

- Only the dependencies above may be added, at exactly those versions. `model2vec-rs` must use `default-features = false, features = ["fancy-regex", "hf-hub"]` — the default `onig` feature links a C library and breaks static musl.
- Embedding model: `minishlab/potion-retrieval-32M`, exact revision resolved and recorded in Task 6 Step 1 (never "latest" at runtime).
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean at every commit. `#![warn(missing_docs)]` holds in superdev-core.
- Coverage ≥90% lines per crate (`npm run coverage:check`); disclosed minimal test additions are fine (no new production code for them).
- Prose: `.agents/PROSE.md`; British English. Conventional Commits; no Claude signature.
- Exit codes: validate 0 pass / 1 findings-at-level / 2 usage-or-hard-failure; BrokenPipe on stdout is success.
- MCP tools never exit the process on tool errors; they return rmcp error payloads.
- All bundle paths in outputs are bundle-relative with forward slashes; line numbers are 1-based and inclusive.
- Tests: `cargo nextest run -p superdev-core` (or `--workspace`); network-dependent tests (real model download) live only in the manual smoke script.

## File Structure

```
crates/lib/superdev-core/src/aokf/
  mod.rs        # module decls; re-exports Concept, Bundle, Section
  concept.rs    # frontmatter parsing → Concept; Section type; heading split
  bundle.rs     # bundle walk/load, reserved-file rules, manifest.aokf.yaml
  graph.rs      # link resolution, inverse synthesis, edge map, neighbourhood
  validate.rs   # document check + conformance ladder + findings + JSON
  embed.rs      # Embedder trait; Model2VecEmbedder; ApiEmbedder; model cache
  index.rs      # tantivy schema, vector store, hash manifest, staleness, search+RRF
  mcp.rs        # rmcp server: aokf_search / aokf_read / aokf_graph / aokf_overview
crates/lib/superdev-core/tests/fixtures/aokf/   # fixture bundles + golden validator JSON
crates/app/superdev/src/
  aokf_cli.rs   # `aokf validate`, `aokf index`, `mcp aokf` verb implementations
  main.rs       # new subcommands wiring (modify)
```

The `aokf` provider changes (`.mcp.json`, asset edits, validator.py removal) land in the existing `components/aokf.rs` in Tasks 12–13.

---

### Task 1: Dependencies and the concept parser

**Files:**
- Modify: `Cargo.toml` (workspace deps), `crates/lib/superdev-core/Cargo.toml`
- Create: `crates/lib/superdev-core/src/aokf/mod.rs`, `crates/lib/superdev-core/src/aokf/concept.rs`
- Modify: `crates/lib/superdev-core/src/lib.rs` (`pub mod aokf;`)

**Interfaces:**
- Produces (in `superdev_core::aokf`, defined in `concept.rs`, re-exported from `mod.rs`):

```rust
pub struct Concept {
    pub path: String,                 // bundle-relative, forward slashes
    pub kind: String,                 // frontmatter `type` (required)
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Status,               // absent ⇒ Stable
    pub tags: Vec<String>,
    pub resource: Option<String>,
    pub sources: Vec<Source>,
    pub links: Vec<Link>,
    pub raw: serde_yaml_ng::Value,    // full frontmatter for validator-only checks
    pub sections: Vec<Section>,
}
pub enum Status { Draft, Stable, Deprecated }
pub struct Source { pub id: Option<String>, pub resource: Option<String>, pub title: Option<String> }
pub struct Link { pub rel: Option<String>, pub to: Option<String>, pub note: Option<String> }
pub struct Section {
    pub heading_path: Vec<String>,    // [] for the root section
    pub start_line: usize,            // 1-based inclusive
    pub end_line: usize,
    pub text: String,
}
/// Parse one concept file's full text. The body is split at headings in
/// Task 2 — here `sections` contains ONLY the root section (frontmatter
/// title + description + tags joined by newlines, lines 1..frontmatter end).
pub fn parse_concept(path: &str, text: &str) -> Result<Concept>;
pub struct ParseError { pub path: String, pub message: String }
```

  `Result` here is `std::result::Result<Concept, ParseError>` — a malformed concept is data for the validator, not a crate `Error`. Missing/empty `type` parses (validator flags it): `kind` falls back to `""`. Fields with wrong YAML types (e.g. `links` as a string) parse to empty/None and are preserved in `raw` for the validator. Frontmatter is delimited by a first line `---` and the next `---` line; a file without frontmatter is a `ParseError`.

- [ ] **Step 1: Add dependencies**

Workspace `Cargo.toml` under `# External dependencies (alphabetical)`:

```toml
pulldown-cmark = { version = "0.13.4", default-features = false }
rmcp = { version = "3.1.2", features = ["server", "transport-io"] }
serde_json = "1"
serde_yaml_ng = "0.10.0"
tantivy = "0.26.1"
```

and (alphabetical position):

```toml
model2vec-rs = { version = "0.2.1", default-features = false, features = ["fancy-regex", "hf-hub"] }
```

In `crates/lib/superdev-core/Cargo.toml` add all six as `{ workspace = true }`. Run `cargo build -p superdev-core` once now — surfacing any feature-resolution surprise before writing code is the point of this step; if `model2vec-rs` or `rmcp` fail to build with exactly these features, STOP and report (do not improvise feature sets).

- [ ] **Step 2: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "---\ntype: Module\nid: planner\ndescription: Pure planning stage.\ntags: [core]\nlinks:\n  - rel: depends-on\n    to: config\n---\n\n# Role\n\nBody text.\n";

    #[test]
    fn parses_frontmatter_fields() {
        let c = parse_concept("planner.md", DOC).unwrap();
        assert_eq!(c.kind, "Module");
        assert_eq!(c.id.as_deref(), Some("planner"));
        assert_eq!(c.tags, vec!["core"]);
        assert_eq!(c.links.len(), 1);
        assert_eq!(c.links[0].rel.as_deref(), Some("depends-on"));
        assert!(matches!(c.status, Status::Stable));
    }

    #[test]
    fn root_section_carries_frontmatter_text_and_lines() {
        let c = parse_concept("planner.md", DOC).unwrap();
        let root = &c.sections[0];
        assert!(root.heading_path.is_empty());
        assert_eq!(root.start_line, 1);
        assert!(root.text.contains("Pure planning stage."));
    }

    #[test]
    fn missing_type_parses_with_empty_kind() {
        let c = parse_concept("x.md", "---\nid: x\n---\nbody\n").unwrap();
        assert_eq!(c.kind, "");
    }

    #[test]
    fn no_frontmatter_is_a_parse_error() {
        assert!(parse_concept("x.md", "just markdown\n").is_err());
    }

    #[test]
    fn wrong_typed_fields_degrade_but_raw_survives() {
        let c = parse_concept("x.md", "---\ntype: T\nlinks: nope\n---\nb\n").unwrap();
        assert!(c.links.is_empty());
        assert_eq!(c.raw["links"].as_str(), Some("nope"));
    }
}
```

- [ ] **Step 3: Run to verify failure** — `cargo nextest run -p superdev-core aokf::concept` fails to compile.

- [ ] **Step 4: Implement**

`mod.rs`:

```rust
//! AOKF: parsing, validation, indexing, and the MCP server for the
//! knowledge bundle format defined in `.agents/aokf/SPEC.md`.

pub mod concept;

pub use concept::{Concept, Link, ParseError, Section, Source, Status, parse_concept};
```

`concept.rs` implementation notes (the tests above are the contract):
- Split frontmatter: first line must be exactly `---`; scan for the next line that is exactly `---`. Record `fm_end_line` (1-based line number of the closing fence).
- Parse the YAML block with `serde_yaml_ng::from_str::<serde_yaml_ng::Value>` → `raw`. A YAML error is a `ParseError` carrying the yaml message.
- Extract typed fields from `raw` defensively: strings via `.as_str()`, sequences via `.as_sequence()`, each element again defensively (a non-map link entry → `Link { rel: None, to: None, note: None }`). `status`: `"draft"`/`"deprecated"` map, anything else ⇒ `Stable`.
- Root section: `heading_path: vec![]`, `start_line: 1`, `end_line: fm_end_line`, `text` = title, description, and `tags` joined with `\n` (skip absent parts).
- Body is NOT split here; `sections` len is exactly 1 after this task.
- `#![warn(missing_docs)]` applies: doc-comment every public item.

- [ ] **Step 5: Run to verify pass**, then `cargo clippy --workspace --all-targets -- -D warnings` and fmt.

- [ ] **Step 6: Commit** — `git commit -m "feat(core): aokf concept parser and dependency set"`

---

### Task 2: Bundle loading and heading sections

**Files:**
- Create: `crates/lib/superdev-core/src/aokf/bundle.rs`
- Modify: `crates/lib/superdev-core/src/aokf/concept.rs` (body splitting), `mod.rs`

**Interfaces:**
- Consumes: Task 1's types.
- Produces:

```rust
// concept.rs — replaces the single-root-section behaviour:
/// After this task, `sections[0]` is the root section and subsequent
/// entries are one per markdown heading (any level), in document order.
/// `heading_path` is the full path, e.g. ["Verbs", "superdev init"].
/// A section spans its heading line to the line before the next heading
/// (or EOF). Fenced code blocks are NOT scanned for headings.

// bundle.rs:
pub struct Bundle {
    pub root: PathBuf,                       // absolute bundle dir
    pub manifest: Option<BundleManifest>,    // manifest.aokf.yaml
    pub concepts: Vec<Concept>,              // parse successes, sorted by path
    pub broken: Vec<ParseError>,             // parse failures
}
pub struct BundleManifest { pub aokf: Option<String>, pub name: Option<String> }
/// Walk `dir` for `.md` files (recursive). `manifest.aokf.yaml` (root only)
/// and any `index.md` are reserved, not concepts. Hidden dirs skipped.
pub fn load_bundle(dir: &Path) -> Result<Bundle>;   // crate::error::Result — Io errors only
```

- [ ] **Step 1: Write the failing tests**

```rust
// concept.rs additions
#[test]
fn splits_body_at_headings_with_line_ranges() {
    let doc = "---\ntype: T\n---\nintro ignored? no — pre-heading body joins the root section\n\n# One\n\nalpha\n\n## Sub\n\nbeta\n\n# Two\n\ngamma\n";
    let c = parse_concept("x.md", doc).unwrap();
    let paths: Vec<Vec<String>> = c.sections.iter().map(|s| s.heading_path.clone()).collect();
    assert_eq!(paths[0], Vec::<String>::new());
    assert_eq!(paths[1], vec!["One"]);
    assert_eq!(paths[2], vec!["One", "Sub"]);
    assert_eq!(paths[3], vec!["Two"]);
    let one = &c.sections[1];
    assert!(one.text.contains("alpha"));
    assert!(!one.text.contains("beta"));
    // heading line itself is the section start
    assert_eq!(doc.lines().nth(one.start_line - 1).unwrap(), "# One");
}

#[test]
fn headings_inside_code_fences_are_not_sections() {
    let doc = "---\ntype: T\n---\n\n# Real\n\n```\n# not a heading\n```\n";
    let c = parse_concept("x.md", doc).unwrap();
    assert_eq!(c.sections.len(), 2); // root + Real
}
```

```rust
// bundle.rs tests (tempfile)
#[test]
fn loads_concepts_and_skips_reserved_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("manifest.aokf.yaml"), "aokf: \"0.1\"\nname: t\n").unwrap();
    std::fs::write(dir.path().join("index.md"), "# Index\n").unwrap();
    std::fs::write(dir.path().join("a.md"), "---\ntype: T\nid: a\n---\nbody\n").unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();
    std::fs::write(dir.path().join("sub/index.md"), "# Sub\n").unwrap();
    std::fs::write(dir.path().join("sub/b.md"), "---\ntype: T\nid: b\n---\nbody\n").unwrap();
    std::fs::write(dir.path().join("sub/broken.md"), "no frontmatter\n").unwrap();
    let b = load_bundle(dir.path()).unwrap();
    assert_eq!(b.manifest.as_ref().unwrap().name.as_deref(), Some("t"));
    let paths: Vec<&str> = b.concepts.iter().map(|c| c.path.as_str()).collect();
    assert_eq!(paths, vec!["a.md", "sub/b.md"]);
    assert_eq!(b.broken.len(), 1);
}
```

- [ ] **Step 2: Run to verify failure**, then implement. Splitting uses `pulldown_cmark::Parser` with `Options::empty()` over the body only, tracking heading events + byte offsets (`into_offset_iter()`), converting byte offsets to 1-based line numbers against a precomputed line-start table offset by the frontmatter length. `heading_path` maintenance: keep a stack keyed by heading level (an `h2` after an `h1` nests; an `h1` resets). Pre-heading body text belongs to the root section (its `end_line` extends to the first heading line − 1); the root section text gains that body text appended after the frontmatter-derived text.

- [ ] **Step 3: Run to verify pass**, clippy, fmt.

- [ ] **Step 4: Commit** — `git commit -m "feat(core): aokf bundle loading and heading sections"`

---

### Task 3: The link graph

**Files:**
- Create: `crates/lib/superdev-core/src/aokf/graph.rs`
- Modify: `mod.rs`

**Interfaces:**
- Consumes: `Bundle`, `Concept`, `Link`.
- Produces:

```rust
pub struct Graph { /* built once per bundle load */ }
pub struct Edge {
    pub from: String,           // concept identity: id if set, else path
    pub rel: String,
    pub to: String,             // resolved identity, or the raw target if unresolved
    pub note: Option<String>,
    pub resolved: bool,
    pub synthesised: bool,      // true for inverse edges
}
impl Graph {
    pub fn build(bundle: &Bundle) -> Graph;
    /// Declared edges only, grouped by source, source order = concept path order.
    pub fn edge_map(&self) -> Vec<Edge>;
    /// Single hop, both directions (declared + synthesised inverses),
    /// deduplicated. Unknown id → Err with up to 3 near-miss candidates.
    pub fn neighbours(&self, id: &str) -> Result<Vec<Edge>, UnknownId>;
    /// Resolve an id-or-path to the concept's identity, if present.
    pub fn resolve(&self, target: &str) -> Option<String>;
}
pub struct UnknownId { pub asked: String, pub candidates: Vec<String> }
/// SPEC §8 inverse vocabulary: relates-to↔relates-to, part-of↔has-part,
/// depends-on↔depended-on-by, references↔referenced-by,
/// supersedes↔superseded-by, contradicts↔contradicts; unknown rels
/// synthesise as relates-to.
pub fn inverse_rel(rel: &str) -> &str;
```

  Resolution per SPEC §8: `to` resolves as an `id` first, then as a `/`-rooted or bundle-relative path. Near-miss candidates: case-insensitive substring match on ids, then Levenshtein? No — keep it simple and deterministic: ids sharing a prefix or containing the asked string, first 3 in path order.

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::aokf::{load_bundle};

    fn bundle_with(files: &[(&str, &str)]) -> crate::aokf::Bundle {
        let dir = tempfile::tempdir().unwrap();
        for (p, t) in files {
            let path = dir.path().join(p);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, t).unwrap();
        }
        let b = load_bundle(dir.path()).unwrap();
        b
    }

    const A: &str = "---\ntype: T\nid: alpha\ndescription: A.\nlinks:\n  - rel: depends-on\n    to: beta\n---\nbody\n";
    const B: &str = "---\ntype: T\nid: beta\ndescription: B.\n---\nbody\n";

    #[test]
    fn edge_map_lists_declared_edges_only() {
        let g = Graph::build(&bundle_with(&[("a.md", A), ("beta.md", B)]));
        let edges = g.edge_map();
        assert_eq!(edges.len(), 1);
        assert_eq!((edges[0].from.as_str(), edges[0].rel.as_str(), edges[0].to.as_str()), ("alpha", "depends-on", "beta"));
        assert!(edges[0].resolved);
        assert!(!edges[0].synthesised);
    }

    #[test]
    fn neighbours_include_synthesised_inverse() {
        let g = Graph::build(&bundle_with(&[("a.md", A), ("beta.md", B)]));
        let n = g.neighbours("beta").unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].rel, "depended-on-by");
        assert!(n[0].synthesised);
    }

    #[test]
    fn unknown_id_names_candidates() {
        let g = Graph::build(&bundle_with(&[("a.md", A), ("beta.md", B)]));
        let err = g.neighbours("bet").unwrap_err();
        assert_eq!(err.candidates, vec!["beta"]);
    }

    #[test]
    fn unresolved_targets_are_flagged_not_dropped() {
        let g = Graph::build(&bundle_with(&[("a.md", A)]));
        let edges = g.edge_map();
        assert!(!edges[0].resolved);
    }

    #[test]
    fn inverse_vocabulary_matches_the_spec() {
        assert_eq!(inverse_rel("part-of"), "has-part");
        assert_eq!(inverse_rel("contradicts"), "contradicts");
        assert_eq!(inverse_rel("custom-thing"), "relates-to");
    }
}
```

- [ ] **Step 2: Run to verify failure**, implement (`Graph` holds identity→index maps and the declared edge list; `neighbours` walks declared edges touching the id in either direction, synthesising the inverse for incoming ones). Truncation (~30/group) is presentation, owned by the MCP layer in Task 9 — not here.

- [ ] **Step 3: Run to verify pass**, clippy, fmt.

- [ ] **Step 4: Commit** — `git commit -m "feat(core): aokf link graph with inverse synthesis"`

---

### Task 4: The validator

**Files:**
- Create: `crates/lib/superdev-core/src/aokf/validate.rs`
- Modify: `mod.rs`

**Interfaces:**
- Consumes: `Bundle`, `Concept`, `Graph`.
- Produces:

```rust
pub struct Finding { pub path: String, pub message: String, pub error_at: Option<u8> }  // level at which this is an error; None = always a warning
pub struct Report {
    pub achieved_level: u8,       // 0..=2 (or the level below the first failure)
    pub checked_level: u8,
    pub findings: Vec<Finding>,
    pub concept_count: usize,
}
impl Report {
    pub fn passed(&self) -> bool;                    // no finding with error_at <= checked_level
    pub fn to_json(&self) -> serde_json::Value;      // shape mirrors the Python --json output
    pub fn render_human(&self) -> String;            // the CLI text form
}
/// Run the document check (SPEC §10) and grade the ladder (§11).
/// `repo_root` resolves `/`-rooted targets.
pub fn validate(bundle: &Bundle, repo_root: &Path, checked_level: u8) -> Report;
```

  Checks to implement — this is the full list; each maps to a rule in SPEC §10/§11, and the fixture matrix in Task 5 exercises every one:
  - frontmatter parses; `type` present and non-empty (level 0)
  - stamped field `generated` absent (level 0)
  - `verified` well-formed when present: list (or single map, read as one-element list) of `{by, at}`, `by` matches `^(human|process):.+`, `at` ISO 8601 (level 0)
  - `id` a valid slug (`^[a-z0-9]+(-[a-z0-9]+)*$`) (level 0); unique across the bundle (level 0 for duplicates); every concept has an id (level 1)
  - manifest present declaring `aokf` and `name` (level 1)
  - `links` entries have `rel` and `to` (level 0); `to` resolves (level 2); mirrored by a body markdown link to the same target (level 2)
  - warnings (never errors, any level): broken `/`-paths and relative body links, `sources` entries cited by a footnote but lacking `id`, footnote labels with no matching source, `index.md` entries pointing at missing files, non-core `rel` values ("read as relates-to")
  - `.md` files that failed to parse are level-0 errors carrying the parse message
  Body links for mirroring/broken-link checks come from `pulldown_cmark` link events over the body (reuse the section pass or a second parse — implementer's choice; do not regex).

- [ ] **Step 1: Write the failing tests** — three representative unit tests now (the exhaustive matrix is Task 5's fixtures):

```rust
#[test]
fn clean_bundle_passes_level_2() {
    // build a 2-concept bundle with a mirrored link and a manifest (reuse graph.rs's helper shape)
    let b = bundle_with(&[("manifest.aokf.yaml", "aokf: \"0.1\"\nname: t\n"), ("a.md", A_MIRRORED), ("beta.md", B)]);
    let r = validate(&b, b.root.as_path(), 2);
    assert!(r.passed());
    assert_eq!(r.achieved_level, 2);
}

#[test]
fn duplicate_ids_fail_level_0() {
    let b = bundle_with(&[("a.md", "---\ntype: T\nid: dup\n---\nx\n"), ("b.md", "---\ntype: T\nid: dup\n---\nx\n")]);
    let r = validate(&b, b.root.as_path(), 2);
    assert!(!r.passed());
    assert!(r.findings.iter().any(|f| f.message.contains("dup")));
}

#[test]
fn unmirrored_link_fails_only_at_level_2() {
    let b = bundle_with(&[("a.md", "---\ntype: T\nid: a\nlinks:\n  - rel: depends-on\n    to: b\n---\nno body link\n"), ("b.md", "---\ntype: T\nid: b\n---\nx\n")]);
    assert!(validate(&b, b.root.as_path(), 1).passed());
    assert!(!validate(&b, b.root.as_path(), 2).passed());
}
```

- [ ] **Step 2: Run to verify failure**, implement. Keep each check a small private function taking `(&Bundle, &Graph, &mut Vec<Finding>)`; `validate` composes them. `to_json` field names must match the Python validator's JSON — open `.agents/aokf/tools/validator.py` and copy its `as_dict`/output shape exactly (keys, level grading fields); Task 5 locks this with goldens.

- [ ] **Step 3: Run to verify pass**, clippy, fmt.

- [ ] **Step 4: Commit** — `git commit -m "feat(core): aokf validator in rust"`

---

### Task 5: Validator parity fixtures

**Files:**
- Create: `crates/lib/superdev-core/tests/fixtures/aokf/<case>/…` (fixture bundles)
- Create: `crates/lib/superdev-core/tests/fixtures/aokf/<case>.golden.json` (Python output)
- Create: `crates/lib/superdev-core/tests/validator_parity.rs`

**Interfaces:**
- Consumes: `aokf::{load_bundle, validate}`.
- Produces: the golden files later tasks and CI rely on after the Python validator is deleted.

- [ ] **Step 1: Build the fixture matrix**

One fixture bundle per failure class, each tiny (1–3 files). Cases: `clean` (passes level 2), `broken-links` (unresolvable `to` + broken body path), `duplicate-ids`, `malformed-frontmatter` (bad YAML), `missing-type`, `stamped-field` (`generated` present), `bad-verified` (wrong `by` form, bad date), `unmirrored-link`, `no-manifest`, `custom-rel` (warning only), `footnote-mismatch` (cited source without id + orphan footnote). Also treat this repo's live `knowledge/` as a case (validated in place, not copied).

- [ ] **Step 2: Generate goldens with the Python validator**

```bash
for d in crates/lib/superdev-core/tests/fixtures/aokf/*/; do
  name=$(basename "$d")
  python3 .agents/aokf/tools/validator.py "$d" --json --repo-root "$d" > "crates/lib/superdev-core/tests/fixtures/aokf/$name.golden.json" || true
done
python3 .agents/aokf/tools/validator.py knowledge --json > /tmp/live.golden.json || true
```

Inspect each golden — a golden that is itself wrong (Python crash output, empty file) is a fixture bug to fix now, not to enshrine.

- [ ] **Step 3: Write the parity test**

```rust
// validator_parity.rs
use std::path::Path;
use superdev_core::aokf::{load_bundle, validate};

fn parity(case: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aokf").join(case);
    let golden: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.with_extension("golden.json")).unwrap()).unwrap();
    let bundle = load_bundle(&dir).unwrap();
    let ours = validate(&bundle, &dir, 2).to_json();
    assert_eq!(ours, golden, "parity failure for {case}");
}

#[test] fn clean() { parity("clean") }
#[test] fn broken_links() { parity("broken-links") }
// … one #[test] per case, same shape
```

Sort findings deterministically (path, then message) on BOTH sides — if the Python output isn't sorted, sort the golden's findings array in Step 2 with `jq` and sort ours in `to_json`; note this normalisation in the test file header.

- [ ] **Step 4: Run** — `cargo nextest run -p superdev-core --test validator_parity`. Fix divergences in the Rust side (the Python behaviour is the spec here) until all cases pass. Then run against the live bundle: `cargo run --quiet -- aokf validate knowledge --json` is not wired yet — instead call `validate()` on `knowledge/` in one more test and compare to `/tmp/live.golden.json` content pasted as a fixture? No — the live bundle changes; instead assert `passed() && achieved_level == 2`, not JSON equality.

- [ ] **Step 5: Commit** — `git commit -m "test(core): validator parity fixtures against the python reference"`

---

### Task 6: Embedders

**Files:**
- Create: `crates/lib/superdev-core/src/aokf/embed.rs`
- Modify: `mod.rs`; `crates/lib/superdev-core/src/manifest.rs` (embeddings sub-table)

**Interfaces:**
- Produces:

```rust
pub trait Embedder {
    /// Stable identifier recorded in the index manifest; a change forces a rebuild.
    fn model_id(&self) -> String;
    fn embed(&self, texts: &[String]) -> crate::error::Result<Vec<Vec<f32>>>;
}
pub struct Model2VecEmbedder { /* loaded model */ }
impl Model2VecEmbedder {
    /// Load from the user cache, downloading on first use.
    /// Cache: dirs-equivalent of ~/.cache/superdev/models/<model>/<revision>/
    /// (respect $XDG_CACHE_HOME; no new dirs crate — a small helper fn).
    pub fn load() -> crate::error::Result<Model2VecEmbedder>;
}
pub struct ApiEmbedder { /* provider, model, endpoint, key from env */ }
pub struct EmbeddingsConfig { pub provider: String, pub model: String }  // manifest [<capability>.embeddings]
/// Choose the embedder from the manifest knowledge entry: Some(api config)
/// → ApiEmbedder (key from OPENAI_API_KEY-style env named per provider),
/// None → Model2VecEmbedder. Load failure → Ok(None) = lexical-only mode.
pub fn embedder_from(config: Option<&EmbeddingsConfig>) -> crate::error::Result<Option<Box<dyn Embedder>>>;
pub const LOCAL_MODEL: &str = "minishlab/potion-retrieval-32M";
pub const LOCAL_MODEL_REVISION: &str = "RESOLVED-IN-STEP-1";
#[cfg(test)] pub(crate) struct FakeEmbedder;  // deterministic: hash-based unit vectors
```

  `manifest.rs`: `CapabilityConfig` gains `#[serde(default, skip_serializing_if = "Option::is_none")] pub embeddings: Option<EmbeddingsConfig>` — a shape change to a committed file format; existing manifests parse unchanged (field optional). `ApiEmbedder` supports `provider = "openai"` only for now (endpoint `https://api.openai.com/v1/embeddings`, key `OPENAI_API_KEY`) via `ureq` — which is ALREADY in the tree as model2vec-rs's hf-hub transitive; declare it as a workspace dep pinned to the same version rather than adding anything new. Unknown provider → `Error::Manifest`.

- [ ] **Step 1: Resolve the model revision (recorded, not guessed)**

```bash
curl -s https://huggingface.co/api/models/minishlab/potion-retrieval-32M | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['sha'])"
```

Record the commit sha as `LOCAL_MODEL_REVISION`. Also record in the report which files the crate downloads (model2vec-rs fetches `model.safetensors`, `tokenizer.json`, `config.json`) and their sizes.

- [ ] **Step 2: Write the failing tests** (no network in tests — FakeEmbedder and config selection only):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_embedder_is_deterministic_and_normalised() {
        let f = FakeEmbedder;
        let a = f.embed(&["hello".into()]).unwrap();
        let b = f.embed(&["hello".into()]).unwrap();
        assert_eq!(a, b);
        let norm: f32 = a[0].iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn api_config_selects_api_embedder_and_bad_provider_errors() {
        let cfg = EmbeddingsConfig { provider: "openai".into(), model: "text-embedding-3-small".into() };
        // Construction must not hit the network or require the key; only embed() needs it.
        assert!(embedder_from(Some(&cfg)).unwrap().is_some());
        let bad = EmbeddingsConfig { provider: "nope".into(), model: "x".into() };
        assert!(embedder_from(Some(&bad)).is_err());
    }

    #[test]
    fn manifest_accepts_embeddings_subtable() {
        let m = crate::manifest::Manifest::parse(
            "blueprint = \"0.1.0\"\n[knowledge]\nprovider = \"aokf\"\n[knowledge.embeddings]\nprovider = \"openai\"\nmodel = \"text-embedding-3-small\"\n",
        ).unwrap();
        assert_eq!(m.capabilities["knowledge"].embeddings.as_ref().unwrap().provider, "openai");
    }
}
```

- [ ] **Step 3: Run to verify failure**, implement. `FakeEmbedder`: seed a simple FNV-style hash per text into 8 dims, L2-normalise — deterministic, no deps. `Model2VecEmbedder::load()` builds the cache dir, downloads via `model2vec_rs`'s hf-hub path pinned to `LOCAL_MODEL_REVISION`, loads from disk when present; unit tests do NOT call it (network) — its coverage comes from the manual smoke. Mark it `#[cfg_attr(coverage_nightly, coverage(off))]` with a comment (untestable without network), consistent with existing practice for untestable glue.

- [ ] **Step 4: Run to verify pass**, clippy, fmt. `cargo build --target x86_64-unknown-linux-musl -p superdev-core` if the target is installed (it is in CI; locally best-effort) to confirm no C linkage crept in.

- [ ] **Step 5: Commit** — `git commit -m "feat(core): embedding providers with local model2vec default"`

---

### Task 7: The index — storage, staleness, incremental update

**Files:**
- Create: `crates/lib/superdev-core/src/aokf/index.rs`
- Modify: `mod.rs`

**Interfaces:**
- Consumes: `Bundle`, `Section`, `Embedder`.
- Produces:

```rust
pub struct Index { /* tantivy index + vectors + manifest, all under dir */ }
pub struct IndexDir(pub PathBuf);              // .superdev/cache/aokf-index
pub const SCHEMA_VERSION: u32 = 1;
impl Index {
    /// Open or create at `dir`, then bring up to date against `bundle`:
    /// compare per-file sha256 against the stored manifest; re-parse/re-embed
    /// changed+new files, remove deleted ones. Full rebuild when
    /// SCHEMA_VERSION or the embedder's model_id differs (or embedder became
    /// None/Some). Returns stats for reporting.
    pub fn open_and_sync(dir: &IndexDir, bundle: &Bundle, embedder: Option<&dyn Embedder>) -> crate::error::Result<(Index, SyncStats)>;
    pub fn force_rebuild(dir: &IndexDir, bundle: &Bundle, embedder: Option<&dyn Embedder>) -> crate::error::Result<(Index, SyncStats)>;
}
pub struct SyncStats { pub reindexed: usize, pub removed: usize, pub full_rebuild: bool, pub lexical_only: bool }
```

  Tantivy schema: fields `path` (STRING stored), `concept_id` (STRING stored), `heading_path` (STRING stored, `>`-joined), `start_line`/`end_line` (u64 stored), `kind` (STRING stored+indexed), `tags` (STRING indexed, multi), `text` (TEXT indexed stored). Vector store: `vectors.bin` — bincode-free hand-rolled: header (dim, count) + per-record (path-hash u64, section ordinal u32, f32×dim), rewritten wholesale on any change (trivial at this scale); manifest `manifest.json` (serde_json): `{schema_version, model_id: Option<String>, files: {path: sha256}}`. Reuse `crate::lock::sha256_hex`.

- [ ] **Step 1: Write the failing tests** (all with `FakeEmbedder` or `None`):

```rust
#[test]
fn first_open_indexes_everything() {
    let (dir, bundle) = fixture();                       // tempdir bundle: 2 concepts, 3 sections
    let idx = IndexDir(dir.path().join("idx"));
    let (_, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    assert!(stats.full_rebuild);
    assert_eq!(stats.reindexed, 2);
}

#[test]
fn unchanged_bundle_syncs_nothing() { /* second open_and_sync → reindexed 0, full_rebuild false */ }

#[test]
fn edited_file_reindexes_only_that_file() { /* touch one file with new content → reindexed 1 */ }

#[test]
fn deleted_file_is_removed() { /* delete one → removed 1; its sections stop matching in search (asserted in Task 8) */ }

#[test]
fn model_change_forces_full_rebuild() { /* sync with FakeEmbedder, then with None → full_rebuild, lexical_only */ }
```

(Write the bodies out fully when implementing — the comments above name the exact behaviour each must assert; every test constructs its own tempdir fixture.)

- [ ] **Step 2: Run to verify failure**, implement. Incremental tantivy: delete-by-term on `path` then re-add that file's section documents in one writer commit. `open_and_sync` never touches the network — the embedder is constructed by the caller.

- [ ] **Step 3: Run to verify pass**, clippy, fmt.

- [ ] **Step 4: Commit** — `git commit -m "feat(core): aokf index with lazy incremental sync"`

---

### Task 8: Search — BM25, cosine, fusion

**Files:**
- Modify: `crates/lib/superdev-core/src/aokf/index.rs`

**Interfaces:**
- Produces:

```rust
pub struct Hit {
    pub path: String, pub concept_id: Option<String>,
    pub heading_path: Vec<String>, pub start_line: usize, pub end_line: usize,
    pub snippet: String,          // first ~200 chars of the section text, single line
    pub score: f32,               // fused score
}
pub struct SearchOpts { pub limit: usize, pub kinds: Vec<String>, pub tags: Vec<String> }  // limit default 8
impl Index {
    /// Hybrid search. Lexical always runs; semantic runs when the index has
    /// vectors. Filters (kinds/tags) apply to both lists pre-fusion.
    /// Fusion: reciprocal rank fusion, k = 60: score = Σ 1/(60 + rank).
    pub fn search(&self, query: &str, embedder: Option<&dyn Embedder>, opts: &SearchOpts) -> crate::error::Result<Vec<Hit>>;
}
```

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn lexical_search_finds_the_right_section() {
    // fixture: concept "release" with a section containing "tag-driven pipeline",
    // concept "testing" with unrelated text
    let hits = idx.search("tag-driven release pipeline", None, &SearchOpts::default()).unwrap();
    assert_eq!(hits[0].concept_id.as_deref(), Some("release"));
    assert!(hits[0].start_line > 0 && hits[0].end_line >= hits[0].start_line);
}

#[test]
fn semantic_contributes_when_vectors_exist() {
    // FakeEmbedder is hash-based, so semantic similarity == exact text match;
    // craft the query to equal one section's text so cosine ranks it top,
    // while a different section wins BM25 — fused winner must carry both ranks.
    // Assert: a hit present in both lists outranks a hit present in one.
}

#[test]
fn filters_restrict_kinds_and_tags() { /* kinds: ["Spec"] excludes non-spec hits */ }

#[test]
fn fusion_maths() {
    // pure function test: rrf(&[list_a, list_b]) — item in both at ranks 0 and 1
    // scores 1/60 + 1/61; item only at rank 0 scores 1/60. Expose rrf as
    // pub(crate) fn rrf(lists: &[Vec<DocKey>]) -> Vec<(DocKey, f32)> and test directly.
}
```

(As in Task 7: the comment lines name the required assertions; write them out fully.)

- [ ] **Step 2: Run to verify failure**, implement (tantivy `QueryParser` over `text` with `AND`-preferred terms falling back to `OR`; cosine over the in-memory vector table; both lists truncated at `limit * 4` before fusion; grouped-by-concept ordering happens in the MCP layer, `search` returns the flat ranked list).

- [ ] **Step 3: Run to verify pass**, clippy, fmt.

- [ ] **Step 4: Commit** — `git commit -m "feat(core): hybrid search with reciprocal rank fusion"`

---

### Task 9: The MCP server

**Files:**
- Create: `crates/lib/superdev-core/src/aokf/mcp.rs`
- Modify: `mod.rs`

**Interfaces:**
- Consumes: everything above.
- Produces:

```rust
pub struct AokfServer { /* bundle dir, repo root, index dir, embedder */ }
impl AokfServer {
    pub fn new(bundle_dir: PathBuf, repo_root: PathBuf, index_dir: IndexDir,
               embedder: Option<Box<dyn Embedder>>) -> AokfServer;
    /// Serve MCP over the given transport until the client disconnects.
    pub async fn serve_stdio(self) -> crate::error::Result<()>;
}
```

  Four tools via rmcp's `#[tool_router]`/`#[tool]` macros (rmcp 3.1 server pattern — follow the rmcp docs.rs examples for `ServerHandler` + `tool_router!`). Tool behaviours (each call begins with `open_and_sync` — that IS the lazy freshness rule; a sync error becomes a tool error):

  - `aokf_search { query: String, limit: Option<u32>, types: Option<Vec<String>>, tags: Option<Vec<String>> }` → text content: hits grouped by concept, each line `path:start-end  [heading > path]  snippet` with the concept line first (`id — description`). Append `note: semantic search unavailable (lexical only)` when the embedder is absent.
  - `aokf_read { id: String, heading: Option<String> }` → whole concept (or one section when `heading` matches a heading-path segment join): frontmatter summary block (type, status, tags, links one-per-line), then body with a `path:start-end` header per section. Unknown id → tool error listing `UnknownId::candidates`.
  - `aokf_graph { id: Option<String> }` → no id: edge map lines `from --rel--> to  (note)` grouped by source; with id: neighbour lines `--rel--> id — description` / `<--rel-- id — description`. Both modes: cap 30 lines per group, then `+N more (rels: a, b)`.
  - `aokf_overview {}` → manifest name, concept count, `SyncStats` one-liner when work happened, then the directory tree (`dir/` lines, `  id — description` under each), then, when the current bundle fails validation at level 2, a `warnings:` block with the first 10 findings.

- [ ] **Step 1: Write the failing integration test** (this task's test IS the in-process client test):

Create `crates/lib/superdev-core/tests/mcp_tools.rs` — rmcp client and server over `tokio::io::duplex`:

```rust
// Fixture bundle: 3 concepts (one Spec, one Module with links both ways, one draft),
// manifest, one broken relative link (to exercise the overview warning block).
// Helper: async fn serve_and_client(bundle: TempDir) -> (RunningService<RoleClient, ()>, …)
// using rmcp::serve_client / serve_server over duplex halves — copy the pattern from
// rmcp's own transport tests (docs.rs rmcp examples "in-process").

#[tokio::test]
async fn search_returns_locators() {
    // call aokf_search {"query": "planning stage"}; assert the text content
    // contains "module-a.md:" and a "-" line range, and the concept id line.
}

#[tokio::test]
async fn read_whole_and_section() { /* aokf_read id → contains both section headers; with heading → only that section; unknown id → is_error with candidate */ }

#[tokio::test]
async fn graph_map_and_neighbours() { /* no-id → declared edge line; id → synthesised inverse line present */ }

#[tokio::test]
async fn overview_orients_and_warns() { /* name, count, tree line, warnings block for the broken fixture */ }

#[tokio::test]
async fn stale_index_refreshes_between_calls() {
    // call search; append a new section to a concept file on disk; call search
    // again for the new text; assert it is found (lazy sync per call).
}
```

Write these fully — the comment lines are the required assertions. `tokio` dev-dependency for superdev-core: `tokio = { workspace = true, features = ["macros", "rt", "io-util"] }` (tokio is already in the workspace tree via rmcp; declare the workspace entry in Task 1 if `cargo` asks for it here — version matching rmcp's requirement).

- [ ] **Step 2: Run to verify failure**, implement `mcp.rs`. All rendering is plain string building in small private fns (`render_hits`, `render_concept`, `render_edges`, `render_overview`) — unit-test the truncation rule (`+N more`) directly on `render_edges` with 35 synthetic edges.

- [ ] **Step 3: Run to verify pass** (`cargo nextest run -p superdev-core --test mcp_tools mcp`), clippy, fmt.

- [ ] **Step 4: Commit** — `git commit -m "feat(core): aokf mcp server with search, read, graph and overview"`

---

### Task 10: Binary verbs

**Files:**
- Create: `crates/app/superdev/src/aokf_cli.rs`
- Modify: `crates/app/superdev/src/main.rs`
- Modify: `crates/app/superdev/Cargo.toml` (add `tokio = { workspace = true, features = ["rt", "io-std", "macros"] }`)

**Interfaces:**
- Consumes: `superdev_core::aokf::*`, existing `manage.rs` patterns (`out()` helper, exit-code mapping).
- Produces, in `main.rs`'s `enum Command`:

```rust
    /// Serve project subsystems over MCP
    #[command(subcommand)]
    Mcp(McpCommand),
    /// AOKF knowledgebase commands
    #[command(subcommand)]
    Aokf(AokfCommand),
// aokf_cli.rs:
pub enum McpCommand { /// Serve the AOKF bundle over stdio
    Aokf }
pub enum AokfCommand {
    /// Validate the bundle against the AOKF spec
    Validate { path: Option<PathBuf>, #[arg(long)] level: Option<u8>, #[arg(long)] json: bool, #[arg(long)] repo_root: Option<PathBuf> },
    /// Rebuild the search index from scratch
    Index { path: Option<PathBuf> },
}
pub fn run_mcp(cmd: &McpCommand, root: &Path) -> superdev_core::error::Result<u8>;
pub fn run_aokf(cmd: &AokfCommand, root: &Path) -> superdev_core::error::Result<u8>;
```

  Rules: `path` defaults to `<root>/knowledge`; `repo_root` defaults to `root`; `level` defaults to 2. `Validate` prints `render_human()` (or the JSON) via the `out()` pattern and returns `0`/`1` from `Report::passed()`. `Index` builds the embedder from the manifest when `.superdev/config.toml` exists (absent → local default), forces a rebuild, prints stats. `Mcp Aokf` builds a small tokio current-thread runtime, constructs `AokfServer`, `serve_stdio`; startup failure (missing bundle dir, unwritable index dir) is an `Err` → exit 2; clean stdin close → 0. `main()`'s `run()` match gains the two arms; keep the existing BrokenPipe rule.

- [ ] **Step 1: Write the failing e2e tests** — extend `crates/app/superdev/tests/cli.rs` (these need no PATH fakes and run on all platforms):

```rust
#[test]
fn aokf_validate_passes_the_live_bundle() {
    Command::cargo_bin("superdev").unwrap()
        // workspace root, so `aokf validate` resolves <cwd>/knowledge
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.."))
        .args(["aokf", "validate"])
        .assert().success();
}

#[test]
fn aokf_validate_fails_a_broken_bundle_with_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("kb")).unwrap();
    std::fs::write(dir.path().join("kb/a.md"), "---\ntype: T\nid: dup\n---\nx\n").unwrap();
    std::fs::write(dir.path().join("kb/b.md"), "---\ntype: T\nid: dup\n---\nx\n").unwrap();
    Command::cargo_bin("superdev").unwrap()
        .current_dir(dir.path()).args(["aokf", "validate", "kb"]).assert().code(1);
}

#[test]
fn aokf_validate_json_is_machine_readable() { /* --json parses as serde_json::Value with a findings array */ }

#[test]
fn mcp_server_initialises_over_stdio() {
    // Spawn `superdev mcp aokf` in a fixture repo dir with a tiny knowledge/,
    // write an MCP initialize request + initialized notification + one
    // aokf_overview call as raw JSON-RPC lines to stdin, close stdin,
    // assert exit 0 and stdout contains "\"result\"" and the bundle name.
    // (Raw JSON-RPC over std::process — no client library needed for a smoke.)
}
```

`tempfile` is already a dev-dep of the binary crate.

- [ ] **Step 2: Run to verify failure**, implement `aokf_cli.rs` + wiring.

- [ ] **Step 3: Full workspace gate** — nextest, doctests, clippy, fmt. The existing manage e2e must stay green.

- [ ] **Step 4: Commit** — `git commit -m "feat(cli): aokf validate, aokf index and mcp aokf verbs"`

---

### Task 11: The validator swap

**Files:**
- Modify: `.claude/settings.json` (hook), `package.json` (`check:aokf`), `.github/workflows/checks.yml` (aokf step)
- Delete: `.agents/aokf/tools/validator.py`, `crates/lib/superdev-core/assets/agents/aokf/tools/validator.py`
- Modify: `.agents/VALIDATION.md`, `crates/lib/superdev-core/assets/agents/VALIDATION.md`, `crates/lib/superdev-core/src/components/aokf.rs` (FILES table drops the validator entry)

**Interfaces:**
- Consumes: the `aokf validate` verb (Task 10).
- Produces: one validator everywhere. The blueprint ships no Python.

- [ ] **Step 1: Switch this repo's three call sites**

- `package.json`: `"check:aokf": "cargo run --quiet -- aokf validate knowledge"`.
- `.claude/settings.json`: read the file first; replace the hook command that invokes `validator.py` with `cargo run --quiet -- aokf validate knowledge`, leaving everything else (matchers, other hooks) byte-identical.
- `checks.yml`: find the step running the Python validator (`npm run check:aokf` already wraps it — if CI only calls the npm script, no workflow change is needed; verify and say which in the report).

- [ ] **Step 2: Update the docs that name the command**

- `.agents/VALIDATION.md` (this repo's): command becomes `cargo run --quiet -- aokf validate knowledge`; the hook sentence stays true here.
- Asset `assets/agents/VALIDATION.md` (target repos): command becomes `superdev aokf validate knowledge`; keep the "no hook is installed yet — run it yourself" framing.
- `components/aokf.rs`: remove the validator.py FILES entry; adjust the aokf provider unit tests that count/list owned files.

- [ ] **Step 3: Delete both validator.py copies**

`git rm .agents/aokf/tools/validator.py crates/lib/superdev-core/assets/agents/aokf/tools/validator.py`. The parity goldens (Task 5) keep the Python behaviour as the reference — they are now the only trace, which is the point.

- [ ] **Step 4: Verify the swap end to end**

`npm run check:aokf` → PASS at level 2, exit 0. Temporarily duplicate an id in a scratch copy? No — trust the e2e from Task 10 for failure paths; here assert the live wiring: edit nothing, run the hook command exactly as settings.json has it, confirm exit 0. Then full workspace gate (nextest, clippy, fmt, coverage, check:aokf).

- [ ] **Step 5: Commit** — `git commit -m "feat: replace the python validator with superdev aokf validate"`

---

### Task 12: `.mcp.json` registration via the aokf provider

**Files:**
- Modify: `crates/lib/superdev-core/src/components/aokf.rs`
- Modify: `crates/lib/superdev-core/src/action.rs` (new action), `crates/lib/superdev-core/src/engine.rs` (execute it)
- Create: `.mcp.json` (this repo's own, committed)

**Interfaces:**
- Produces: a new `Action` variant, mirroring the `.mise.toml` shared-file rule for JSON:

```rust
    /// Set one top-level key path in a JSON file, preserving all other content.
    /// Creates the file (`{}`-rooted) when absent. Used for .mcp.json.
    SetJsonKey {
        /// Target path (repo-relative).
        path: String,
        /// Dotted key path, e.g. `mcpServers.superdev-aokf`.
        pointer: String,
        /// The value to set, as a JSON string.
        value_json: String,
    },
```

  `describe()`: `set {pointer} in {path}`. Engine execution: read (or `{}`), `serde_json::from_str` (parse failure → `Error::Toml`-style `Error::Manifest`? No — add the path to `Error::Toml`'s doc: it covers "a structured config file failed to parse"; use `Error::Toml { path, message }` for JSON too and note the naming in a comment), navigate/create the pointer path, set, write pretty with a trailing newline. Journal: `RestoreFile` with prior content (same as EnsureLine). Lock: hash under `files["<path>:<pointer>"]` (mirror `pin_lock_key`'s shape).

  The aokf provider plans, when the knowledge capability is enabled and `.mcp.json` lacks the key or differs:

```json
{ "command": "superdev", "args": ["mcp", "aokf"] }
```

  under pointer `mcpServers.superdev-aokf`. Drift compare: parse existing value, semantic JSON equality (not string equality).

- [ ] **Step 1: Write the failing tests**

```rust
// action.rs: describe test gains
// assert_eq!(Action::SetJsonKey { path: ".mcp.json".into(), pointer: "mcpServers.superdev-aokf".into(), value_json: "{}".into() }.describe(), "set mcpServers.superdev-aokf in .mcp.json");

// engine.rs tests:
#[test]
fn set_json_key_merges_and_preserves_other_servers() {
    // existing .mcp.json with another server; apply SetJsonKey; assert both present,
    // and lock.files contains ".mcp.json:mcpServers.superdev-aokf"
}
#[test]
fn set_json_key_creates_the_file_when_absent() { /* apply → file exists, parses, key set */ }
#[test]
fn malformed_mcp_json_fails_cleanly() { /* file "not json" → Failed outcome naming .mcp.json, unwind restores */ }

// components/aokf.rs tests:
#[test]
fn plans_mcp_registration_when_missing_and_not_when_present() {
    // fresh plan includes the SetJsonKey action; after applying equivalent JSON
    // (different formatting/key order), re-plan is empty for it
}
```

- [ ] **Step 2: Run to verify failure**, implement all three sites.

- [ ] **Step 3: This repo's own `.mcp.json`** — create by hand (committed):

```json
{
  "mcpServers": {
    "superdev-aokf": {
      "command": "cargo",
      "args": ["run", "--quiet", "--", "mcp", "aokf"]
    }
  }
}
```

(`cargo run` form deliberately differs from the blueprint's installed-binary form — this repo IS superdev; note it in the file? `.mcp.json` takes no comments — note it in `knowledge/development-procedure.md` in Task 14.)

- [ ] **Step 4: Run to verify pass** (workspace nextest — the manage e2e asserts init's plan; check whether `init_sets_up_a_fresh_repo` needs the new action in its expectations and update it), clippy, fmt.

- [ ] **Step 5: Commit** — `git commit -m "feat(core): register the aokf mcp server in .mcp.json"`

---

### Task 13: The AGENTS.md switchover

**Files:**
- Modify: `AGENTS.md` (this repo's)
- Modify: `crates/lib/superdev-core/assets/AGENTS.md` (blueprint template)
- Modify: `crates/lib/superdev-core/src/components/aokf.rs` tests if any assert template content

**Interfaces:** none new — content change.

- [ ] **Step 1: Rewrite this repo's AGENTS.md**

Keep: the AOKF spec reference block, the Agent Rules block (all four `@.agents/*.md`), the canonical-knowledge sentence with `@knowledge/index.md`. Delete: the entire "Core Concepts" `@`-list and its instruction paragraph. Add, after the knowledge block:

```markdown
### Working with the knowledgebase

The bundle is served over MCP (`superdev-aokf`). Orient with `aokf_overview`
and `aokf_graph`; search with `aokf_search` before assuming an answer; use
`aokf_read` before editing a concept; run `npm run check:aokf` after edits.
The index above is the map — the tools are how you read the territory.
```

- [ ] **Step 2: Mirror in the blueprint template** (`assets/AGENTS.md`): same structure, with the same wording (target repos get the identical instructions — the server name and tools are the same there).

- [ ] **Step 3: Live verification** — this is the dogfood gate, run it honestly:

Restart-free check: with `.mcp.json` in place from Task 12, run `claude mcp list` in this repo (the server should appear); then in a scratch session or via a direct JSON-RPC exercise (`printf` the initialize + `aokf_search` for "release procedure" into `cargo run --quiet -- mcp aokf`), confirm the top hit is `release-procedure.md` with a sane line range. Record the transcript in the report. If search cannot answer "how do releases work" convincingly, STOP — the spec's success criterion fails and the switchover must not land; report back instead of landing a broken dogfood.

- [ ] **Step 4: Validator + gate** — `npm run check:aokf` (index.md unchanged, so no validator impact expected), full workspace tests, clippy, fmt.

- [ ] **Step 5: Commit** — `git commit -m "feat: switch agents.md to search-first knowledge access"`

---

### Task 14: Documentation, knowledge, and landing

**Files:**
- Modify: `knowledge/architecture.md` (the aokf subsystem joins the layer description), `knowledge/software-components.md` (new modules, verbs, the MCP server), `knowledge/api-contracts.md` (three new verbs + tool surface summary + exit codes), `knowledge/error-handling.md` (validate's 0/1/2; MCP-tools-never-exit rule), `knowledge/configuration.md` (`[knowledge].embeddings`, the model cache path, `.mcp.json`), `knowledge/technology-stack.md` (six new deps with one-line reasons; the model pin), `knowledge/testing-strategy.md` (parity goldens, in-process MCP tests), `knowledge/development-procedure.md` (this repo's `cargo run` forms for hook/.mcp.json), `knowledge/glossary.md` (section, locator, hybrid search, RRF — two lines each), `knowledge/development-commands.md` (check:aokf's new form, `aokf index`)
- Modify: `README.md` (verb list gains the three), `CHANGELOG.md` (Unreleased: the server, the verbs, the validator swap, the switchover)
- Modify: `scripts/manage-smoke.sh` — after the existing status check: `superdev aokf validate knowledge`... the scratch repo's bundle: add `"${OLDPWD}/target/release/superdev" aokf validate knowledge` and an `aokf index` run (real model download exercised here, and only here)
- Modify: `knowledge/specs/2026-08-11-aokf-mcp-server-design.md` — `status: draft` → `stable`
- Delete: `knowledge/plans/2026-08-11-aokf-mcp-server.md` (this plan) via `git rm`

- [ ] **Step 1: Make every edit.** PROSE.md rules; keep frontmatter links mirrored; ids untouched.
- [ ] **Step 2: Validate** — `npm run check:aokf` → PASS level 2, 0 warnings (the plan's `implements` warning disappears with the deletion).
- [ ] **Step 3: Full pre-PR gate** — fmt-check, clippy `--all-targets -- -D warnings`, nextest workspace, doctests, rustdoc `-D warnings`, `npm run test:launcher`, `npm run verify-version`, `npm run check:aokf`, `npm run coverage:check`.
- [ ] **Step 4: Commit** —

```bash
git add -A && git rm knowledge/plans/2026-08-11-aokf-mcp-server.md
git commit -m "docs: record the aokf mcp server design as landed"
```

---

## Deviations

Any deviation from this plan during implementation (an API that doesn't exist as written — rmcp's macro surface is the likeliest — a test that can't pass as specified, a better structure) gets raised before proceeding, not silently absorbed — per `.agents/CODING.md` rule 1.
