# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
superdev uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While
superdev is pre-1.0, minor versions may contain breaking changes.

Every released tag needs its own section here. The release workflow refuses to
publish a version it cannot find a heading for.

## [Unreleased]

### Added

- **`/file` files an issue or an idea.** A knowledge-carried skill
  writes a bug, a feature or a chore as an `open` issue in the user's
  words — no interview, no branch, no criterion the user did not state
  — numbered after the highest issue and filed by `superdev validate
  --fix`. An idea goes to `knowledge/ideas/` per `schema-idea`; an
  existing idea named with a kind is promoted to an issue that links
  it. With no kind, or one it does not know, `/file` asks and files
  nothing (ADR-048, ADR-050).
- **A heading is declared per variant.** A schema may declare one heading
  in more than one section rule when every such rule carries `variants`
  and the sets are disjoint; a document is checked against the rule its
  discriminator value selects, at that rule's place in the order, so one
  heading carries a different shape per variant. Two rules for one
  heading whose sets share a value, or of which one is untagged, are a
  finding on the schema naming the heading and the overlap, and both
  bind nothing (ADR-049).
- **The workflow skills judge and declare a contract.** `/build` carries
  JUDGE THE CONTRACTS: a contract the change touched is read as its
  consumer would read it, and the report names an included region that
  omits part of the promised surface, an optional section the kind's
  checklist names and the contract lacks, and prose a reader could not
  learn the interface from — a judgement that blocks nothing.
  `/contract-design` reads `schema-contract` in place of the per-kind
  schemas and writes a new definition element into its marked source
  region with its behaviour unbuilt; `/accept` refuses a settling
  contract whose Behaviour or Stability still carries `PENDING`
  (ADR-044). The pack ships all three.
- **The contract schema declares its twelve kinds.** `variant-key: kind`
  selects a contract's rules: twelve title rules, one per kind's display
  name, so `# CLI contract: …` is required by `kind: cli`; and each kind's
  Behaviour sections as level-3 rules tagged with the kinds they bind —
  `### Exit codes` and `### Streams` required for `cli`, `### Errors` and
  `### Versioning` for `api` and `library`, `### Authentication` for `api`
  — the optional ones declared beside them with what each says. The
  schema carries one example per kind. The filing check reports a
  contract whose id's third segment and `kind` disagree, naming both
  (ADR-043, ADR-045).
- **A schema declares variants.** `variant-key` names the frontmatter key
  whose value selects a variant; a section rule, frontmatter constraint or
  `sections-prohibited` entry tagged `variants: [v1]` binds those values
  alone, and an untagged rule binds every value. A document is checked
  against the rules its value selects, in declared order. With
  `variant-key` set, `example` is a map keyed by value, one document per
  enum value, each checked against the base rules and its own variant's,
  its value equal to its key. A tag outside the enum, a tag or keyed
  example with no `variant-key`, a `variant-key` with no enum, and a
  missing example are each an error on the schema file (ADR-045).
- **A contract's promise carries its criteria.** The contract schema's
  Behaviour and Stability rules declare a `nested` rule, so a promise
  may carry a nested bullet list of the criteria that check it, each
  opening with an `AC_` key in a code span and an EARS tag, one modal
  verb, in the promise's form; a nested item without them is an error
  naming the item, and a key repeated across the contract's `P_` and
  `AC_` keys one naming both items. A promise with no criterion is its
  own check, so a contract on file changes nothing. A criterion is cited
  as a promise is: `AC_<slug>` where the contract is the subject,
  `<contract id> AC_<slug>` elsewhere (ADR-050, ADR-051).
- **A section rule declares `nested` and `item-key-optional`.** `nested`
  is the rule for the items one level below the section's own — `item-key`,
  `item-pattern`, `item-prohibited-pattern`, `required` and its own
  `nested`, to any depth. A nested item is a marker of the section's list
  kind indented past the item above it; a deeper marker than the declared
  depth, or one of the other kind, is text of the item it sits in. Each
  level's items are checked as the top level's are, one finding per item,
  the finding naming the nested item; `required` reports an item with no
  nested item beneath it; a nested key is unique with every key of the
  document. `item-key-optional` makes an item not matching `item-key` a
  plain item, held to `item-prohibited-pattern` alone, while a keyed item
  is held to `item-pattern` and `nested`. A `nested` on a section with no
  list content, a nested key whose capture count is not one, and the flag
  with no `item-key` are each an error on the schema file (ADR-051).
