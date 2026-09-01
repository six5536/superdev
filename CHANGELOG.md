# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
superdev uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While
superdev is pre-1.0, minor versions may contain breaking changes.

Every released tag needs its own section here. The release workflow refuses to
publish a version it cannot find a heading for.

## [Unreleased]

### Added

- **The file-format contract kind splits into text and binary.**
  `schema-contract-text-format` governs a file whose shape is keys and values
  and `schema-contract-binary-format` one whose shape is a byte layout —
  offsets, widths, endianness, a magic number and a version a reader checks
  first (ADR-037). **`schema-contract-file-format` is retired**: a contract
  of that type must change its frontmatter `type` to `TextFormatContract`
  and its id token from `file-format` to `text-format`, which
  `superdev validate --fix` then refiles. superdev's own contracts 005, 006
  and 008 moved with it.
- **A drift failure says which way the difference runs.** An element the
  implementation carries and its contract does not declare reports as a
  `DEFECT`; one the contract promises and the implementation has yet to keep
  reports as `PENDING` (ADR-038). A contract element may carry a `pending`
  marker naming the slice that will build it: its binding reverses, failing
  once the implementation has it, so the marker cannot outlive the work it
  names, and `accept` refuses a contract still carrying one. The feature plan
  now orders a slice that closes a contract-implementation gap before slices
  that do not, so a contract's promise does not fail the slices that do not
  own it.
- **The MCP contract defines its tools.** `contract-003` carries every tool's
  arguments — name, type, requiredness and meaning — and its result shape in
  a JSON block, where before it named four tools in prose and carried no
  schema at all. A test compares the block to the tools the server actually
  serves, in both directions, so an argument cannot be added, removed or
  retyped without its contract moving with it.
- **The CLI contract defines the command line.** `contract-002` carries every
  command, positional argument, flag and exit code superdev offers in a YAML
  block a caller can build from, and a test walks the command tree and
  compares it to that block element for element in both directions — so a
  flag cannot reach the binary without reaching its contract (ADR-036). A
  second test runs the binary and asserts each declared exit code. The
  contract had drifted: `status --drift`, `--help` and a shipped template
  were missing from it, and `--repo-root` printed a value name the contract
  never named.
- **A schema can bind the shape of a definition block.** A section rule
  declares `block-language` — `yaml` or `json` — plus `block-keys` and
  `block-entry-keys`, and `superdev validate` parses the section's fenced
  block and reports a missing key naming the file, the section, the entry
  and the key (ADR-035). A block that will not parse, one whose fence
  carries the wrong tag, and a section with no block at all each report
  what is wrong; a block in a language the validator does not read
  declares no `block-language`, and its drift test binds it instead.
- **An acceptance criterion's EARS tag is checked.** A feature-request
  criterion that does not open with `[ubiquitous]`, `[event]`, `[state]`,
  `[conditional]`, `[optional]`, `[complex]` or `TBD — ` fails
  `superdev validate`, naming the criterion (ADR-031). The shape framing
  has always asked for is now enforced where it is written.
- **A schema can bind the shape of a section's text.** A section rule
  declares `item-pattern`, which every top-level item of the section's list
  must match, and `content-pattern`, which the section's whole body must
  match; both are regexes matched found-anywhere, so a rule binds the ends
  by writing `^` and `$` (ADR-030). A pattern that does not compile, or an
  `item-pattern` beside a content kind with no items, is reported on the
  schema and binds nothing.
- **A knowledge document can include another's body.** A concept carries a
  `<!-- sokf:include <id> -->` … `<!-- /sokf:include -->` marker pair, and
  `superdev validate --fix` materializes the named concept's body between
  the markers, refreshing every copy when the source changes; bare
  `validate` reports a stale, empty or unresolvable include block as an
  error (ADR-027). Shared prose gets one authored home while every file
  stays self-contained on disk.
- **A schema's worked example is checked.** `superdev validate` reads each
  schema's `example:` block as a document and checks it against the schema
  that declares it — the frontmatter contract, the sections, their order
  and their content kinds — reporting every failure as an error on the
  schema file, prefixed `example:` so the reader sees the example broke
  rather than the schema's own shape (ADR-024). An example that does not
  parse as a document — no frontmatter block where the schema dispatches
  by frontmatter `type`, or frontmatter that is not YAML — is an error on
  the schema file too; a schema that names its documents by glob governs
  frontmatter-less documents, so its example owes no frontmatter block.
- **An example's links bind by form and never resolve.** Inside a
  schema's example, a concept link takes the `[text][sokf:<id>]`
  reference form: a link or link definition whose target is a path into
  the knowledge, and a `sokf:` destination written inline, are each an
  error on the schema file (ADR-025). No id or target is resolved — a
  fictional `sokf:` label passes, and a URL or a repository path outside
  the knowledge keeps its ordinary markdown form, as does an image. An
  example's `lifecycle` also binds here: the filing check, which owns
  the key for real documents, never reads an example.
