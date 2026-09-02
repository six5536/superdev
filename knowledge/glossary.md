---
type: Glossary
id: glossary
title: Domain Glossary
description: The terms the blueprint engine uses — blueprint, capability, provider, provenance, component, owned file, scaffold, project template, template adoption, skill pack, knowledge-carried skill, content pack, pack source, embedded snapshot, pack item, pack layer, pack format, PROJECT.md layer, custom skill, harvest, claim, orphan — plus the knowledge terms section, locator, hybrid search, RRF, lifecycle and variant, and the contract terms binding, drift test and EARS.
status: stable
---

- **Blueprint** — superdev's opinion of a managed repo, compiled into the
  binary: the component set plus the registry of default versions tested
  together. The binary's version is the blueprint version.
- **Capability** — a functionality slot in a managed repo: `code-index`,
  `frontend`, `skills`. The SOKF knowledge is not one: it is part of
  superdev. Capability names are what users type; see
  [architectural-rules][sokf:architectural-rules].
- **Cardinality** — how many providers a capability holds at once, declared
  in the blueprint: *single* (one provider, exclusively — alternatives
  compete for the slot) or *many* (a set of providers, additively). Skills
  is the many slot; a manifest cannot turn an exclusive slot plural.
- **Provider** — the tool that fills a capability, e.g. `codegraph` for
  `code-index`. The registry carries one entry per (capability, provider) pair
  and flags one as the default; the manifest selects among them — one
  `provider` field for a single slot, one entry per enabled pack for a many
  slot — so a capability's implementation changes without its user-facing
  name changing. Every slot currently has exactly one registry entry.
- **Provenance** — why a pinned version is locked to the registry default:
  the binary carries either the checksum of the fetched artefact or the
  embedded content itself, so a version the binary cannot vouch for is
  refused.
- **Component** — the code implementing one provider. It observes the repo,
  compares against the manifest, and returns actions; it never applies them.
- **Owned file** — a file superdev writes and keeps current, hashed into
  `lock.toml`. `sync` rewrites it, backing up and reporting any user edit.
  The embedded SOKF spec and validator are owned.
- **Scaffold** — a file superdev writes once and never touches again, such as
  the starter knowledge concepts. It is the user's from the moment it
  exists, so it cannot drift.
- **Project template** — a set of write-once scaffolds embedded in the
  binary that `init` seeds a repo from, token-substituted and disjoint from
  every capability's files. `config.toml` records the name, token values and
  template version as provenance; the engine never revisits the files —
  updates happen only through the `template-update` skill, as user edits.
  `rust-npm` is the first — see the
  [template contract][sokf:contract-008-format-template].
- **Template adoption** — taking a project template into a repo that was
  never seeded from one: the `template-update` skill's first run there,
  merging the rendered template into the existing shape with the user
  deciding each collision, then recording `[template]` after the fact. From
  then on the repo updates like a seeded one.
- **Skill pack** — the two skills the `skills` capability ships as
  owned files under `.claude/skills/`, embedded in the binary and versioned
  with it. Claude Code loads them from there natively, so there is nothing to
  install. The knowledge-carried skills are not pack skills: the SOKF
  component carries them.
- **Knowledge-carried skill** — one of the 17 SOKF-carried skills the
  `knowledge` capability materialises into `.claude/skills/<name>/` as owned
  files, each skill its whole directory: SKILL.md, companions, harness
  configs. The set exists exactly where knowledge exists.
- **Content pack** — a versioned set of superdev's prose content: skills,
  document templates, project templates, knowledge skeletons and the
  general-rules scaffolds. A pack is resolved from a pinned source and
  materialised into the repo as ordinary owned files and scaffolds; it
  declares no executable action, so post-install work it needs is a skill
  it ships rather than a command it runs.
- **Pack source** — where a pack is resolved from: a git URL with a rev, or
  a local filesystem path. Sources are compared by a normalised identity —
  scheme, userinfo, port, `.git` suffix and trailing slash removed, host and
  path lowercased — so every spelling of one repository is one source
  ([ADR-004][sokf:adr-004-base-pack-identity]). Naming one in `config.toml` is the trust
  decision, exactly as adding a crate to `Cargo.toml` is. superdev
  guarantees only that the pinned bytes are the bytes applied; it makes no
  claim about the content.
- **Embedded snapshot** — the first-party pack compiled into the binary at
  the blueprint's default pin. It is a convenience copy so the default path
  needs no network, not an independent content set: pinning the default rev
  and fetching it yield the same bytes.
- **Pack item** — the unit a later layer supersedes: a whole skill directory,
  one project template, one document template, one knowledge skeleton or one
  general-rules scaffold. Its identity is (owning capability, kind, name), all
  three read from where it sits in the pack tree rather than from a list in
  `pack.toml` — `knowledge/skills/<name>/`, `knowledge/concepts/<name>`,
  `knowledge/templates/<name>.md`, `skills/<name>/`, `agents/<name>.md`,
  `projects/<name>/`. Superseding replaces a whole item, never part of one,
  and only ever within the same owner. See
  [ADR-003][sokf:adr-003-items-by-layout].
- **Pack layer** — the precedence order among content sources. Layer 0 is the
  embedded snapshot; a pack from the snapshot's own source replaces it
  outright, so what that pack drops leaves the repo, while a pack from any
  other source sits above it in manifest order and supersedes items by name.
  Superseding layer 0 is the normal case and passes unreported; only
  pack-over-pack shadowing is reported.
- **Pack format** — the version a pack manifest declares. A binary refuses a
  format it does not know, with a guided error. The format is not stable
  before 1.0.