- **A section rule declares `item-key`, `item-only-pattern` and
  `item-prohibited-pattern`.** `item-key` is a regex with one capture
  group every top-level item of the section's list must match; the
  capture is the item's key, unique across the document's items under
  every rule declaring `item-key`, and an item with no match or a
  repeated key is an error naming the item. `item-only-pattern` may
  match only inside a top-level item, and a match on any other body line
  is an error naming the line. `item-prohibited-pattern` is a regex no
  top-level item may match, a match naming the item and the text. All
  three skip fenced blocks, and an item draws one finding: `item-key`,
  then `item-prohibited-pattern`, then `item-pattern`. A mis-declared
  pattern is an error on the schema file and binds nothing; `--fix`
  never supplies a key (ADR-047).
- **A schema section may declare `content: include`.** The sixth content
  kind is satisfied by an include block naming a source path, and a fenced
  block in such a section outside an include is an error naming the
  section: a definition is materialised, never authored (ADR-042). Nothing
  inside an include is read by the schema check. The `block-language`,
  `block-keys` and `block-entry-keys` declarations below are withdrawn with
  the block check they drove; a schema still carrying one reports an
  unknown key on the schema file. The contract schema's Definition section
  now declares `include`.
- **An include block names a source region.** Beside a concept id, an
  include block's argument may be a `/`-rooted repository path with an
  optional `#region`: `<!-- sokf:include /src/main.rs#cli -->`, bounded
  by lines containing `sokf:begin cli` and `sokf:end cli` in the file's
  comment syntax; regions sharing a name concatenate, and a path with no
  `#` includes the whole file. `superdev validate --fix` writes the region
  as a fenced block tagged by the file's extension, and `superdev
  validate` reports a stale, empty or absent block, a missing or
  out-of-repository path, and an absent region, each naming the path and
  the region (ADR-041).
- **The file-format contract kind split into text and binary** (ADR-037),
  retiring `schema-contract-file-format`; both kinds were then folded
  into the one contract schema (ADR-043, above).
- **A drift failure says which way the difference runs.** An element the
  implementation carries and its contract does not declare reports as a
  `DEFECT`; one the contract promises and the implementation has yet to
  keep reports as `PENDING` (ADR-038), and `/accept` refuses a settling
  contract still carrying the marker. A plan orders a block that closes
  a contract-implementation gap before the blocks that do not, so a
  contract's promise does not fail the blocks that do not own it.
- **The CLI and MCP contracts define their surfaces.** `contract-002`
  carried every command, flag and exit code and `contract-003` every
  tool's arguments and result shape (ADR-036); the blocks and their
  drift tests were later replaced by source includes (ADR-042, below).
- **A schema can bind the shape of a section's text.** A section rule
  declares `item-pattern`, which every top-level item of the section's list
  must match, and `content-pattern`, which the section's whole body must
  match; both are regexes matched found-anywhere (ADR-030). A pattern that
  does not compile, or an `item-pattern` beside a content kind with no
  items, is reported on the schema and binds nothing.
- **A knowledge document can include another's body.** A concept carries a
  `<!-- sokf:include <id> -->` … `<!-- /sokf:include -->` marker pair, and
  `superdev validate --fix` materializes the named concept's body between
  the markers, refreshing every copy when the source changes; bare
  `validate` reports a stale, empty or unresolvable include block as an
  error (ADR-027). Shared prose gets one authored home while every file
  stays self-contained on disk.
- **A schema's worked example is checked.** `superdev validate` reads each
  schema's `example:` block as a document and checks it against the schema
  that declares it, reporting every failure as an error on the schema
  file prefixed `example:` (ADR-024). An example that does not parse as
  a document is an error on the schema file too; a schema that names its
  documents by glob owes no frontmatter block in its example.
- **An example's links bind by form and never resolve.** Inside a
  schema's example, a concept link takes the `[text][sokf:<id>]`
  reference form; a link whose target is a path into the knowledge, and
  a `sokf:` destination written inline, are each an error on the schema
  file (ADR-025). No id or target is resolved, and the filing check
  never reads an example.
- **Required frontmatter keys bind.** `superdev validate` reads the
  per-key `required: true` flag a schema declares (ADR-022) and reports,
  as an error naming the document, the key and the schema, an absent key
  marked required; a present one keeps its value checks. The shipped
  schemas each declare their required keys, so a filed document that
  loses its identity or its listing line fails validation.
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

- **A list kind needs a top-level item, and an item ends at a heading.**
  `content: bullet-list` or `numbered-list` is satisfied only by a
  top-level item as the item declarations read one — a bullet nested
  under a numbered step or a `- - -` break is not one — so a keyed rule
  never passes over a list it cannot bind. A heading, a table row or an
  HTML comment directly under an item ends the item rather than joining
  it as a continuation, so a heading's verb is reported outside the item
  and not as the item's; an HTML comment line is not bound by
  `item-only-pattern`, as it is not prose; and an item two keyed rules
  capture — a level-2 rule over its level-3 subsections — counts once,
  so it does not repeat its own key. The `item-only-pattern` finding
  says "outside a top-level item", and the contract schema says a nested
  bullet is not a promise. The `item-pattern` of a criterion and of a
  promise opens with the same key grammar `item-key` declares.
- **A checkout with CRLF line endings validates.** Every check behind
  `superdev validate` compared bytes, so a Windows checkout — where git hands
  the knowledge tree CRLF unless `.gitattributes` says otherwise — registered
  no schema at all, reported every document as naming no schema and every
  generated block as ungenerated. A line is now the same line whatever ends
  it: the checks read lines with the terminator gone and compare them a line
  at a time, in both halves of the validator and in the drift tests. Nothing
  normalises what it reads, so a document keeps its own line endings and
  `validate --fix` leaves a CRLF file alone instead of rewriting it to LF.
- **An unreadable named path is reported as it was typed.** `superdev
  validate <path>` reported a path it could not read absolutised and with
  the platform's separator, so one report carried two spellings of a path
  and a Windows caller was handed back one they never typed. The error now
  uses the repository-relative, forward-slashed spelling every finding uses.
- **A link to a file that is not there fails the run.** Five findings the
  repository alone can settle — a broken body link, a missing `resource`, a
  missing `sources[].resource`, an index entry naming a missing file, and a
  footnote label matching no `sources[].id` — were warnings, so nothing
  failed and nobody read them: the canonical knowledge carried 39 unactioned
  until someone happened to look. They are errors (ADR-039). A non-core `rel`
  stays a warning and is now the only one the document check emits: the tier
  is split by decidability rather than emptied. **SPEC §10 and §11 change with
  them** — permissiveness binds a consumer displaying knowledge, not a
  validator checking a repository.
- **A turn does not end while the knowledge carries an error.** Two of the
  five are what work in progress looks like: a link written before the file
  it points at lands. The PostToolUse hook no longer blocks on those, because
  it is handed one edited file and cannot see whether the target arrives in
  the next edit — so a plan citing the concepts its own slices will add stays
  writable. The Stop hook holds the turn instead, naming what stands, which
  is where a document is claimed to be finished. It lets the turn end on
  knowledge it cannot read, and stops after three holds, so a finding nobody
  can settle stalls nothing.
- **The template contract is bound to the templates.** `contract-008` names
  the five substitution tokens and carries one section per shipped template,
  and both are now compared to the binary in each direction: a token the
  binary substitutes without declaring it, or a template it ships without
  describing, reports as a `DEFECT`; one the contract names and the binary
  does not have reports as `PENDING`. Eight of the nine active contracts were
  bound already; this was the ninth.
- **A `..` cannot walk a repair out of the knowledge.** The guard that bounds
  `validate --fix` compared an unresolvable path lexically, and
  `<knowledge>/gone/../../elsewhere` begins with the knowledge root as text
  while the filesystem lands it outside. `canonicalize` resolves none of it,
  because the components do not exist. Such a path is now refused rather than
  resolved, in both the move and the write guard.
- **`validate --fix` refiles under a symlinked root.** The guard that keeps
  a repair inside the knowledge resolved the destination of a move, which
  does not exist until the move creates it, and fell back to comparing an
  unresolved path against a resolved root — so refiling was refused wherever
  the knowledge is reached through a symlink, which on macOS is any path
  under `/tmp` or `/var`. It now resolves the nearest existing ancestor,
  and still refuses a genuine escape.
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

- **An issue is one template: `kind` in the frontmatter, `open`, `done`
  or `wontfix`, six headings in prose and bullets, no key.** `schema-issue`
  governs every issue — `type: Issue`, id `issue-<nnn>-<slug>`, `kind`
  one of `bug`, `feature`, `chore`, `lifecycle` one of `open`, `done`,
  `wontfix`, the folder the value — with the headings Summary, Context,
  Behaviour, Scope, Resolution and Comments, each prose with bullets
  beneath it and no `item-key`, `item-pattern` or `nested` declaration;
  Resolution is required once the issue is done or wontfix and refused
  while it is open. Keys and EARS live in the contracts alone.
  **`schema-bug-report`, `schema-feature-request` and `schema-chore`
  retire**, and `framed` and `unframed` with them: `sokf_search` ranks
  `open` and `active` as live and every other value settled. `/file`
  writes the template. superdev's own 52 issues were rewritten by hand —
  the verdict under Resolution, the criteria and the expected behaviour
  as plain bullets, the ids shortened (`issue-030-filing-…`) and every
  citation with them. **After a pack update, a managed repository's
  issues fail `superdev validate`** as documents naming no schema: set
  each one's `type` to `Issue`, add `kind`, set a `framed` or `unframed`
  `lifecycle` to `open`, move the verdict under `## Resolution` and the
  remaining sections under the six headings, drop the item keys and
  tags, and run `superdev validate --fix`, which refiles `issues/framed/`
  and `issues/unframed/` into `issues/open/`; delete the emptied
  folders. The id's kind segment may stay — the pattern admits any slug
  (ADR-050).