- **Required frontmatter keys bind.** `superdev validate` reads the
  per-key `required: true` flag a schema declares (ADR-022) and reports,
  as an error naming the document, the key and the schema, an absent key
  marked required; a present one keeps its value checks. The shipped
  schemas each declare their required keys — `type`, `id`, `title` and
  `description` on every frontmatter-carrying kind, `sources` on
  research — so a filed document that loses its identity or its listing
  line fails validation instead of passing unexamined.

- **A section's declared content kind binds.** `superdev validate` reads
  each section rule's `content` kind — prose, bullet-list, numbered-list,
  table or code — and reports, as an error naming the document, the
  section and the schema, a matched section whose body lacks the kind's
  form: one bullet, one numbered item, one table, one fenced block, or one
  plain paragraph line. Presence is what binds, so a lead-in sentence
  before a list passes and content beside the form is tolerated; a
  subsection's content counts, and lines inside fenced blocks do not. A
  schema declaring a kind outside the five is reported on the schema file
  itself and binds nothing.
- **The frontmatter contract binds on present values.** `superdev validate`
  reads every frontmatter key's constraint block and reports, as an error
  naming the document, the key and the schema, a present value that breaks
  its `const`, `pattern` or `enum`. Constraints compare against the value's
  scalar string form, so a value with no scalar form — a list, a map, a
  folded block — cannot satisfy one. A key declared with only a
  `description` is guidance; an absent key is not reported. A `pattern`
  that does not compile is reported on the schema file itself and binds
  nothing.

### Fixed

- **A named document is checked as what it is.** `superdev validate
  <path>` runs the bare pipeline and scopes the report to what the path
  covers, so a named document — dispatched by its frontmatter `type` or
  a schema's `target-files` glob — gets exactly the bare run's schema,
  filing and link findings for that file and none about any other file
  (ADR-026, I019). The misreading goes: a document never takes the
  grammar's fallback kind. The fallback stays for a file no schema and
  no grammar kind claims, so a skill outside the roots is still checked
  as one. A fault reading the knowledge fails only a run whose paths
  touch it, `validate --fix <path-inside-the-knowledge>` repairs the
  knowledge the findings point at, and the bare and named runs read one
  schema set — the knowledge's own `schemas/` directory — so the two
  cannot disagree, `--knowledge <DIR>` included.
- **The document checks read YAML as YAML and say each fault once.** A
  schema contract that fails to deserialize is reported on the schema
  file instead of silently governing nothing. Frontmatter values compare
  with comments stripped and quotes removed, and a CRLF document reads
  as its LF twin. Fences follow one reading everywhere — nested and
  tilde fences included — for content kinds, headings and tables; a
  columns rule finds its table in a subsection; the prose kind ignores
  link definitions, HTML comments, deeper headings and dividers. A
  `lifecycle` key is skipped only where the filing check owns it, and an
  unreadable declaration earns one finding, through the grammar's schema
  check.
- **The core file is checked again.** The grammar's core kind matched the
  file by the basename `core.md`, which the aggregator rename removed, so
  `.agents/superdev.md` was claimed by no kind and skipped by every
  `superdev validate` run. The kind now matches `superdev.md`, and the
  `unit|core` duplication pair is gone: the core file is a generated
  aggregator that embeds the units' prose by design, so the pair could only
  ever fire on intended content. The skill bootstraps and the CLI contract
  that still named `.agents/core.md` now name `.agents/superdev.md`.

### Changed

- **A contract now defines its interface, not describes it.** Every
  contract-kind schema demands a definition block carrying the whole surface
  a caller depends on — commands and flags, tool schemas, settings, file
  shapes, exported signatures — in the form that kind's ecosystem already
  reads, and demands the project bind that block to its implementation
  (ADR-033, ADR-034, ADR-036). **`ADR-029` is superseded**, and its
  judgement-based standard with it. **After a pack update, a contract whose
  definition block is incomplete starts failing `superdev validate`**, and a
  contract with no binding — no generation from it, no test against it — is
  non-compliant even while it validates. superdev's own contracts were
  rewritten to the bar: the CLI contract had drifted from the binary, the MCP
  contract carried no tool schema at all, and the config contract never
  declared its `[template]` table.

