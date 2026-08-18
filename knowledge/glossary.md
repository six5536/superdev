---
type: Glossary
id: glossary
title: Domain Glossary
description: The terms the blueprint engine uses — blueprint, capability, provider, provenance, component, skill override, owned file, scaffold, project template, skill pack, materialised skill, PROJECT.md layer, custom skill, harvest, claim, orphan — plus the search terms section, locator, hybrid search and RRF.
status: stable
---

- **Blueprint** — superdev's opinion of a managed repo, compiled into the
  binary: the component set plus the registry of default versions tested
  together. The binary's version is the blueprint version.
- **Capability** — a functionality slot in a managed repo: `knowledge`,
  `code-index`, `workflows`, `frontend`, `skills`. Capability names are what
  users type; see [architectural-rules](architectural-rules.md).
- **Provider** — the tool that fills a capability, e.g. `codegraph` for
  `code-index`. The registry carries one entry per (capability, provider) pair
  and flags one as the default; the manifest's `provider` field selects among
  them, so a capability's implementation changes without its user-facing name
  changing. `workflows` is the only slot with more than one entry.
- **Provenance** — why a pinned version is locked to the registry default:
  the binary carries either the checksum of the fetched artefact or the
  embedded content itself, so a version the binary cannot vouch for is
  refused.
- **Component** — the code implementing one provider. It observes the repo,
  compares against the manifest, and returns actions; it never applies them.
- **Skill override** — an embedded replacement for one of a workflows
  provider's skills, carried by that provider's component and materialised
  in place of the upstream version. It exists only where that provider is
  installed, and the skill's `custom` entry releases override and upstream
  alike.
- **Owned file** — a file superdev writes and keeps current, hashed into
  `lock.toml`. `sync` rewrites it, backing up and reporting any user edit.
  The embedded AOKF spec and validator are owned.
- **Scaffold** — a file superdev writes once and never touches again, such as
  `AGENTS.md`. It is the user's from the moment it exists, so it cannot drift.
- **Project template** — a set of write-once scaffolds embedded in the
  binary that `init` seeds a repo from, token-substituted and disjoint from
  every capability's files. `config.toml` records the name and token values
  as provenance; `sync` never revisits the files. `rust-npm` is the first —
  see the [spec](specs/2026-08-18-project-templates-design.md).
- **Skill pack** — the three generic skills the `skills` capability ships as
  owned files under `.claude/skills/`, embedded in the binary and versioned
  with it. Claude Code loads them from there natively, so there is nothing to
  install. The knowledge-lifecycle skills are not pack skills: the aokf
  component carries them.
- **Materialised skill** — a skill copied out of a pinned provider checkout
  into `.claude/skills/<name>/` as owned files, with the lock recording which
  capability put it there. The names come from upstream rather than the binary,
  which is what the attribution is for.
- **PROJECT.md layer** — a `PROJECT.md` beside a shipped skill. Every SKILL.md
  ends with a trailer telling the agent to read it and let it win on conflict,
  so a project extends a stock skill without forking it. superdev never writes
  or tracks the file.
- **Custom skill** — a skill named in a capability's `custom` list
  (`[skills]` or `[workflows]`) and thereby released from management:
  superdev stops writing it, drops its hashes from the lock, and `status`
  reports it as unmanaged rather than drifted. A name the capability no
  longer ships reports as having no effect instead of failing.
- **Harvest** — the move `aokf-adopt` performs: relocate a durable fact from
  stranded prose (or an opted-in code comment) into the bundle, leaving a
  one-line summary and a link behind in the source. See the
  [spec](specs/2026-08-18-knowledge-owned-skills-design.md).
- **Claim** — a typed lock entry a component declares it owns: a file, a
  `.mise.toml` pin, or a managed JSON key. The orphan pass subtracts the live
  claims from the lock, which is how a migration is derived.
- **Orphan** — a lock entry no live claim covers. `sync` removes it when its
  content still hashes to the locked value, and otherwise releases it: left in
  place, dropped from the lock, reported once.

Terms from the knowledge-serving side:

- **Section** — the unit of retrieval: one heading's body, or the root section
  (frontmatter plus anything before the first heading). A concept is indexed,
  searched and returned section by section, never whole.
- **Locator** — what a hit carries so it can be read next: bundle-relative
  path, concept id, heading path, line range, snippet, score.
- **Hybrid search** — running the lexical index and the vector index over the
  same sections and merging the two rankings. Exact terms are found by BM25,
  paraphrases by the embeddings; neither alone covers both.
- **Reciprocal rank fusion (RRF)** — how those two rankings merge: each list
  contributes `1/(60 + rank)` per section and the sums are sorted. It needs no
  score calibration between BM25 and cosine, which is the whole reason for it.

The files these terms describe are in [configuration](configuration.md); the
layering is in [architecture](architecture.md).