- **A plan is one template: Goal, Contract changes, Work blocks, Deferred
  decisions.** `schema-plan` governs every plan — `type: Plan`, id
  `plan-<nnn>-<slug>`, `lifecycle` one of `open`, `done`, `abandoned`,
  the folder the value — a `# Plan: …` heading with a Request line where
  the plan delivers an issue; Goal in prose; Contract changes as one
  bullet per contract touched naming the promises and criteria added,
  changed or withdrawn, or the single bullet "none"; Work blocks as
  `### Block n: …` bullet lists carrying a Done checkbox, Depends-on,
  Change, Done-check and Cases — a case citing the contract criteria it
  covers by key where one exists and stating what it checks otherwise;
  Deferred decisions optional. Blocks sort by dependency, then a block
  closing a contract-implementation gap, then risk, as slices did.
  **`schema-feature-plan` and `schema-adhoc-plan` retire.** superdev's
  own 27 plans were rewritten by hand — slices became blocks, an ad-hoc
  plan's fourteen sections folded into the four, a feature plan's
  contract changes recovered from its issue — and the ids shortened
  (`plan-026-filing-…`) with every citation. **After a pack update, a
  managed repository's plans fail `superdev validate`** as documents
  naming no schema: set each one's `type` to `Plan`, retitle it
  `# Plan: …`, fold the body into Goal, Contract changes, Work blocks
  and Deferred decisions with `### Slice n:` or `### Wn:` headings
  becoming `### Block n:`, and run `superdev validate --fix`. The id's
  kind segment may stay — the pattern admits any slug (ADR-050).