- **A contract's promise sections now declare that they bind.** The fifteen
  contract-kind schemas carry the RFC 2119 keyword as an `item-pattern` on
  sections where every entry is a promise — cli Behaviour, data
  Constraints, deployment Health and lifecycle, events Ordering and
  delivery — and as a `content-pattern` on sections whose job is promises,
  Stability among them (ADR-032). Definitional sections declare nothing:
  they bind by form. **After a pack update, an existing contract whose
  promise sections state no requirement in modal terms starts failing
  `superdev validate`.** Give each flagged promise its modal verb; a
  section that describes rather than binds belongs in a definitional
  section.
- **The contracts read as binding surfaces.** Every contract-kind schema
  carries the contract style standard — one RFC 2119 sentence per
  requirement, structured forms for enumerable surfaces, reasoning in
  the linked ADRs — and the nine active contracts are swept to it, with
  the pack-layout tables brought current (schemas and fragments ship;
  the dropped templates kind is gone).
- **The schemas keep their own rules.** Every schema's worked example now
  satisfies its own frontmatter constraints (26 were stale). The seven
  report schemas — code review, security review, investigation,
  postmortem, status update, release notes, migration guide — gain an id
  pattern (`{kind}-{nnn}-{slug}`) and file under `knowledge/reports/`.
  Contracts, ADRs and ideas each take one shape: a title heading over
  level-2 sections; ADR state lives in `lifecycle`, a `supersedes` link
  and `status: draft` in place of the body Status bullet, and the six
  public contracts, all 21 live ADRs and idea-001 are restyled to match. The removed
  spec workflow's vocabulary is gone from the schemas, the contract
  number series says "public and internal", and the constraints schema's
  type is `ConstraintsNonGoals`.
- **Integrate reviews once per feature.** The `/code-review` in the
  integrate skill runs at the last slice only, over the whole feature diff
  against the merge target; findings still return to build before the
  merge. Non-final slices keep every other check — cases, done-check,
  contracts, build and tests.
- **Documents are filed by lifecycle.** Breaking change to the knowledge
  layout. Issues, plans, specs, decisions and contracts each carry one
  `lifecycle` frontmatter key — `open`/`done`/`wontfix` for issues,
  `open`/`done`/`abandoned` for plans, `active`/`deprecated` for specs,
  decisions and contracts — and every document sits in a folder named
  exactly its value (`knowledge/issues/open/`, `knowledge/plans/done/`),
  with each kind's base directory holding only `index.md`. The `status`
  key and the `done`/`wontfix`/`needs-triage` tags that answered the same
  question are gone from these kinds. Three new findings, all errors: a
  value outside the schema's enum, a folder disagreeing with the value,
  and an unfiled document in a base directory; `superdev validate --fix`
  repairs the filing by moving the file and regenerating the definition
  blocks that name it. `sokf_search` down-ranks any non-live `lifecycle`
  and takes a `lifecycle` filter, and `sokf_overview` and `sokf_read`
  report each concept's value.

- **SOKF 0.4: a body link addresses a concept by id.** Breaking change to the
  knowledge format. A link between concepts is now written as a
  reference-style link labelled `sokf:<id>` — `[config][sokf:config]` — and
  resolves through the id, so renaming or moving a document breaks nothing
  that cites it. Each document carries a generated `<!-- sokf:links -->` block
  at its foot giving every cited id its current repo-root path; the block is
  what makes the link navigate in a plain markdown renderer, and resolution
  never reads it. A link to anything that is not a concept stays a path.

  Five new findings, all errors: a path link to a concept, a `sokf:` label
  naming no concept, a missing definition, a stale one, and two ids sharing a
  kind and a number. `knowledge/manifest.sokf.yaml` declares `sokf: "0.4"`;
  a 0.3 knowledge whose body links name paths no longer conforms.

- **`superdev validate --fix`.** Converts a path link to the id form, reading
  the target document's own `id`, and regenerates every definition block. It
  writes only inside the SOKF knowledge, is idempotent, and is never run by
  `superdev hook validate` — the hook fires after an edit, so a hook that
  repaired would rewrite the file the agent is still working in. Run it before
  committing.

