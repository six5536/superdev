---
type: Spec
id: spec-aokf-mcp-server
title: AOKF MCP Server
description: Design for the read-side AOKF MCP server — hybrid search, graph, the Rust validator, and the search-first AGENTS.md switchover.
status: stable
links:
  - rel: relates-to
    to: spec-cli-core-blueprint-engine
    note: Sub-project 2; builds on the blueprint engine and adjusts its update(!) promise.
  - rel: relates-to
    to: architecture
    note: Adds the knowledge-serving layer to the product architecture.
---

# Summary

Sub-project 2 of superdev (see the
[CLI core spec](S001-cli-core-blueprint-engine-design.md) for the
product frame). `superdev mcp aokf` serves the canonical project knowledge to agents over
MCP, so querying replaces the wholesale `@knowledge/…` preloading in
AGENTS.md. The AOKF format is defined in `/.agents/aokf/SPEC.md`.

**Scope adjustment against sub-project 1's spec**: structured update through
MCP is postponed to a later phase. This sub-project delivers the read side —
search, read, graph, overview — plus the Rust validator
(`superdev aokf validate`) replacing the Python one, and the AGENTS.md
switchover itself. Write-class enforcement remains the validator + diff
check until the update phase.

Decisions bound earlier and honoured here: hybrid search (lexical +
embeddings); local embedding model by default, API opt-in; the switchover
happens in this sub-project, dogfooded on this repo.

# Behaviour

- Stated in the design sections below rather than gathered here. This spec
  predates the contract that asks for one Behaviour section, and the
  sections were left where they were written rather than reshuffled after
  the fact.

# Acceptance criteria

A fresh agent session in this repo, with no per-concept preloads, can
answer "how do releases work"-class questions by orienting and searching
through the four tools, landing on the right concept sections with correct
line ranges. `superdev aokf validate` matches the Python validator on the
fixture matrix and gates the hook, npm script, and CI; `validator.py` no
longer exists in the repo or the blueprint. Search still works offline
(lexical-only) with the model absent. All existing CI gates stay green.

# Edge cases & errors

- Tool failures are MCP error payloads, never process exits: unknown id →
  near-miss candidates; parse-broken file → the validator finding for it;
  model unavailable → lexical-only warning.
- Knowledge failing validation still indexes and serves — agents need
  search most while fixing a broken knowledge; `aokf_overview` carries the
  warning block.
- `mcp aokf` exits 0 on clean stdin close, 2 on startup failure. The
  BrokenPipe-is-success rule stays.

# Architecture

All logic in `superdev-core`, new `aokf` subsystem; the binary stays thin
(see [architecture](../architecture.md)).

- **`aokf` (format)** — frontmatter parsing into a `Concept` model, knowledge
  loading (reserved-file rules), body splitting at markdown headings into
  sections with line ranges, and the link graph (id/path resolution,
  inverse-edge synthesis per SPEC §8). One parser feeds the validator and
  the index.