- **`/scope` replaces `/frame`, `/feature-plan` and `/adhoc-plan`, and
  `/contract-design` becomes its sub-skill.** `/scope` takes a filed
  issue or a one-off request: it cuts the branch —
  `feature/<nnn>-<slug>` after the issue, `adhoc/<nnn>-<slug>` after the
  plan where there is no issue — calls `/grill-me` where the design is
  open, `/research` for an external fact and `/design` or `/prototype`
  for UI, decides the contract changes, writes the plan per
  `schema-plan`, double-checks it, commits the plan with the contract
  edits on the branch, and hands to `/build`. `/contract-design` is a
  phase no longer: it takes one plan's Contract changes, makes the
  contract and source-declaration edits, records the ADRs, presents the
  change set for the user's go-ahead and hands back to `/scope`, which
  commits. **After a pack update, `/frame`, `/feature-plan` and
  `/adhoc-plan` are gone from `.claude/skills/`**: invoke `/scope` in
  their place, and rewrite every skill, hook or note in a managed
  repository that names one (ADR-050).
- **The workflow is FILE → SCOPE → BUILD → ACCEPT, accept optional, and
  `/build` verifies once.** `/build` works the plan's Work blocks in
  order — tests, then code, then the block's own tests and the tests its
  change touches, then a commit that ticks the block — and after the
  last block runs the full build, tests, lint and `superdev validate`
  once, checks and judges the contracts, updates the changelog and the
  knowledge, merges on the work's branch and sets the plan `done`. A
  block needing a contract change, or too big to commit in one pass,
  returns to `/scope`. **`/integrate` retires** with its verification
  per slice; its update onto the merge target, contract check and
  judgement, migration guide and records commit close `/build` instead.
  **`/execute-feature-plan` is renamed `/execute-plan`**, driving
  `/build` over the blocks with the run verbs and the Stop hook
  unchanged, returning a failing block to `/build` at most twice and
  deferring every question a gate sends to `/scope`. `/accept` is the
  user's optional last step: `/code-review` of the whole change, a
  finding the user wants fixed returning to `/build`, the contract
  criteria walked on the merged code, the documentation checked against
  the issue's Behaviour, each gap filed as an `open` issue, and the
  issue set `done` with its Resolution. **After a pack update,
  `/integrate` and `/execute-feature-plan` are gone**: invoke
  `/build` and `/execute-plan` in their place, and rewrite every skill,
  hook or note in a managed repository that names one (ADR-050).