- **PROJECT.md layer** — a `PROJECT.md` beside a shipped skill. Every SKILL.md
  ends with a trailer telling the agent to read it and let it win on conflict,
  so a project extends a stock skill without forking it. superdev never writes
  or tracks the file.
- **Custom skill** — a skill named in a capability's `custom` list
  (`[skills]` or `[knowledge]`) and thereby released from management:
  superdev stops writing it — for a knowledge skill, its whole directory —
  drops its hashes from the lock, and `status` reports it as unmanaged rather
  than drifted. A name the capability no longer ships reports as having no
  effect instead of failing.
- **Harvest** — the move `bootstrap` performs: relocate a durable fact from
  stranded prose (or an opted-in code comment) into the canonical knowledge, leaving a
  one-line summary and a link behind in the source.
- **Claim** — a typed lock entry a component declares it owns: a file, a
  `.mise.toml` pin, or a managed JSON key. The orphan pass subtracts the live
  claims from the lock, which is how a migration is derived.
- **Orphan** — a lock entry no live claim covers. `sync` removes it when its
  content still hashes to the locked value, and otherwise releases it: left in
  place, dropped from the lock, reported once.
- **Run** — one unattended pass over a feature plan: armed by
  `.superdev/cache/run.toml`, written only by the `superdev run`
  verbs, owned by one session, and enforced by the `superdev hook run`
  Stop hook. A watchdog bounds it — ten turn boundaries without an
  `advance` and the run dies — and a blocked run ends, leaving its
  questions in the plan's deferred decisions.

Terms from the knowledge-serving side:

- **Section** — the unit of retrieval: one heading's body, or the root section
  (frontmatter plus anything before the first heading). A concept is indexed,
  searched and returned section by section, never whole.
- **Locator** — what a hit carries so it can be read next: knowledge-relative
  path, concept id, heading path, line range, snippet, score.
- **Hybrid search** — running the lexical index and the vector index over the
  same sections and merging the two rankings. Exact terms are found by BM25,
  paraphrases by the embeddings; neither alone covers both.
- **Reciprocal rank fusion (RRF)** — how those two rankings merge: each list
  contributes `1/(60 + rank)` per section and the sums are sorted. It needs no
  score calibration between BM25 and cosine, which is the whole reason for it.
- **Lifecycle** — the one field that says whether a document is live or
  settled, on every issue, plan, decision and contract. Its value names
  the folder the document sits in — `knowledge/issues/open/`,
  `knowledge/plans/done/` — and `superdev validate --fix` moves a document
  whose folder disagrees. SOKF `status` no longer appears on these kinds: it
  answered the same question in a second vocabulary, and an absent `status`
  reads as `stable` by the SOKF spec, so dropping it changed nothing.
- **Variant** — one of the values a schema's `variant-key` frontmatter key
  admits, selecting which of the schema's rules a document is checked
  against: a rule tagged `variants` binds the values it names, an untagged
  rule binds every value, and the schema carries one example per value
  ([ADR-045][sokf:adr-045-a-schema-declares-variants]).

Terms from the contract side:

- **Binding** — what holds a contract to its implementation, in two
  halves
  ([ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]).
  A Definition is bound by materialisation: `superdev validate --fix`
  writes the included source region into the contract, and `superdev
  validate` fails on a block that is stale, empty or absent — superdev
  supplies this half. A Behaviour or Stability promise is bound by a
  test of the behaviour it promises, which the project writes in its own
  language and test runner. A generated rendering a contract includes
  binds only while a test proves it current.
- **Drift test** — a test that compares a hand-written copy of a
  definition to the code. There is none: a Definition is a source
  include the validator keeps current, so no copy exists to compare,
  and a test that opens a fenced block out of a contract to compare it
  to the binary is a finding
  ([I049][sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]
  criterion 23). A test of a Behaviour promise is a behaviour test, not
  a drift test.
- **EARS** — the Easy Approach to Requirements Syntax: a requirement
  opens with one of six pattern tags — `[ubiquitous]`, `[event]`,
  `[state]`, `[conditional]`, `[optional]`, `[complex]` — and states its
  trigger or condition in that pattern's words before one modal verb
  and one requirement. A feature-request's acceptance criteria take the
  form, numbered, with "THE SYSTEM" as the subject
  ([ADR-031][sokf:adr-031-ears-criteria-are-checked-by-item-pattern]);
  a contract's Behaviour and Stability promises do not yet
  ([I037][sokf:issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears]).

The files these terms describe are in [configuration][sokf:configuration]; the
layering is in [architecture][sokf:architecture].

<!-- sokf:links -->
[sokf:adr-003-items-by-layout]: /knowledge/adrs/active/adr-003-items-by-layout.md
[sokf:adr-004-base-pack-identity]: /knowledge/adrs/active/adr-004-base-pack-identity.md
[sokf:adr-031-ears-criteria-are-checked-by-item-pattern]: /knowledge/adrs/active/adr-031-ears-criteria-are-checked-by-item-pattern.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-045-a-schema-declares-variants]: /knowledge/adrs/active/adr-045-a-schema-declares-variants.md
[sokf:architectural-rules]: /knowledge/architectural-rules.md
[sokf:architecture]: /knowledge/architecture.md
[sokf:configuration]: /knowledge/configuration.md
[sokf:contract-008-format-template]: /knowledge/contracts/public/active/contract-008-format-template.md
[sokf:issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears]: /knowledge/issues/open/issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears.md
[sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]: /knowledge/issues/open/issue-049-feature-request-a-contract-cannot-point-at-its-definition.md