- **AOKF is SOKF, and it is part of superdev.** The format is renamed —
  Superdev Open Knowledge Format — and stops being a capability a provider
  fills. `Capability::Knowledge`, its registry entry and `--no-knowledge` are
  gone; every `init` writes the knowledge scaffold, the hook and the MCP
  registration. `[knowledge]` is now a plain top-level table in
  `.superdev/config.toml` holding `custom` and `embeddings`; a manifest still
  naming a `provider` there is refused with the edit to make.

  The standing term is **SOKF knowledge**, always in full. "Knowledge" alone
  is an ordinary English word — `frame/SKILL.md` used it in three senses in
  forty-three lines — and `sokf` is the token that means one thing only. It
  anchors every identifier:

  | Was | Is |
  |-----|----|
  | `superdev aokf validate` | `superdev validate` |
  | `superdev aokf index` | `superdev sokf index` |
  | `superdev aokf hook validate` | `superdev hook validate` |
  | `superdev mcp aokf` | `superdev mcp sokf` |
  | `aokf_search`, `aokf_read`, `aokf_graph`, `aokf_overview` | `sokf_search`, `sokf_read`, `sokf_graph`, `sokf_overview` |
  | `mcpServers.superdev-aokf` | `mcpServers.superdev-sokf` |
  | `knowledge/manifest.aokf.yaml`, key `aokf` | `knowledge/manifest.sokf.yaml`, key `sokf` |
  | `.agents/aokf.md`, `.agents/aokf/SPEC.md` | `.agents/sokf.md`, `.agents/sokf/SPEC.md` |
  | `.agents/format/grammar.yaml` | `.agents/sokf/grammar.yaml` |
  | `.superdev/cache/aokf-index` | `.superdev/cache/sokf-index` |
  | `validate --bundle <DIR>` | `validate --knowledge <DIR>` |
  | the `"bundle"` key in `--json` | the `"knowledge"` key |

  The MCP key and the hook marker are lock keys in every managed repo: one
  `sync` removes the old entries and writes the new ones, which is what the
  orphan pass is for. The old `aokf` verb group is gone outright, with no
  alias — pre-1.0, and there is nothing left for an alias to be compatible
  with. `.superdev/cache/aokf-index/` is not in the lock, so nothing removes
  it; delete it by hand if you mind.

  In `superdev-core`, `aokf` and `format` are replaced by `sokf` (the read
  side) and `validate` (both halves of the check, meeting in the parent
  rather than inside one of them). This breaks the published API.

  See P008.

### Added

- **`superdev run`.** Three verbs own the state of an unattended workflow
  run, `.superdev/cache/run.toml`: `begin` creates it exclusively and
  refuses a second run naming the owner, `advance` records a step forward —
  rewriting the next step, resetting the watchdog counter and refreshing
  the owning session — and `end` removes it, harmlessly when none exists.
  `superdev hook run` is the Claude Code Stop hook that reads it: while a
  run is armed for the payload's session it refuses to let the turn end,
  naming the next step, and it lets the run die after ten continues
  without an advance. Without a run state it is invisible, and it fails
  open on an unreadable one. The knowledge capability's sync writes the
  hook's `hooks.Stop` entry into `.claude/settings.json` beside the
  PostToolUse one, claimed in the lock the same way. See P013.

- **`/execute-feature-plan`.** A knowledge-carried skill drives
  feature-plan, build and integrate in a loop on the feature's branch:
  it cuts the plan when none exists, takes each slice whose dependencies
  are done through build and integrate in a subagent, retries a failing
  slice at most twice before deferring it, writes the questions only the
  user can answer into the plan's deferred decisions, and ends by putting
  them to the user in sequence. It drives `superdev run`, so the Stop
  hook enforces the loop where Claude Code is present. See P013.

- **The workflow branches and commits.** `/frame` cuts `feature/<slug>`
  off the default branch — a repo whose development-procedure concept
  names its own convention keeps it — and commits the framed issue there;
  `/adhoc-plan` cuts `adhoc/<slug>` when its work touches code.
  `/contract-design` ends on the user's go-ahead and commits the contract
  and decision-record edits; `/integrate` commits the changelog,
  knowledge and plan edits after a successful merge. Nothing unattended
  reaches the default branch; a human fast-forwards it (ADR-021). See
  P013.

- **A feature plan models its dependencies.** Each slice carries a
  `Depends-on:` line — slice numbers in either direction, or none — and
  the plan may carry a `Deferred decisions` section holding the questions
  an unattended run could not answer. The slice list is ordered so every
  slice follows its dependencies, a forward reference never renumbers the
  slices already written, and the feature-plan skill refuses a cycle. See
  P013.

- **`superdev validate`.** One command checks both specs this repository
  owns: the SOKF knowledge, and every document against the schema its `type`
  names — the skills and `.agents/core.md` included, against the grammar
  that defines the language they are written in. One report, findings
  grouped by file, one exit code, and one PostToolUse hook, so a file both
  checks have something to say about is reported once and the two cannot
  reach different verdicts. The grammar is read from
  `.agents/sokf/grammar.yaml`, or from the copy inside the binary when a
  repository has none, and `--doc` prints it as prose. See P006 and P008.