- **The concepts, the glossary and the README carry the four-phase
  workflow.** `development-procedure`'s Workflow reads FILE → SCOPE →
  BUILD → ACCEPT; the glossary defines Scope, Plan and Work block and
  drops the framed and unframed states, the slice and integrate; the
  contracts, issues and plans indexes name `/contract-design` and the
  plan's work blocks; the schemas' prose follows; and the README's
  Usage section carries the loop and the two templates. **A managed
  repository's own concepts are write-once scaffolds superdev never
  rewrites**: edit by hand any that name `/frame`, `/feature-plan`,
  `/adhoc-plan`, `/integrate` or `/execute-feature-plan` (ADR-050).
- **The backlog retires.** Its entries became ideas 007 to 009 and the
  wontfix chore I051, and `schema-backlog`, the concept and every
  reference to it go. **A managed repository's `Backlog` document
  loses its schema** and fails `superdev validate` as a document naming
  no schema: file each entry as an idea per `schema-idea` or as a
  `wontfix` issue, then delete the document (ADR-048).
- **A contract's Behaviour and Stability are keyed EARS promises.** The
  contract schema declares both sections as bullet lists whose every
  item, at any heading depth, opens with a `P_` key in a code span and
  an EARS tag — `[ubiquitous]`, `[event]`, `[state]`, `[conditional]`,
  `[optional]`, `[complex]` — and carries the interface element as its
  subject and one verb from `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD
  NOT` and `MAY`; `MUST`, `REQUIRED`, `RECOMMENDED` and `OPTIONAL` are
  retired from contracts. Prose in either section describes and
  carries no modal verb, a numbered list is a sequence and never a
  promise, a table stays where the kind's checklist wants one, `PENDING`
  sits beside the verb, and no item reads `TBD`. The key is the
  promise's identity — a rewording keeps it, a removed key is not
  reused — cited bare where the contract is the subject and after the
  contract's id elsewhere: `contract-002-cli-superdev
  P_init-outside-git`. **After a pack update, a contract whose
  Behaviour or Stability is prose with a modal verb fails `superdev
  validate`**, naming each keyless, tagless, retired-verb, two-verb or
  `TBD` item and each modal verb outside an item; `--fix` rewrites
  nothing. The schema's twelve examples carry the form, and superdev's
  nine active contracts were swept to it: 182 modal verbs became 174
  keyed promises, a sentence that stood in two places becoming one
  promise cited from the other (ADR-046, ADR-047).
- The two hook entries `sync` writes into `.claude/settings.json` carry
  `timeout: 30`, so a hook that wedges — a `cargo run` waiting on a build
  lock — is killed by Claude Code after 30 s instead of holding the
  session open.
- **A contract includes its definition, under one schema.** Every
  contract's Definition is one or more source includes, as the
  `content: include` and source-region entries above say (ADR-041,
  ADR-042). One schema, `Contract`, governs every kind: a `kind` from twelve — `api`,
  `events`, `cli`, `library`, `interface`, `ui`, `data`, `format`,
  `config`, `telemetry`, `authz`, `deployment` — in the frontmatter and
  the id, a title opening with the kind's display name, a Definition, a
  Behaviour of RFC 2119 prose under one `###` per checklist item the
  kind requires, and a Stability promise (ADR-043, ADR-045). **The
  sixteen contract-kind schemas are deleted**, `schema-contract-authz`
  through `schema-contract-ui`, and the `contract-style` fragment with
  them: the standard is prose in the one schema. **After a pack update, a
  contract under a per-kind type fails `superdev validate`**: set `type:
  Contract`
  and a `kind`, rename the id's third segment to the kind, replace the
  hand-written definition with an include of the declaring source, and
  regroup the promise sections as `###` under Behaviour; `--fix` refiles
  it. A definition is bound by materialisation, which superdev supplies;
  a Behaviour or Stability promise is bound by a test of the behaviour,
  which the project writes; `PENDING` marks a prose promise whose
  behaviour is unbuilt and nothing else, and CONTRACT-DESIGN writes a new
  definition element into its source region first (ADR-044). This
  repository's nine contracts moved, the four tests that compared a
  hand-written copy to the code are gone, and the block-shape check of
  ADR-035 — never released — is withdrawn with the copies it checked.
