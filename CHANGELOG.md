# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
superdev uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While
superdev is pre-1.0, minor versions may contain breaking changes.

Every released tag needs its own section here. The release workflow refuses to
publish a version it cannot find a heading for.

## [Unreleased]

### Added

- **A heading is declared per variant.** A schema may declare one heading
  in more than one section rule when every such rule carries `variants`
  and the sets are disjoint; a document is checked against the rule its
  discriminator value selects, at that rule's place in the order, so one
  heading carries a different shape per variant. Two rules for one
  heading whose sets share a value, or of which one is untagged, are a
  finding on the schema naming the heading and the overlap, and both
  bind nothing (ADR-049).
- **The workflow skills judge and declare a contract.** `/integrate` gains
  JUDGE THE CONTRACTS: a contract the slice touched is read as its
  consumer would read it, and the report names what was checked and
  where an included region omits part of the promised surface, where an
  optional section the kind's checklist names is absent with no reason,
  and where a reader could not learn the interface — a judgement that
  blocks nothing, and a slice touching no contract says so.
  `/contract-design` reads `schema-contract` in place of the per-kind
  schemas, writes a new definition element into its marked source region
  with its behaviour unbuilt, and presents and commits that declaration
  under the approval it already requires. `/accept` refuses a settling
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
- **A section rule declares `item-key`, `item-only-pattern` and
  `item-prohibited-pattern`.** `item-key` is a regex with one capture
  group that every top-level item of the section's list must match; the
  capture is the item's key, unique across the document's items under
  every rule declaring `item-key`. An item with no match is an error
  naming the section, the item and the form a key takes, and a repeated
  key one naming the key and both items. `item-only-pattern` is a regex
  that may match only
  inside a top-level item of the section's list: a match on any other
  body line — prose, a table row, a heading, an item of the other list
  kind, a nested item — is an error naming the section and the line, and
  on a section with no list content every line is outside.
  `item-prohibited-pattern` is a regex no top-level item may match: a
  match is an error naming the item and the matched text. All three skip
  fenced blocks and read an item as `item-pattern` does, and an item
  draws one finding: `item-key` is checked first, then
  `item-prohibited-pattern`, then `item-pattern`, and an item reported
  by one is not checked by the next. A key pattern
  whose capture count is not one, an `item-key` or
  `item-prohibited-pattern` on a section with no list content, and a
  pattern that does not compile are each an error on the schema file and
  bind nothing. `--fix` never supplies a key (ADR-047).
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
  optional `#region`: `<!-- sokf:include /src/main.rs#cli -->`. The region
  is bounded by lines containing `sokf:begin cli` and `sokf:end cli`, in
  whatever comment syntax the file uses; regions sharing a name concatenate,
  and a path with no `#` includes the whole file. `superdev validate --fix`
  writes the region as a fenced block tagged by the file's extension,
  carrying a `sokf:generated-by` line from the file's head, and
  `superdev validate` reports a stale, empty or absent block, a path that is
  missing or resolves outside the repository, and a region the file does not
  carry, each as an error naming the path and the region (ADR-041).
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
- **The tracker's cited lists carry keys.** A feature-request's
  acceptance criteria open with an `AC_` key in a code span before the
  EARS tag or the `TBD`; a bug-report's repro steps with an `RS_` key
  and a chore's definition of done with a `DD_` key, each with no tag —
  **a step or a done item carrying an EARS tag after its key is an
  error** naming the item and the tag. The key is the item's identity,
  unique within the issue, and the
  number the reading order; a plan case cites the keys it covers, bare
  ("covers AC_c1, AC_stale-include"), and elsewhere the issue's id
  precedes the key. **After a pack update, an issue whose cited list
  carries a keyless, mis-prefixed or repeated item fails `superdev
  validate`**, naming the item; `--fix` supplies no key. The fifty
  issues on file were swept once to the slug `c<n>`, `n` the item's
  number — 141 criteria, 72 repro steps, 22 done items — so every
  citation of a number stands, and every open plan's cases cite keys
  (ADR-046).
- The two hook entries `sync` writes into `.claude/settings.json` carry
  `timeout: 30`, so a hook that wedges — a `cargo run` waiting on a build
  lock — is killed by Claude Code after 30 s instead of holding the
  session open.
- **A contract includes its definition, under one schema.** Every
  contract's Definition is one or more source includes —
  `<!-- sokf:include /path#region -->` — that `superdev validate --fix`
  materialises from the marked regions of the code and `superdev
  validate` keeps current; a fenced block authored in a Definition is an
  error, and nothing inside an include is parsed (ADR-041, ADR-042). One
  schema, `Contract`, governs every kind: a `kind` from twelve — `api`,
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
- **The CLI and MCP contracts include their source.** `contract-002`'s
  Definition is the clap tree materialised from the `cli` regions of
  `crates/app/superdev/src/`, one include per file, and its exit codes are
  a per-command table under `### Exit codes` in Behaviour; the hand-written
  YAML block and the test that compared it to the binary are gone
  (ADR-042). The MCP contract is now `contract-003-api-sokf`, kind `api`,
  its Definition the server's argument structs and tool methods from the
  `tools` regions of `mcp.rs`; its drift test is gone with the JSON block.
  `contract_exit_codes.rs` still exercises every code the contract states.
- **The config and format contracts include their structs.** `contract-004`
  is kind `config`: its Definition is the manifest's on-disk shape
  materialised from the `config` regions of `manifest.rs` and
  `sokf/embed.rs`, doc comments included, in place of the Settings table
  and the hand-written `config.toml` block; Behaviour carries Sources and
  precedence — now naming `CLAUDE_SESSION_ID`, which `run begin` and `run
  advance` read — Defaults, Secrets and Validation. The pack and lock
  contracts are `contract-005-format-pack` and `contract-006-format-lock`,
  kind `format`, their Definitions the `pack` region of `pack/manifest.rs`
  and the `lock` regions of `lock.rs`; the pack contract now names
  `agents/superdev.md` as the refused path, which is what the source
  refuses. The four tests that parsed the contracts' TOML blocks are gone
  (ADR-042).
- **`superdev validate` counts its warnings and lists them on request.** A run
  lists every error and closes with both counts, as before; the warnings
  themselves now appear only under **`--warnings`** (ADR-040). Warnings are
  what the repository alone cannot settle — a custom `rel`, a frontmatter key
  whose portability depends on where a skill is published — so none of them is
  actionable for the edit in hand, and this repository reprinted the same five
  on every hook-triggered run. The counts are read from the findings and not
  from what was printed, so a warning nobody listed is still counted. **No
  verdict moves**: the same findings are found, and the same exit codes are
  returned.

  `--json` reports the same information as the text output: **`errors` and
  `warnings` counts, which it never carried**, and the findings the text run
  listed. **A consumer that derived its counts from the `findings` array now
  undercounts** and should read the two counts instead, or pass `--warnings`.
  The keys `documents` and `schemas`, emitted since they were added and never
  declared, are now in `contract-002` with them.

  The **PostToolUse and Stop hooks default the same way**, so one rule holds
  whoever ran the check. This is what the change is for: an agent editing one
  document read the whole repository's advisory findings on every pass, and
  now reads what it can act on and a count of what it cannot.

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

The entries moved to [CHANGELOG-0.1-0.2.md](CHANGELOG-0.1-0.2.md) when
this file reached its 800-line limit; they are unchanged there.

## [0.1.0] - 2026-08-19

The entries moved to [CHANGELOG-0.1-0.2.md](CHANGELOG-0.1-0.2.md) when
this file reached its 800-line limit; they are unchanged there.