- **Documents are checked against their schemas.** Forty schemas declared
  `target-files`, a glob naming the documents each governs, and nothing had
  ever read it — so no document had ever been checked against the contract
  that claimed to govern it. Dispatch now runs through the frontmatter
  `type`, one type to one schema, and `validate` reports a missing required
  section, a section out of order, a prohibited section, a wrong table column
  and an over-limit line count. Switching it on found 218 disagreements
  across this repository, and every one was resolved by bringing the document
  to its schema — except where the documents were unanimous against it, which
  happened once: all nine settled issues put the resolution section directly
  under the title, where the schema had it last.

  The report says what it covered: `documents: N checked against M schemas`,
  and a run that found no schemas says so rather than passing in silence.
  That matters today, because the content pack ships the templates and not
  the schemas, so a managed repo has none and checks no document against any
  contract — see I020. See P008.

### Removed

- **The `bash-output-filter` capability and its rtk provider.** Bash output is
  no longer rewritten before it reaches the agent: the compaction was not
  worth what it cost the agent reading it. rtk was the slot's only provider
  and the slot existed for rtk, so both go, and superdev now manages three
  capabilities.

  A manifest still carrying `[bash-output-filter]` is refused at load with the
  edit to make. **Delete that table, then run `superdev sync`**: the orphan
  pass removes `.miserc.toml`, `mise.unix.toml`, `mise.windows-x64.toml`,
  `.agents/rtk.md` and the `mise exec http:rtk -- rtk hook claude`
  `PreToolUse` element, and prunes their lock entries. Any of the five you
  have edited is released from the lock and left on disk rather than deleted.
  `--no-bash-output-filter` is gone, and `superdev update
  bash-output-filter` is now an unknown capability.

  The mise version floor goes with it: `.miserc.toml`'s `auto_env` was the
  only thing requiring mise 2026.8 or newer, so a managed repository works on
  an older mise again. See P009.

- **The Node format validator.** `scripts/superdev-format/` and the
  meta-schema beside the grammar are gone: the checks are in the binary, the
  Rust types are the meta-schema, and nothing in the repository needs Node to
  validate the format. The reference's behaviour is held by goldens captured
  from it while it still ran.

- **The AOKF conformance ladder.** The knowledge now passes or fails; there
  is no level to grade against. `--level` is gone, and the report drops `checked_level`, `achieved_level` and each finding's
  `error_at_level`. No verdict changes: every implementation graded at the top
  level already, where a finding was an error exactly when it carried any level
  at all. What does change is that `--level 0` can no longer wave knowledge with
  broken links and no manifest past the hook and the pre-PR check. The format is
  0.3; see ADR-017.

### Added

- **Content packs.** A `[[packs]]` array in `.superdev/config.toml` names
  where superdev's skills, templates and scaffolds come from — a git URL with
  a `rev`, or a path on this machine — recorded relative to the repository, so
  a committed pin reads the same in every clone. Entries layer in the order
  written and a later item of the same name wins; an entry naming the source
  the binary's own content is a copy of replaces it rather than layering, so
  what that rev drops leaves the repo. A manifest naming no pack behaves
  exactly as before, and `sync` never adds an entry to one that lacks it.

  A git source is fetched with the user's own `git` — shallow, blobless and
  sparse — so credentials and forge access stay theirs and superdev holds no
  token. Each resolved pack is verified against the digest the lock records
  for its rev and kept under `.superdev/cache/packs/`, so a later `sync` or
  `status` needs the network only for bytes the machine does not have. A
  directory pack records no digest in the lock — it is read afresh every run,
  so there are no pinned bytes to check against, and a value there would be
  rewritten by every commit touching the pack and read by nothing. A rev
  that resolves to different bytes than the lock recorded fails the run and
  writes nothing: re-pinning is the only way forward, and is itself the new
  trust decision. `status` gains `content:` lines naming what it resolved
  from and any item one pack hid from another.

  `init` writes the default entry into the manifest rather than leaving the
  array absent — both resolve identically, but the written one is the pin a
  reader can see and edit. `superdev update` asks that default source for its
  newest content release and moves the pin there, even ahead of what the
  binary embeds: it is how a skill fix reaches you without a new binary. A pin
  naming any other source is reported and left alone, because naming a source
  is your trust decision to make again. With no network the pin moves no
  further than the binary's own default and the run says it could not check,
  and a manifest written by an earlier binary gains the entry on the first
  `update`. The newest release *your binary can read*: the pack is fetched
  before the pin naming it is written, and a release built for a later
  superdev leaves your pin where it is and says why. Written first, that pin
  would have been unreachable by any superdev command — `update` saves the
  manifest before the sync that validates it, and never moves a pin backwards,
  so hand-editing `.superdev/config.toml` was the only way out. That costs one
  extra fetch on a run that actually moves a pin, and nothing on the runs that
  do not. That query is the one request superdev makes that you did not ask
  for, so it is the one on a clock: a few seconds, after which a network that
  neither answers nor refuses is reported as unreachable rather than holding
  the command for however long your OS takes to give up. A clone is not
  bounded — you pinned that pack and asked for it, and a slow link is not
  superdev's to give up on. No git call prompts for credentials, so a source
  you cannot read anonymously fails rather than waiting for you to type. `source` accepts `github:owner/repo` and `gitlab:owner/repo` as
  shorthand, and otherwise a git URL over `https://`, `ssh://` or `file://` —
  the scp form and a bare ssh alias included, so your ssh config and your
  mirrors keep working.

  Releasing is one command per release. `npm run release X.Y.Z` cuts the
  binary and, from the same commit, the content release its pin names, so
  there is no second step to forget and no way to ship a binary pinned at
  content it did not embed. `npm run release:pack` cuts a content release
  alone: superdev's skills, templates and scaffolds change without a new
  binary, no workflow runs and nothing reaches a registry. The pack carries a
  version series of its own — `assets-vA.B.C`, apart from the binary's
  `vX.Y.Z` — so binary semver stays contiguous. A release candidate cuts a
  candidate content tag, which `update` never moves a released pin to; a pin
  left on one comes forward as soon as a release covers it.