- **The CLI, MCP, config and format contracts include their source.**
  `contract-002`'s Definition is the clap tree from the `cli` regions of
  `crates/app/superdev/src/`, its exit codes a per-command table under
  `### Exit codes`, and `contract_exit_codes.rs` still exercises every
  code. The MCP contract is `contract-003-api-sokf`, kind `api`, its
  Definition the argument structs and tool methods from the `tools`
  regions of `mcp.rs`. `contract-004`, kind `config`, includes the
  manifest's on-disk shape from the `config` regions of `manifest.rs`
  and `sokf/embed.rs`, doc comments included, in place of the Settings
  table and the `config.toml` block; its Behaviour carries Sources and
  precedence — naming `CLAUDE_SESSION_ID`, which `run begin` and `run
  advance` read — Defaults, Secrets and Validation. The pack and lock
  contracts are `contract-005-format-pack` and `contract-006-format-lock`,
  kind `format`, including the `pack` region of `pack/manifest.rs` and
  the `lock` regions of `lock.rs`; the pack contract names
  `agents/superdev.md` as the refused path, which is what the source
  refuses. The hand-written blocks and the tests that compared them to
  the code are gone (ADR-042).
- **`superdev validate` counts its warnings and lists them on request.** A run
  lists every error and closes with both counts, as before; the warnings
  themselves now appear only under **`--warnings`** (ADR-040). Warnings are
  what the repository alone cannot settle — a custom `rel`, a frontmatter key
  whose portability depends on where a skill is published — so none of them is
  actionable for the edit in hand, and this repository reprinted the same five
  on every hook-triggered run. The counts are read from the findings and not
  from what was printed, so a warning nobody listed is still counted. **No
  verdict moves**: the same findings are found, and the same exit codes are
  returned. `--json` gains **`errors` and `warnings` counts, which it never
  carried**, beside the findings the text run listed — **a consumer that
  derived its counts from the `findings` array now undercounts** and should
  read the two counts instead, or pass `--warnings`; the keys `documents` and
  `schemas`, emitted since they were added and never declared, are now in
  `contract-002` with them. The **PostToolUse and Stop hooks default the same
  way**, so an agent editing one document reads what it can act on and a
  count of what it cannot, in place of the whole repository's advisory
  findings on every pass.
