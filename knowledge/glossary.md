---
type: Glossary
id: glossary
title: Domain Glossary
description: The terms the blueprint engine uses — blueprint, capability, provider, component, owned file, scaffold, skill pack, PROJECT.md layer, custom skill — plus the search terms section, locator, hybrid search and RRF.
status: stable
---

- **Blueprint** — superdev's opinion of a managed repo, compiled into the
  binary: the component set plus the registry of default versions tested
  together. The binary's version is the blueprint version.
- **Capability** — a functionality slot in a managed repo: `knowledge`,
  `code-index`, `workflows`, `frontend`, `skills`. Capability names are what
  users type; see [architectural-rules](architectural-rules.md).
- **Provider** — the tool that fills a capability, e.g. `codegraph` for
  `code-index`. Swappable without changing the user-facing surface.
- **Component** — the code implementing one provider. It observes the repo,
  compares against the manifest, and returns actions; it never applies them.
- **Owned file** — a file superdev writes and keeps current, hashed into
  `lock.toml`. `sync` rewrites it, backing up and reporting any user edit.
  The embedded AOKF spec and validator are owned.
- **Scaffold** — a file superdev writes once and never touches again, such as
  `AGENTS.md`. It is the user's from the moment it exists, so it cannot drift.
- **Skill pack** — the five skills the `skills` capability ships as owned files
  under `.claude/skills/`, embedded in the binary and versioned with it. Claude
  Code loads them from there natively, so there is nothing to install.
- **PROJECT.md layer** — a `PROJECT.md` beside a shipped skill. Every SKILL.md
  ends with a trailer telling the agent to read it and let it win on conflict,
  so a project extends a stock skill without forking it. superdev never writes
  or tracks the file.
- **Custom skill** — a skill named in `[skills] custom` and thereby released
  from management: superdev stops writing it, drops its hash from the lock, and
  `status` reports it as unmanaged rather than drifted.

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