- **`aokf::validate`** — document check + conformance ladder on that model.
- **`aokf::index`** — tantivy lexical index plus a flat vector store, one
  document and one embedding **per section** (frontmatter
  title/description/tags index as the concept's root section). Storage in
  `.superdev/cache/aokf-index/` with a manifest of per-file content hashes,
  index schema version, and embedding model id. Brute-force cosine; no ANN
  structure at this scale.
- **`aokf::mcp`** — the rmcp stdio server exposing the four tools below.

New binary verbs: `superdev mcp aokf` (serve), `superdev aokf validate`
(exit codes 0 pass / 1 findings / 2 usage — the Python codes, which already
match the CLI contract), `superdev aokf index` (forced full rebuild;
reindexing is otherwise lazy).

New dependencies (versions pinned at plan time): `rmcp`, `tantivy`,
`model2vec-rs`, a YAML frontmatter parser, `pulldown-cmark`. All pure Rust;
the static-musl release builds are non-negotiable and ruled out
ONNX-runtime alternatives.

# MCP tool surface

Four tools, stdio only, no resources/prompts. Every hit carries the locator
set: knowledge-relative path, concept id, heading path, line range, snippet,
score.

- **`aokf_search`** — `query`, optional `limit`, `types`, `tags`. BM25 +
  cosine over sections, fused with reciprocal-rank fusion (k=60), filters
  applied pre-fusion. Results grouped by concept.
- **`aokf_read`** — `id` (or path), optional `heading`: whole concept or
  one section, frontmatter summarised, line numbers on every block.
- **`aokf_graph`** — two modes. No args: the knowledge-wide edge map, one line
  per **declared** edge (source, rel, target, note), grouped by source;
  synthesised inverses are omitted as derivable. With `id`: the single-hop
  neighbourhood, both directions, one line per edge with the neighbour's
  `description`. No depth parameter — multi-hop is the agent calling again,
  pruning between hops. Edge lists cap at ~30 per group with an explicit
  `+N more (rel: …)` truncation line.
- **`aokf_overview`** — no args: manifest name, concept count, the
  directory tree with each concept's id and description, and a warning
  block when the canonical knowledge currently fails validation.

# Index lifecycle

Freshness is lazy, checked on every tool call: knowledge file hashes compare
against the manifest (milliseconds); changed, new, or deleted files are
re-parsed, re-embedded, and updated incrementally. No watcher, no daemon
state — the server restarts freely and always sees edits on the next call.
Schema-version or model-id mismatch triggers a full rebuild.

The embedding model downloads once into a user-level cache
(`~/.cache/superdev/models/…`, not per-repo) and loads offline thereafter.
Model absent and undownloadable → search degrades to lexical-only with a
warning in the response, never an error.

# Embeddings and configuration

- Default: `model2vec-rs` with a pinned potion retrieval model (exact model
  id and revision fixed in the plan — pinned, not "latest").
- Opt-in API embeddings: the manifest's `[knowledge]` table gains an
  optional `embeddings` sub-table (`provider`, `model`; key via environment
  variable, never in the file). Same interface as local; the recorded model
  id makes a provider switch trigger the rebuild automatically.
- No learned ranking weights and no reranker; revisit only on demonstrated
  misses.

# Validator port and swap

Behavioural port of `/.agents/aokf/tools/validator.py`: same findings, same
level grading, same warning semantics, same `--json` output, same exit
codes. Parity is proven against a fixture matrix — this repo's live knowledge
plus synthetic knowledge trees per failure class (broken links, duplicate ids,
malformed frontmatter, stamped fields present, bad `verified` entries,
unmirrored links) — by comparing both implementations' JSON output.

Then the Python validator is fully replaced: the `.claude/settings.json`
hook, `npm run check:aokf`, and CI switch to the Rust binary;
`validator.py` is deleted from `.agents/aokf/tools/` and from the blueprint
assets, and the VALIDATION.md asset names the new command. In this repo the
hook and npm script run `cargo run --quiet -- aokf validate knowledge`
(compile-cached); target repos use their installed binary.

# Registration and the AGENTS.md switchover

- `.mcp.json` at the target-repo root gains the server entry, written by
  the aokf provider as a shared file: targeted JSON merge preserving other
  servers, only superdev's key managed and hashed — the `.mise.toml` rule
  applied to JSON. This repo gets the same entry pointing at `cargo run`.
- AGENTS.md (this repo's and the blueprint template) keeps the AOKF spec
  reference and the `.agents/*.md` rules, keeps `@knowledge/index.md` as
  the preloaded safety net, deletes the per-concept `@` list, and adds
  standing instructions: orient with `aokf_overview` and `aokf_graph`,
  search before assuming, `aokf_read` before editing a concept, validate
  after edits.

# Testing

- **Unit**: parser (write-class fields, section splitting with line
  ranges, link resolution), graph (inverse synthesis, truncation, edge
  map), RRF maths, staleness manifest. A fake embedder trait impl keeps
  vector tests deterministic.
- **Validator parity**: the fixture matrix, JSON-compared against the
  Python output (fixtures retain the Python results as golden files after
  the script is deleted).
- **MCP integration**: rmcp client over an in-process duplex stream against
  fixture knowledge trees — every tool, asserting locators, line numbers,
  truncation, and the lexical-only degradation.
- **E2e**: `aokf validate` exit codes in the existing assert_cmd harness;
  one server smoke (initialize + one search + clean shutdown on stdin
  close). Real-model runs live in the manual smoke script only.
- Coverage ≥90% per crate, unchanged.

# Out of scope

Update tools (postponed phase); file watchers; HTTP transport; reranking;
multi-knowledge serving; plugin-based distribution of the registration
(sub-project 3); knowledge upkeep verbs beyond `validate`/`index`
(sub-project 4).

# Test plan: aokf mcp server

## Scope

- The parser, the graph, the index and the four MCP tools.
- Out: everything the sections above place out of scope.

## Risks driving this plan

1. Recorded after the fact. This plan was written when the spec was
   conformed to its contract, not when the feature was built, so it names
   the risks the tests actually cover rather than the ones weighed at the
   time.

## Test cases

### Automated

| # | Case | Type | Inputs / setup | Expected result |
|---|------|------|----------------|-----------------|
| 1 | Parsing and the graph | unit | fixture concepts | write-class fields, section ranges, synthesised inverses |
| 2 | Ranking | unit | a fake embedder | deterministic hybrid results |
| 3 | The tools over a pipe | integration | a real rmcp client, transport stubbed | locators, line numbers, truncation, lexical-only degradation |

### Manual verification

1. None recorded. The feature shipped under the automated cases above; no
   manual step was written down at the time, and inventing one now would
   claim a check nobody made.

## Exit criteria

- The automated cases above pass.
- `superdev validate` reports no error for this document.