- A `.agents/tools.md` scaffold: reach for internal and MCP tools before Bash.
  Written once, like the other agent rules, and imported by
  `.agents/superdev.md` after the capability files.

### Security

- A pack source can no longer choose what superdev runs, or what it arrives
  over. A `[[packs]]` entry arrives with a repository, and git's `ext::`
  transport takes a command as its connection — so on a machine configured to
  permit that transport, running `sync` or `update` in a repo you cloned ran
  whatever its manifest named. A source may now name only `https`, `ssh` or
  `file`, and a `<name>::<address>` remote helper is refused as one whatever
  its address; anything else is refused when the manifest is read, naming the
  source and the transport, before superdev spawns anything. Every git call
  superdev makes then refuses every transport it did not admit, `git://` and
  `http://` by name — neither authenticates, so anyone on the path could
  answer for a pack, and because a source keys the same however it is spelled,
  the pack they answered with would be the one that replaces superdev's own
  content. A source or rev beginning with `-` is refused when the manifest is
  read, and operands are passed after `--`. A stock git already refused
  `ext::`; what changes is that superdev no longer depends on your
  configuration to say so.
- A pack cannot ship a file it does not contain. A symlink in a pack tree was
  followed when it pointed at a file, so a pack could name a link to anything
  the user running superdev could read and have those bytes written into the
  repository as pack content. A symlink anywhere in a pack — the tree, the
  pack's own root, its `pack.toml` — now stops the run naming the path.
  Refused rather than stepped over, because a pack that resolves without the
  item its author meant a link to stand for is a pack shipping less than it
  declares, and neither `sync` nor `status --drift` would have said so. If you
  were deduplicating a file with a link, copy it instead.

  For a fetched pack, git's index decides what a link is rather than the
  filesystem. Windows without `core.symlinks` checks a link out as a plain
  file holding the target's path, which no filesystem check can tell from
  content — so the same revision used to digest differently there, and a lock
  committed from Linux failed on Windows saying only that the bytes did not
  match. It now fails on both, naming the file. A submodule under the pack is
  refused on the same answer: the shallow clone leaves it empty, so the pack
  would have shipped an item with nothing in it.

### Fixed

- The lock describes what is on disk, not only what the last run wrote. A file
  edited into agreement with what superdev ships — the way a contributor tries
  a change before shipping it — kept the hash of what it replaced, so the next
  run that touched it announced an edit nobody had made and backed the file
  up. The same staleness on a managed `.mcp.json` or `.claude/settings.json`
  key was worse: superdev decides whether such an entry is its own to remove
  by that hash, so a stale one left its registration behind for good when the
  capability was disabled.

- `superdev update` said it moved pins to this binary's defaults. It stopped
  meaning that when it began asking the pack source for a newer release — the
  one verb that reaches the network, describing itself as one that does not.
  Corrected in `--help`, the man page and the completions, which all come from
  the same line. The README now says where superdev's content comes from at
  all: what a pack is, how entries layer, and that content releases under its
  own tags.

### Changed

- The agent-rules files are each wrapped in a tag naming what they are —
  `<coding-rules>`, `<professionalism>`, `<code-exploration>` — as
  `.agents/process.md` and the aggregator already were. `.agents/codegraph.md`
  is owned, so it is rewritten on the next `sync` (backed up first); the rest
  are scaffolds and existing repos keep their own copies.

## [0.2.0] - 2026-08-25

### Changed