- **A contract defines its interface and its promises bind.** The
  contract-kind schemas, before the one `Contract` schema replaced them,
  demanded a definition block carrying the whole surface a caller
  depends on — commands and flags, tool schemas, settings, file shapes,
  exported signatures — in the form that kind's ecosystem reads, bound
  by the project to its implementation (ADR-033, ADR-034, ADR-036), and
  carried the RFC 2119 keyword as an `item-pattern` on sections where
  every entry is a promise and as a `content-pattern` on Stability and
  its like (ADR-032), with the contract style standard — one RFC 2119
  sentence per requirement, structured forms for enumerable surfaces,
  reasoning in the linked ADRs. **`ADR-029` is superseded**, and its
  judgement-based standard with it. A contract with no binding — no
  generation from it, no test against it — is non-compliant even while
  it validates. superdev's nine active contracts were rewritten to the
  bar: the CLI contract had drifted from the binary, the MCP contract
  carried no tool schema, the config contract never declared its
  `[template]` table, and the pack-layout tables were brought current
  (schemas and fragments ship; the dropped templates kind is gone).
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
- **Documents are filed by lifecycle.** Breaking change to the knowledge
  layout. Issues, plans, specs, decisions and contracts each carry one
  `lifecycle` frontmatter key — `open`/`done`/`wontfix` for issues,
  `open`/`done`/`abandoned` for plans, `active`/`deprecated` for specs,
  decisions and contracts — and every document sits in a folder named
  exactly its value (`knowledge/issues/done/`, `knowledge/plans/done/`),
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
  over. A source may name only `https`, `ssh` or `file`; a `<name>::<address>`
  remote helper, git's `ext::` transport, `git://` and `http://` are refused
  when the manifest is read, naming the source and the transport, before
  superdev spawns anything, and every git call superdev makes refuses every
  transport it did not admit. A source or rev beginning with `-` is refused,
  and operands are passed after `--` (ADR-012).
- A pack cannot ship a file it does not contain. A symlink anywhere in a pack
  — the tree, its root, its `pack.toml` — stops the run naming the path; for
  a fetched pack, git's index decides what a link is, so a lock committed from
  Linux fails on Windows the same way, naming the file, and a submodule under
  the pack is refused on the same answer. Copy a file you were deduplicating
  with a link (ADR-014).

### Fixed

- The lock describes what is on disk, not only what the last run wrote: a
  file edited into agreement with what superdev ships keeps the hash of what
  is on disk, so the next run neither announces an edit nobody made nor
  leaves a managed `.mcp.json` or `.claude/settings.json` entry behind when
  its capability is disabled.
- `superdev update` describes itself as the one verb that reaches the network
  in `--help`, the man page and the completions; the README says where
  superdev's content comes from — what a pack is, how entries layer, and that
  content releases under its own tags.

### Changed

- The agent-rules files are each wrapped in a tag naming what they are —
  `<coding-rules>`, `<professionalism>`, `<code-exploration>` — as
  `.agents/process.md` and the aggregator already were. `.agents/codegraph.md`
  is owned, so it is rewritten on the next `sync` (backed up first); the rest
  are scaffolds and existing repos keep their own copies.

## [0.2.0] - 2026-08-25

The entries moved to [CHANGELOG-0.1-0.2.md](CHANGELOG-0.1-0.2.md) when
this file reached its 800-line limit; they are unchanged there.

## [0.1.0] - 2026-08-19

The entries moved to [CHANGELOG-0.1-0.2.md](CHANGELOG-0.1-0.2.md) when
this file reached its 800-line limit; they are unchanged there.