- **Breaking:** the knowledge-carried skill set is replaced by the process
  layer. The derived engineering skills (code-review, codebase-design,
  diagnosing-bugs, domain-modeling, tdd, to-spec, to-plan, wayfinder and the
  rest, with their MIT licence file) are swept on the next `sync` — backed up,
  as any removal is — and in their place land the eight workflow phases
  (`frame`, `spec`, `interface-design`, `feature-plan`, `build`, `verify`,
  `integrate`, `accept`) and their support skills (`adhoc-plan`, `bootstrap`,
  `brainstorm`, `grill-me`, `handoff`, `how-do-i`, `maintain`, `prototype`,
  `research`). `aokf-bootstrap` and `aokf-maintain` continue as `bootstrap`
  and `maintain`.
- The agent-rules scaffolds grow up: `.agents/prose.md` becomes
  `.agents/professionalism.md`, a new `.agents/process.md` describes the
  eight-phase workflow, and the `.agents/superdev.md` aggregator imports
  professionalism, process, then coding. Existing repos keep their own copies
  (scaffolds are write-once); fresh repos get the new set.
- The skill pack slims to `double-check` and `template-update`; `humanise`
  and `self-improve` leave the pack and are swept on sync unless marked
  custom.
- The init hint now says `run /bootstrap in Claude Code`.

### Added

- The seeded knowledge bundle now carries what the workflow skills reference:
  the document template library at `knowledge/templates/` (owned, so it
  versions with the skills), the issue-tracker convention, and a plans index
  beside the specs index — all listed from the bundle index.

## [0.1.0] - 2026-08-19

### Added

- A second project template, `web-react-android-ios-native`: one product as
  three native codebases — a React web app, a Kotlin/Jetpack Compose Android
  app and a SwiftUI iOS app, each a hello-world stub that builds and passes
  CI. It ships the tooling that makes that stack workable for agents: an HTTP
  debug server compiled into debug builds only, an MCP server wrapping it so
  an agent can drive a running app, a dev CLI for the
  build/install/launch/logs/screenshot loop, an Android-capable dev
  container, and a fastlane release pipeline keyed off a single
  `release/release.yaml`. Two artefacts can't be seeded and are bootstrapped
  instead — the Gradle wrapper jar and the Xcode project — both documented
  in the template's own `docs/BUILD.md`
- Two template tokens for spellings the slug can't express:
  `{{superdev:project-compact}}` (hyphens dropped) for reverse-domain app
  ids, which Android and iOS constrain in opposite directions, and
  `{{superdev:project-pascal}}` for Swift and Kotlin type names, Xcode
  schemes and Gradle root projects. `template render` prints both alongside
  the existing values
- Project templates: `init --template rust-npm` (or a prompt, on a TTY)
  seeds a new repo with a Rust CLI workspace deployed as prebuilt binaries
  through npm — crates, launcher and platform packages, CI and release
  workflows, repo docs, policy configs, and a dev container that brings up
  the pinned toolchain alongside superdev's own tooling, all
  token-substituted with the project's name. Template files are
  write-once scaffolds: existing files
  win, and `sync` never touches them. `--template none` and `--name` script
  the answers; `.superdev/config.toml` records the choice under `[template]`
- Knowledge-owned skills: the aokf component carries its lifecycle skills —
  the new `aokf-bootstrap` (harvest a repo's existing docs into the bundle,
  then interview the owner to flesh out the seeded skeleton) and the
  relocated `aokf-maintain` — plus the validation hook, so all three
  exist exactly where a bundle exists and a `--no-knowledge` repo gets no
  hook. `[knowledge]` takes a `custom` list like the other skill-writing
  capabilities, and a knowledge-enabled `init` ends with the
  `/aokf-bootstrap` hint
- Search down-ranks settled work: sections of a `deprecated` concept, or
  one tagged `done`, `resolved` or `wontfix`, score lower after fusion, so
  finished plans and issues stop crowding live knowledge. The index schema
  changed; the cache rebuilds itself on the next call
- A fuller knowledge seed: `init` now scaffolds a starter concept skeleton
  (glossary, architecture, testing strategy and the rest) instead of a
  three-file stub, ready for agents to fill in
- Project scaffold: CI and release machinery, npm launcher + platform
  packages, cargo workspace, and the AOKF knowledgebase
- `superdev init`, `status`, `sync` and `update`: set a repo up for
  agent-driven development and keep it matching the blueprint compiled into
  the binary. `status` exits 1 when there is work to do; a failed apply rolls
  back and reports anything it could not undo
- `.superdev/config.toml` (what the repo wants) and `.superdev/lock.toml`
  (what was applied, with hashes of superdev-owned files), plus a gitignored
  `.superdev/cache/` for backups
- Managed capabilities: `knowledge` (a native AOKF bundle), `code-index`
  (codegraph), `workflows` (Superpowers), `frontend` (Anthropic's
  frontend-design plugin) and `skills` (superdev's own pack, below); each can
  be disabled with `init --no-<capability>`
- `workflows` and `code-index` install from checksum-verified release
  bundles pinned in the binary, so `update <capability>@<version>` refuses
  an explicit version for them — bare `update` moves them to the binary's
  pins
- `superdev mcp aokf`: an MCP server over the knowledge bundle with four
  read-only tools — `aokf_search`, `aokf_read`, `aokf_graph` and
  `aokf_overview`. Search is hybrid (BM25 plus a pinned local embedding
  model, fused by reciprocal rank fusion) and degrades to lexical-only when
  no model is available. The index sits in `.superdev/cache/aokf-index/` and
  re-syncs lazily on every call, so edits are visible to the next question
- `superdev aokf validate` and `superdev aokf index`: validate the bundle
  against the AOKF spec (exit 1 on errors, `--json`, `--level`,
  `--repo-root`) and force a full index rebuild
- Optional `[knowledge.embeddings]` in `.superdev/config.toml` to embed
  through an API instead of the local model; the key comes from the
  environment, never the file
- `init` registers the server in `.mcp.json` under
  `mcpServers.superdev-aokf`, merging into whatever servers are already
  there
- The `skills` capability: five skills (aokf-maintain, double-check,
  grill-me, humanise, self-improve) written into `.claude/skills/` as
  superdev-owned files, plus a PostToolUse hook in `.claude/settings.json`
  that runs `superdev aokf hook validate` and blocks edits that break the
  bundle. Claude Code loads both natively — nothing to install
- Per-skill customisation: a `PROJECT.md` beside a skill extends it and is
  never touched; `custom = ["<name>"]` under `[skills]` releases a skill
  from management entirely. The pack's version is the binary's, so
  `update skills@<version>` is refused like the other pinned capabilities
- `superdev aokf hook validate`: the hook as a subcommand — payload on
  stdin, validates in-process, works on every platform superdev ships for
- `init` adopts a repo's existing skills: one already sitting under a pack
  name, with content of its own, is released into `[skills] custom` and
  reported, instead of being overwritten and backed up
- Blueprint migrations: `sync` now removes what the blueprint no longer
  ships — dropped files, renamed paths' old copies, a disabled capability's
  pins and registrations. Unmodified leftovers are removed with a backup;
  user-edited ones are left in place, released from the lock, and reported
- `sync` ensures `CLAUDE.md` imports `AGENTS.md` (`@AGENTS.md`), so Claude
  Code actually loads the managed entry point
- `blueprint` in `.superdev/config.toml` now records the version last
  applied: `sync` stamps it, `status` reports a difference without failing
- Knowledge-carried skills: the knowledge capability ships the full
  aokf-converted skill set — 25 skills, each its whole directory, most
  derived from mattpocock/skills (MIT) — as owned repo files under
  `.claude/skills/`, released per skill by `[knowledge] custom`. The
  workflows capability is gone: a manifest still naming `[workflows]` fails
  with a guided migration error, and the next sync after the table is
  deleted swaps same-named skills to knowledge ownership and sweeps the
  dropped upstream files. The superpowers plugin remains installable by
  hand: `claude plugin install superpowers`
- `superdev template list` and `superdev template render`: read-only views
  of the shipped project templates — render writes the token-substituted
  tree into an empty directory and prints the derived token values.
  `[template]` in `.superdev/config.toml` now records the seeding binary's
  `version`; older manifests parse unchanged
- The skill pack's `template-update`: update a repo from its project
  template, or adopt one into a repo that never used a template — the
  template confirmed with the user, a summary and per-area questions
  before anything is written, and `[template]` restamped afterwards. The
  engine still never touches template files; every update is a user edit

### Fixed

- `sync` no longer installs the repo's whole toolchain. `mise install` and
  `mise exec` now name superdev's own pinned tools, so an unrelated pin that
  cannot build on this machine no longer fails the entire apply — found
  adopting superdev in a repo pinning `cargo:cargo-ndk`

### Changed

- The AOKF validator is now the binary's own `aokf validate`; the bundled
  Python `validator.py` is deleted, and the validation hook, `check:aokf`
  and CI all call the Rust one. Findings, JSON and exit codes are unchanged
- AGENTS.md no longer preloads every concept in the knowledgebase. It keeps
  `knowledge/index.md` as the map and tells agents to search the MCP server
  for the rest

### Removed

- The skill pack's `grill-me` — the default workflows provider ships its
  own; the next sync sweeps the packaged copy (a user-edited copy is left
  in place and released). A `[skills] custom` name that is no longer in
  the pack now reports instead of failing the plan
