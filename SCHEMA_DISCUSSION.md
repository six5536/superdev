# Schema review — findings for discussion

Review of every schema under `knowledge/schemas/`, carried out 2026-09-11 against
the tree at that date. The brief was a general review plus three specific
questions: which documents duplicate content given the workflow, which documents
are unnecessary, and which documents fail to state both what they must or may
include and what they must not. Section 5 answers a fourth: whether the content
now in `architectural-rules` has a better-structured home.

Each finding carries an ID, the evidence, and an empty **Outcome** line to fill
in as we settle it. Nothing here has been changed in the tree.

> **Framing.** The schemas are generic. They ship in `pack/` and superdev writes
> them into any project it manages, so a schema's worth is whether it governs a
> document class projects produce — not whether *this* repository happens to
> carry an instance. superdev is one consumer of the set, and a narrow one: a
> single Rust CLI with no UI, no release cadence yet and no incidents on record.
> An earlier draft of this review used "zero instances here" as evidence of
> redundancy. That reasoning was wrong and section 2 has been rewritten without
> it. Instance counts appear below only as a signal about *superdev's own
> knowledge*, never as a verdict on a schema.

---

## 0. Inventory and baseline

38 schemas plus `index.md`, 5,151 lines in total. `superdev validate` reports
`PASS (0 error(s), 0 warning(s))` over 240 documents against 38 schemas, so every
finding below is invisible to the tool as it stands.

**There are two copies of all of this.** `pack/knowledge/schemas/` holds all 39
files, byte-identical to `knowledge/schemas/`, tracked in git, outside the
validator's roots. The real figure is therefore 78 files and 10,302 lines. See
D7; every finding below applies twice unless stated otherwise.

Sizes, smallest to largest:

| Lines | Schema |
| ----- | ------ |
| 54 | fragment |
| 58 | dependency-policy |
| 60 | development-commands, directory-structure, glossary |
| 61 | architectural-rules, definition-of-done |
| 63 | schemas-index |
| 64 | technology-stack |
| 70 | project-overview |
| 71 | constraints-non-goals, release-procedure |
| 72 | security-requirements |
| 75 | development-procedure |
| 76 | research |
| 78 | error-handling |
| 83 | documentation |
| 88 | issue-tracker |
| 89 | software-components |
| 90 | changelog |
| 91 | architecture |
| 95 | testing-strategy |
| 99 | configuration |
| 103 | coding-standards |
| 106 | index |
| 109 | code-review |
| 121 | investigation, release-notes, visual-system |
| 122 | status-update |
| 124 | adr |
| 140 | security-review |
| 142 | idea, readme |
| 157 | migration-guide |
| 162 | postmortem |
| 272 | issue |
| 351 | plan |
| 1169 | contract |

Coverage against documents that actually exist in the repository:

| Group | Schemas | Instances |
| ----- | ------- | --------- |
| Workflow records | issue, plan, contract, adr, idea | 94 / 30 / 11 / 54 / 17 |
| Knowledge concepts | 21 singletons (architecture, glossary, configuration, …) | 1 each |
| Reports | code-review | 11 |
| Reports | security-review, investigation, postmortem, status-update, release-notes, migration-guide | **0 each** |
| Project files | readme, changelog, schemas-index | 1 each |
| Infrastructure | research, documentation | 1 each |
| Infrastructure | fragment, visual-system | **0 each** |

---

## 1. Duplicated content

### D1 — The plan work-block contract lives in Rust, not in the schema

`schema-plan` declares the repeating block as:

```yaml
- heading-pattern: '^Block \d+: .+$'
  level: 3
  required: true
  repeatable: true
  content: bullet-list
  description: Stable block carrying completion, dependencies, outcomes, executable evidence, and documentation.
```

`content: bullet-list` is the whole of it. The schema carries no `item-pattern`,
no `item-key`, no `item-only-pattern`, no `item-prohibited-pattern` — it is one of
30 schemas with zero item-level or content-level constraints.

The actual contract is hardcoded in
`crates/lib/superdev-core/src/validate/sokf.rs:333-375`:

- required bullets: `Dependencies:`, `Areas:`, `Outcome:`, `Verification:`,
  `Tests:`, `Structural evidence:`, `Documentation:`
- a `- [ ] Done` / `- [x] Done` / `- [X] Done` checkbox
- a prohibition: `workflow: plans contain executable evidence only; `manual:` is prohibited`

`crates/lib/superdev-core/src/validate/fix.rs:159-218` carries a second copy in
migration form: the rename `- Done-check:` → `- Verification:`, and the insertion
of a default `- Structural evidence:` line.

So the block shape has three homes — schema prose, `sokf.rs`, `fix.rs` — and only
the latter two bind. The grammar already supports declaring all of it in the
schema; `schema-contract` proves it with 14 item-level rules. This is precisely
the drift the schema layer exists to prevent.

- **Impact:** an agent reading `sokf:schema-plan` to author a plan cannot see the
  rules it will be judged by.
- **Fix:** move the bullet set and the `manual:` prohibition into `schema-plan`
  as `item-pattern` and `item-prohibited-pattern`; leave in Rust only what the
  schema vocabulary cannot express.
- **Outcome:** _(to record)_

### D2 — `Final verification` is prose in two places and a rule in none

`schema-plan` explains the focused-vs-complete command split twice:

- in the `# Plan Schema` preamble: "Active evidence is executable; …"
- in the fenced contract's `description:` — "Declare focused commands under each
  block's Verification bullet. Declare complete suites under Final verification;
  run those only after every block is done."

No section rule names `Final verification`. The `Documentation changes` section
rule, which is where the SCOPE skill instructs the author to put it
(`.pi/extensions/superdev/skills/scope/SKILL.md`, step 4), describes only "Every
applicable declared surface and its owner and commands, or checked none
evidence." A third copy therefore lives in the skill.

- **Fix:** one home. Either a declared bullet under `Documentation changes` with
  an `item-pattern`, or a section of its own.
- **Outcome:** _(to record)_

### D3 — BUILD-time findings are captured by three sections across two documents

| Document | Section | Schema wording |
| -------- | ------- | -------------- |
| issue | `Discoveries` | "BUILD-time behavioural, scope, architectural, security, or ambiguous discoveries … Unresolved items remain unchecked; SCOPE ticks resolved items" |
| plan | `Implementation decisions` | "Concise BUILD-time local choices with block, reason, and affected paths, or none." |
| plan | `Follow-up issues` | "Links to separately filed unrelated work, or none." |

No rule says which of the three takes a given BUILD-time finding. The boundary is
implied — discovery vs. decision vs. unrelated work — but never stated, and each
schema describes only its own half.

Usage confirms the ambiguity: 1 issue across all 94 carries a `Discoveries`
section; 8 plans out of 30 carry a non-`none` `Implementation decisions`.

- **Fix:** state the boundary once, in whichever schema is the natural home, and
  have the other two reference it.
- **Outcome:** _(to record)_

### D4 — Index summaries duplicate frontmatter descriptions, and the stated equality rule is false

`schema-schemas-index` declares:

> The summary matches the schema's own frontmatter description, so a change to
> one is visible as a difference from the other.

Nothing checks this. Two substantive drifts have already occurred:

**`schema-plan`** — the index still describes the pre-workflow plan:

- index: "the one design document a piece of work carries — its goal, the
  contract changes it makes, its work blocks with their cases, and the decisions
  deferred to the user — filed among the plans."
- frontmatter: "The canonical implementing record for one issue, carrying
  approved scope, stable work blocks, durable BUILD state, decisions, evidence,
  and acceptance."

**`schema-research`** — the index cites a skill that does not exist:

- index: "…derived from the research skill and the SOKF spec."
- frontmatter: "…derived from the SOKF spec."

Beyond those two, all 34 remaining entries differ from their frontmatter by
leading-letter case ("architecture decision records" vs "Architecture decision
records"), because the index style lowercases after the dash. So the rule as
written is false for every single entry, which is why a reader stopped believing
it long enough for the two real drifts to land.

- **Fix:** either enforce equality in the validator and normalise the case, or
  restate the rule honestly ("summarises", not "matches") and accept that it
  binds nothing.
- **Outcome:** _(to record)_

### D5 — Eight schemas assert a reports index that does not exist

These eight each carry the sentence "listed in that directory's index":

`code-review`, `idea`, `investigation`, `migration-guide`, `postmortem`,
`release-notes`, `security-review`, `status-update`

`knowledge/reports/index.md` does not exist. Seven of the eight file into
`knowledge/reports/`, so seven copies of the claim are false. (`idea` files into
`knowledge/ideas/`, which does have an index, so that one is true.)

- **Fix:** create `knowledge/reports/index.md`, or strike the claim from the
  seven schemas. Either way the sentence should have one home, not seven.
- **Outcome:** _(to record)_

### D6 — Boilerplate frontmatter is repeated 35 times, and the mechanism built to stop it is unused

Thirty-five of the 38 schemas declare a `frontmatter:` block, and every one of
the 35 re-declares:

```yaml
  title:
    required: true
  description:
    required: true
```

(The three exceptions — `changelog`, `readme`, `schemas-index` — are the
glob-targeted schemas, whose governed documents carry no frontmatter at all.)

Twenty-one of the 35 additionally re-declare, verbatim:

```yaml
  status:
    enum: [draft, stable, deprecated]
```

A twenty-second, `adr`, declares a deliberately different `status` enum
(`[draft, stable]`, no `deprecated`, because an ADR's retirement is carried by
`lifecycle` instead). That one is a real per-artifact constraint and belongs
where it is; the other 21 are copies.

The grammar is explicit that this should not happen. From
`.agents/sokf/grammar.yaml`, `kinds.schema.document.keys.frontmatter`:

> The governed document's frontmatter contract, one entry per key specific to
> this artifact. Keys every concept carries — title, description, tags, links —
> belong to the tool, not to nine repeated copies here.

There are 35 copies of `title` and `description`, and 21 of `status`.
`schema-fragment` exists to give shared bodies one home through include blocks,
and is used zero times (see U1).

- **Fix:** either lift the universal keys into the tool as the grammar already
  says, or materialise them from a fragment. Note that lifting them into the tool
  is the grammar's stated intent and needs no new mechanism; adopting a fragment
  would put generated YAML inside a fenced contract block, which may not survive
  the include machinery.
- **Outcome:** _(to record)_

### D7 — The whole schema set exists twice, and 36 of the 39 pairs are unguarded

`pack/knowledge/schemas/` contains all 39 files, **byte-identical** to
`knowledge/schemas/`, tracked in git. `pack/` is the template superdev writes
into other repositories, so a second copy is deliberate — the mirror is by
design, and `contract_standard.rs` names it as such: "in the live tree and in the
pack mirror (ADR-043)".

What is not by design is the guard. Exactly **three** pairs are asserted equal:

| Schema | Test | Assertion |
| ------ | ---- | --------- |
| issue | `normative_shapes.rs:196` | `assert_eq!(copies[0].1, copies[1].1, "the pack copy differs from the live one")` |
| plan | `normative_shapes.rs:577` | same |
| contract | `contract_standard.rs:29` | prose-prefix equality across both copies |

The other **36** — including `adr`, `documentation`, `idea`, `readme`,
`changelog`, every knowledge concept and every report schema — have no parity
test. Nothing copies them either: there is no sync command, no build step, and
`pack/` is outside the validator's roots (`.agents`, `.claude/skills`,
`knowledge/schemas`). They are identical today because someone edited both by
hand, and they will diverge the first time someone does not.

This also doubles the cost of every other finding here. Fixing D5's
reports-index claim means editing 14 files, not 7. Fixing D6's boilerplate means
70 blocks, not 35.

- **Fix:** generate `pack/knowledge/schemas/` from `knowledge/schemas/` at build
  time, or assert the whole directory equal in one test rather than three files
  at a time. The three existing assertions are the pattern; they just need to be
  a loop over the directory.
- **Outcome:** _(to record)_

---

## 2. Unused here, and what that does and does not tell us

No schema in the set is unnecessary on the evidence gathered. Each governs a
document class a project plausibly produces, and the set is generic by design.
What the instance counts do tell us is which schemas have never been exercised —
and an unexercised schema is an unreviewed one, because a schema is only tested
by documents written against it.

### U1 — `schema-fragment`: works, was used in production, and is idle because its one consumer stopped needing it

The mechanism is not unproven. Git history:

| Date | Commit | Event |
| ---- | ------ | ----- |
| 2026-09-01 | `9c57089` | `schema-fragment`, `fragments/contract-style.md` and `tests/fragments.rs` land together. **Seventeen** contract-kind schemas materialize the standard through `<!-- sokf:include contract-style -->`. |
| 2026-09-01 | `3eac9b0` | The fragment is revised; every carrier picks up the change. |
| 2026-09-02 | `27cb9a4` | Revised again, +39 lines, still carried by all 17. |
| 2026-09-02 | `8d9accd` | ADR-043 collapses the 17 contract-kind schemas into one. With a single carrier, a shared body has nothing to share with, so the fragment is inlined into `contract.md` and deleted. |

So it worked, at its widest span — one body, seventeen carriers, kept in step by
`--fix` across two revisions. It became idle for a structural reason that has
nothing to do with the mechanism: **the duplication it solved was eliminated by
ADR-043**, which replaced sixteen near-identical schemas with one schema and
twelve variants. A fragment with one carrier is just prose in the wrong file,
and inlining it was correct.

The concept-include code path is live and tested. `cargo test -p superdev-core
--lib include` runs 20 tests, including `a_materialized_include_block_passes`,
`a_stale_include_block_is_an_error_naming_the_id`,
`including_a_concept_that_includes_is_refused`, and
`materialize_fills_a_source_include_and_leaves_a_concept_include_as_it_was`. All
pass.

The assertion in `contract_standard.rs` that
`knowledge/schemas/fragments/contract-style.md` does not exist is not a verdict
on fragments. It is a regression guard for ADR-043's outcome: the standard must
be prose in the one schema, so "no fragment remains to drift from" — a drift
guard against the *old* arrangement returning, not a ban on the mechanism.

What remains true and worth deciding:

- `schema-fragment` is missing from `knowledge/schemas/index.md` (O1), which is
  how it reads as abandoned when it is merely idle.
- D6 is the next genuine candidate: 35 copies of the same frontmatter block is
  exactly the shape `contract-style` had across 17 carriers. The open question
  there is mechanical, not conceptual — whether an include block can sit inside
  a fenced ````yaml```` contract, which `contract-style` never had to do because
  it was prose above the fence.

- **Outcome:** _(to record)_

### U2 — `schema-visual-system`: correct to keep; superdev is the wrong project to validate it

Keeping it is right. Design tokens are a real artifact class, and a schema set
that covers twelve contract kinds but not a UI's palette would have a hole in it.
The schema's own preamble scopes itself honestly: "Only projects with a UI carry
this concept."

The finding is that superdev cannot validate it. This repository has no UI, so
`visual-system` has never been written against and its example is the only
document that has ever satisfied it. That is true of six other schemas too
(U3), and it is the general risk in shipping a template set from a project that
exercises a third of it.

- **Note:** `schema-contract`'s `ui` variant requires a `Visual system` level-3
  section "by reference", so the two are already wired together. That reference
  is also untested.
- **Outcome:** _(to record)_

### U3 — Seven zero-instance report schemas, 944 lines; four lost their producer in the Pi migration

| Schema | Lines | Instances | Last touched |
| ------ | ----- | --------- | ------------ |
| security-review | 140 | 0 | 2026-08-31 |
| investigation | 121 | 0 | 2026-08-31 |
| postmortem | 162 | 0 | 2026-08-31 |
| status-update | 122 | 0 | 2026-08-31 |
| release-notes | 121 | 0 | 2026-08-31 |
| migration-guide | 157 | 0 | 2026-08-31 |
| visual-system | 121 | 0 | 2026-09-04 |

Nothing produces them **now** — but four of them did, and the producers were
deleted rather than the schemas. The archived Claude Code skills called them
directly:

```
archive/.../live-build/SKILL.md:23   sokf_read sokf:schema-migration-guide  when="if a contract change breaks users"
archive/.../live-build/SKILL.md:46   step WRITE INVESTIGATION  "per `schema-investigation`"
archive/.../live-accept/SKILL.md:20  sokf_read sokf:schema-security-review  when="if the change touches auth…"
archive/.../live-accept/SKILL.md:26  step WRITE FINDINGS  "per `schema-code-review`"
archive/.../knowledge/build/SKILL.md   — same two as live-build
archive/.../knowledge/accept/SKILL.md  — same two as live-accept
```

Those skills were replaced by the Pi extension. The replacement prompts —
`.pi/extensions/superdev/prompts/{scope,build,accept,code-review,
requirements-review,orchestrator}.md` — reference **no schema at all**, not one.
So this is not speculative content that was never wired up; it is content whose
wiring was cut during the Pi migration and never reconnected or removed.

That changes the question. It is not "should we have built these?" but "did the
Pi migration intend to drop investigation write-ups, security reviews and
migration guides, or did it lose them?"

`postmortem`, `status-update`, `release-notes` and `visual-system` had no
producer even in the archived skills, and remain speculative on the original
reading.

Separately, grepping for every schema id across `.pi/`, `.claude/`, `.agents/`
and `crates/` returns hits for only four ids: `schema-contract`, `schema-issue`,
`schema-plan` (all in tests and `scope/SKILL.md`) and `schema-readme` (in
`validate/mod.rs`). Nothing else in the live tree names a schema.

All six non-visual-system entries date from 2026-08-31, before the
SCOPE → BUILD → ACCEPT workflow landed (`b38646a feat(workflow): install
canonical record schemas`, `6def09b feat(workflow): orchestrate scope build
accept with Pi`). None has been reconciled against the workflow since.

944 lines in total, or 823 excluding `visual-system`, which is already covered
separately by U2. This is the largest single block of speculative content in the
schema set — 18 per cent of all 5,151 lines — and the project's own rule is
"Apply KISS and YAGNI; build only what is requested."

- **Options:** the schemas themselves should stay — postmortems, status updates
  and release notes are ordinary project artifacts. What needs deciding is the
  **producer**, and for the four with archived producers, whether losing the
  capability in the Pi migration was intentional. A schema no skill invokes is a
  schema no project will discover.
- **Outcome:** _(to record)_

### U4 — `schema-code-review`: 11 instances, none of them the workflow's code review

The 11 documents under `knowledge/reports/` are hand-written design reviews of
this project's own schema and contract work — `code-review-004-contract-design-review`,
`code-review-009-a-contracts-behaviour-is-written-as-ears`, and so on.

The workflow's actual final code review is described in `.agents/superdev.md` as
"fresh, isolated, read-only, and bound to immutable candidate H". Its output goes
through `superdev_submit_result` as structured findings —
`.pi/extensions/superdev/prompts/code-review.md` requires "a stable ID;
classification `correctable-within-scope` or `requires-scope`; summary; evidence;
impact; dependencies" — and its outcome lands in the plan's `Completion
evidence`. No markdown document is written, and the prompt names no schema.

So the schema governs a document class the workflow does not produce, while the
review the workflow does produce has a structured finding shape defined only in a
prompt and in TypeScript, and an outcome governed only by `schema-plan`'s
one-line prose description of `Completion evidence`. The archived
`accept/SKILL.md` did write a `schema-code-review` document (`step WRITE FINDINGS
"per schema-code-review"`); the Pi replacement does not.

- **Question to settle:** the schema should stay — a written code review is an
  ordinary artifact. What to settle is whether the *workflow's* review should
  also land as a `CodeReview` document, or whether structured findings plus a
  `Completion evidence` line is the intended end state. If the latter, the
  schema's description should stop reading as though it governs that review.
- **Outcome:** _(to record)_

### U5 — `schema-changelog`: a schema that cannot express its own document

Its preamble states the problem plainly:

> It is the one schema here that declares no order: the change groups repeat
> *inside* each release heading, which itself repeats, and `sections` is a flat
> list keyed by level — it can say a level-3 group repeats, but not that it
> repeats once per release. The Keep a Changelog ordering … holds by convention
> until the vocabulary can nest repetition.

What it actually enforces: `line-limit: 0`, the presence of `# Changelog`,
`## [Unreleased]`, at least one `## [x.y.z] - date`, and the six-word level-3
heading vocabulary. Everything that makes a changelog a changelog — ordering,
grouping under the right release — is unbound.

- **Options:** keep as a weak guard and say so plainly in its description; or
  extend the section vocabulary to nest repetition, which would also let
  `schema-plan` express work blocks properly (D1) and `schema-contract` express
  its per-kind Behaviour groups. Dropping it is not proposed: a changelog is a
  document every project has, and a weak guard beats none.
- **Outcome:** _(to record)_

---

## 3. Schemas that do not state what must NOT be included

This is the largest systematic gap in the set.

### X1 — Enforceable exclusion is used by three schemas out of 38

The grammar supplies three exclusion mechanisms: `sections-prohibited`,
`item-prohibited-pattern`, `item-only-pattern`. Total usage across all 38
schemas:

| Schema | `sections-prohibited` | `item-prohibited-pattern` | `item-only-pattern` |
| ------ | --------------------- | ------------------------- | ------------------- |
| contract | 0 | 4 | 2 |
| issue | 1 | 0 | 0 |
| documentation | 0 | 0 | 1 |
| **all other 35** | **0** | **0** | **0** |

- **Outcome:** _(to record)_

### X2 — Four schemas carry no exclusion language at all, not even prose

A first pass on a narrow vocabulary ("must not", "never", "belongs in/to", "not
here", "lives in", "rather than") flagged thirteen schemas. Widening the search
to the forms actually used — "Distinct from…", "only what the tree cannot say",
"Outcomes, not activity", "not a finding", "nothing else" — cut that to four.
The wider result is the honest one, and is what follows.

**No boundary statement of any kind** — these four say what to include and never
what belongs elsewhere:

| Schema | Nearest thing to an exclusion | Verdict |
| ------ | ---------------------------- | ------- |
| `definition-of-done` | "checkable by someone who did not make the change" — a quality criterion | none |
| `development-procedure` | — | none |
| `development-commands` | "anything not in it is ad hoc" — sits in the *example*, not in the rules | none |
| `research` | "keys, not positions" — a mechanical note about footnote labels | none |

These four are exactly the schemas whose content overlaps most (X6): three of
them describe commands and procedure, and all three could take the same bullet.

**Structural exclusion only, no content boundary:**

- `dependency-policy` — "The document is a list and nothing else" bounds the
  shape but not the subject matter. Note that `technology-stack` points at
  `dependency-policy` ("belongs to `dependency-policy`, not here") and
  `dependency-policy` does not point back.
- `status-update` — "Outcomes, not activity" bounds one section only.

**Correctly excluded from this finding**, having been flagged by the narrow pass
and cleared by the wide one: `code-review` and `security-review` (both carry
"not a finding"; see X5 for the separate problem with how), `changelog`
("omitted, not left empty"; "the symptom fixed, not the internal mechanics"),
`schemas-index` ("the shape of an entry, not the taxonomy"), `release-notes`
("Distinct from the changelog: the changelog is the complete record, these are
what a user is told about one version"), `directory-structure` ("only what the
tree itself cannot say" — one of the better boundary statements in the set), and
`issue` (whose prose exclusions are strong; its problem is that none of them
bind — see X3).

Note that "Omit the section when empty" — which appears in `release-notes`,
`status-update`, `security-review` and `issue` — is a statement about
optionality, not about exclusion, and was not counted as one.

- **Outcome:** _(to record)_

### X3 — `schema-issue`: the prohibition is stated three times in prose and enforced zero times

The entry point of the whole workflow. Its preamble says:

> No key, no EARS tag and no `TBD` rule holds an issue: keys and EARS live in the
> contracts, whose promises carry the criteria a plan case and a test cite
> (ADR-050).

Its `Behaviour` section rule repeats it:

> No key and no EARS tag: the criteria a test binds live on the contract the work
> touches.

Its frontmatter `description` repeats it a third time:

> …and no key or EARS tag on any of them.

Nothing binds any of it. `schema-contract` demonstrates the exact pattern
needed — `item-prohibited-pattern: '\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b|…'` —
and a one-line `item-prohibited-pattern` on `Behaviour` and `Scope` matching
`` `P_… `` / `` `AC_… `` / `\[(ubiquitous|event|state|conditional|optional|complex)\]` /
`\bTBD\b` would close it.

The one `sections-prohibited` entry the schema does carry covers `Resolution`
under `lifecycle: open` only.

- **Outcome:** _(to record)_

### X4 — `schema-plan` states nothing a plan must not contain

351 lines and no prohibition. Specifically absent:

- a plan must not restate the issue's problem statement (the `Goal and
  boundaries` and `Requirements` sections invite it);
- a plan must not carry contract promise text — `P_`/`AC_` keys belong to
  contracts, and the plan cites them;
- a plan must not carry `TBD`;
- a plan must not carry `- manual:` evidence (enforced in Rust only — see D1).

`.pi/extensions/superdev/skills/build/SKILL.md` states the live constraint —
BUILD "may not add requirements, architecture, application programming interface
(API) changes, contracts, or acceptance criteria that the approved plan does not
contain" — but that is a rule about the actor, not about the document, and it
lives in a skill rather than in the schema.

- **Outcome:** _(to record)_

### X5 — Neither review schema binds what is not a finding

`schema-security-review` states it in prose, in the preamble:

> no realistic path from attacker-controlled input to impact means the entry is
> informational, not a finding.

`schema-code-review` states it parenthetically, inside a section description:

> if you can't construct one, it's probably not a finding

That hedge ("probably") breaks the project's own grammar rule 19, "No hedging",
and rule 16, "Avoid contractions in formal specs". Neither schema expresses the
rule in a form the validator can check, though `item-pattern` on the finding
bullets could require the `Failure scenario:` / `Attack scenario:` line.

- **Outcome:** _(to record)_

### X6 — Knowledge-concept schemas: boundaries stated well in three, absent in the rest

The 21 singleton concept schemas describe inclusion carefully and mostly say
nothing about where the neighbouring content goes.

Doing it right, and worth using as the model:

- `technology-stack`: "Whether a dependency may be added at all belongs to
  `dependency-policy`, not here."
- `testing-strategy`: "This is the standing strategy; the per-feature cases live
  in the plan's work blocks."
- `idea`: "It is not candidate work, so it does not belong in the issue tracker."
- `architecture`: per-layer "what it owns and what it must not know about".

Carrying adjacent content with no boundary statement:

- `configuration` vs `technology-stack` vs `directory-structure` — three schemas
  describing files in the tree, none saying which takes what.
- `error-handling` vs the `cli` and `library` contract variants, both of which
  require an `Exit codes` / `Errors` section.
- `software-components` vs `architecture` vs `directory-structure`.
- `development-commands` vs `development-procedure` vs `definition-of-done` —
  all three describe commands that must pass before a PR.

- **Outcome:** _(to record)_

---

## 4. Other findings

### O1 — Two schemas are missing from the schemas index

`knowledge/schemas/index.md` lists 36 entries. 38 schemas exist. Absent:

- `schema-documentation` — the Documentation Map schema, read by all three
  workflow phases
- `schema-fragment`

`schema-schemas-index` describes the index as "The listing of every schema".
`superdev validate` passes.

- **Outcome:** _(to record)_

### O2 — `knowledge/documentation.md` is missing from `knowledge/index.md`

The concept exists (4,785 bytes), is governed by `schema-documentation`, and is
described by its own schema as "the reviewed boundary used by SCOPE to identify
affected documentation and by BUILD and ACCEPT to require executable evidence".
It appears nowhere in the root index, so an agent browsing the index does not
learn it exists.

- **Outcome:** _(to record)_

### O3 — `knowledge/reports/` is missing from `knowledge/index.md`

Eleven documents, no index entry, and no `index.md` of its own (see D5). The root
index lists `ideas/`, `plans/`, `adrs/`, `contracts/`, `issues/`, `schemas/` and
`research/` — reports is the only populated subdirectory omitted.

- **Outcome:** _(to record)_

### O4 — `line-limit: 800` on 32 of 38 schemas is a default, not a judgement

Distribution: 800 × 32, 1500 × 2 (`plan`, `documentation`), 4000 × 1 (`contract`),
400 × 1 (`idea`), 200 × 1 (`fragment`), 0 × 1 (`changelog`).

The six non-800 values are all clearly reasoned: `changelog` at 0 because its
length is inherent, `contract` at 4000 because of the twelve variants, `idea` at
400 because capture cost governs capture rate. The 32 at 800 include
`definition-of-done` (a bulleted list that will never reach 100 lines) and
`glossary` (which grows without bound as the domain does). A limit nobody derived
is a limit nobody will respect when they hit it.

- **Outcome:** _(to record)_

### O5 — `schema-issue`'s `Resolution` rule reads as unconditional

Two entries govern the same heading:

```yaml
  - heading: "Resolution"
    required: true
    variants: [done, wontfix]
sections-prohibited:
  - heading: "Resolution"
    variants: [open]
```

Correct under the variant semantics, but `required: true` sitting beside
`variants:` reads as "always required" unless the reader already knows that a
`variants` tag scopes the whole rule. Worth a one-line note in the description,
or a grammar-level convention.

- **Outcome:** _(to record)_

### O6 — Schema-to-schema duplication is not checked

`.agents/sokf/grammar.yaml` declares `duplication.crossPairs` as `unit|unit` and
`unit|schema`. There is no `schema|schema` pair and no `withinFileKinds` entry
for `schema`. So the 38 copies of the frontmatter boilerplate (D6), the seven
copies of the reports-index claim (D5), and the three copies of the no-EARS rule
inside `schema-issue` (X3) are all invisible to the duplication checker by
construction.

- **Outcome:** _(to record)_

### O7 — Thirty-four of 38 schemas predate the workflow and have never been reconciled with it

The SCOPE → BUILD → ACCEPT workflow landed across seven commits:

```
b38646a 2026-09-06 feat(workflow): install canonical record schemas
6def09b 2026-09-06 feat(workflow): orchestrate scope build accept with Pi
c83b4c7 2026-09-06 fix(workflow): repair closure and migration safety
864baeb 2026-09-06 fix(workflow): serialize canonical transitions
26662c7 2026-09-07 fix(workflow): preserve same-plan rescope journeys
0917d69 2026-09-09 chore: pi workflow improvements
6e8bf99 2026-09-09 fix(workflow): let BUILD commit its own blocks
```

Only **four** schemas have been touched on or since 2026-09-06:

| Schema | Last touched | By |
| ------ | ------------ | -- |
| documentation | 2026-09-06 | `864baeb` |
| research | 2026-09-06 | — |
| issue | 2026-09-07 | `26662c7` |
| plan | 2026-09-09 | `6e8bf99` |

The other **34** last changed on 2026-09-04 or earlier. `contract` — which the
workflow reads in every phase — last changed 2026-09-04, before the workflow
existed. `readme` last changed 2026-08-28.

Some genuinely did not need to change. Three did and did not:

- **`development-procedure`** (2026-09-04) describes a workflow of "a change
  starts as an issue … Branch from `main` as `feature/{slug}` … `just check` must
  pass before the PR". The live workflow files an issue, opens a plan, works on
  `work/{nnn}-{slug}`, and gates on phase transitions. The schema's `Workflow`
  section rule still describes the replaced process.
- **`definition-of-done`** (2026-08-31) describes "One reviewer has approved it,
  and every review thread is resolved". The live gate is ACCEPT against an
  immutable candidate.
- **`issue-tracker`** (2026-09-04) describes filing and triage labels, with no
  mention of the work branch the issue id determines.

- **Outcome:** _(to record)_
### O8 — The pack ships two of superdev's own concepts as if they were templates

`pack/knowledge/concepts/` is the concept skeleton superdev writes into a new
repository. Nineteen of its 21 concepts are correctly written as skeletons —
`architecture.md` reads "TBD: the parts and the data flow between them",
`glossary.md` carries 2 TBDs, and so on.

Two are not skeletons. They are superdev's own content, byte-identical to the
live concept:

| File | TBDs | Evidence it is not a template |
| ---- | ---- | ----------------------------- |
| `documentation.md` | 0 | "prospective and current users installing and operating **Superdev**"; `cargo run -- validate --fix`; `/crates/app/superdev/src/main.rs`; `contract-002-cli-superdev` |
| `issue-tracker.md` | 1 | `superdev validate --fix` three times |

A project adopting the pack receives superdev's documentation map, naming
superdev's crates and superdev's contracts, as its own.

- **Fix:** skeletonise both, as the other 19 are.
- **Outcome:** _(to record)_

### O9 — Three test assertions are vacuous because the root they scan does not exist

`crates/lib/superdev-core/tests/normative_shapes.rs` scans `pack/knowledge/skills`
at lines 1356, 1421 and 1795. That directory does not exist — `pack/knowledge/`
contains only `concepts/` and `schemas/`.

The helper swallows it silently:

```rust
if path.is_dir() {
    let Ok(entries) = std::fs::read_dir(path) else { return; };
```

`is_dir()` returns false for a missing path, so the walk does nothing and the
assertion passes on an empty result. The skills moved to
`pack/pi/extensions/superdev/skills/` — which those tests do not scan — so
`nothing_names_a_retired_kind_schema` and `nothing_names_the_backlog` are not
checking the pack's skills at all.

- **Fix:** point the roots at `pack/pi/extensions/superdev/skills`, and make a
  missing root an error rather than a silent pass.
- **Outcome:** _(to record)_

---

## 5. Does this information have a better-structured home?

The question behind the review. `architectural-rules` reads as a list of
unrelated requirements, and the reason is that it is one: six bullets with six
different natural homes, gathered under a heading that describes none of them.

### The six rules, and where each belongs

| # | Rule | Kind of statement | Better home | Status there |
| - | ---- | ----------------- | ----------- | ------------ |
| R1 | Components observe and plan; they never change anything | Module boundary | `interface` contract for the pipeline | **No such contract exists** |
| R2 | The engine is the only side-effect site for the repo | Module boundary | same | Already restated in `contract-007` prose, unkeyed |
| R3 | A mise-pinned tool is invoked through `mise exec` | Implementation convention | `coding-standards`, or an `interface` contract for the spawn seam | Bound nowhere |
| R4 | Every mise command names the tools superdev manages | Implementation convention | same | Bound nowhere |
| R5 | Capability names are user-facing; provider names are not | Term definition | `glossary` | Already there — and it links *back* here |
| R6 | Domain logic lives in `superdev-core` | Component responsibility | `software-components` | Already there: "All domain logic; no argument parsing" |

Three of the six are already stated elsewhere. Two are stated nowhere else and
are not bound by anything. One has no home to go to.

### Why an `interface` contract is the better home for R1, R2 and R6

The schema set already has a document type built for exactly this. From the
schemas index: "An `interface` contract is internal — keyed to a module
boundary, updated as features change it." Its required level-3 sections are:

- **Module boundaries** — "What may call what across the boundary, and what must not."
- **Key flows** — "The sequences that cross the boundary, in order."
- **Cross-cutting concerns** — "Security, performance, migration, observability, each as a promise the module keeps."

That is R1, R2 and R6, described by the schema that was designed to hold them.
And the difference is not cosmetic. A contract promise is:

- **keyed** — `P_component-calls-no-pack`, stable across rewordings;
- **EARS-tagged** — one modal verb, one requirement, checkable shape;
- **bound by a test** — the contract standard requires it: "The project MUST
  bind each Behaviour promise by a test of the behaviour it promises";
- **citable** — a plan case or a test doc comment names the key.

An `architectural-rules` bullet is none of those. R1 and R2 are load-bearing —
`status` and `--dry-run` are free *because* planning is pure — and they are
currently guarded by prose alone.

`contract-007` demonstrates the alternative in the same repository. It carries
six keyed module-boundary promises, including `P_component-reads-ctx` and
`P_component-calls-no-pack`, which are R1 restricted to the pack seam.

**The gap:** three internal contracts exist, covering `src/pack`,
`src/validate/schema/document.rs` and `src/workflow`. Nothing covers the
`pipeline` → `plan_repo` → `engine` path that R1, R2 and R6 are about. R2
appears there only as unkeyed prose inside `contract-007`'s Module boundaries
narrative — "`engine` stays the only side-effect site for repo writes" — which
is a restatement, not a binding, and sits inside a contract whose subject is
pack resolution rather than the engine.

### The circularity

`glossary` defines Capability and Provider well, then defers upward:

> Capability names are what users type; see [architectural-rules].

`architectural-rules` states R5, which is the same fact. Two documents, one
fact, and the arrow points from the better-structured one to the looser one.
`contract-004-config-superdev` already binds the enforcement —
`P_unknown-provider-fails`, `P_removed-capability-fails` — so the naming rule
has a home with a keyed promise beside it.

### What should be left

If R1, R2 and R6 move to an interface contract, R3 and R4 to `coding-standards`
or a spawn-seam contract, and R5 collapses into `glossary`, nothing remains.

That is the honest answer: **`architectural-rules` as currently written has no
residue.** Every bullet is either a module boundary, a convention, or a
definition, and each of those has a purpose-built home with stronger structure.

Two ways to read that:

1. **The concept is redundant for this project.** superdev's architecture is
   small enough that its invariants fit inside the contracts that bind the
   modules. Delete the concept, move the six, and let `architecture` link the
   contracts.
2. **The concept is the index, not the content.** Keep it as the one-page map of
   which invariants exist and which contract binds each — no reasoning, just the
   rule and the key. It stays useful for a reader orienting, and stops competing
   with the contracts for authority.

Option 2 is the safer change and the one I would propose. Option 1 is cleaner
but breaks three ADRs that cite `architectural-rules` as the authority —
ADR-002 opens "[architectural-rules] settles what may happen where" — and those
citations are the evidence that the concept has been doing real work, whatever
its current state.

The generic schema is not the problem either way. `schema-architectural-rules`
asks for "the rule stated as an invariant that code review can enforce, followed
by the reason it holds", which is a reasonable request. Most projects have no
contract layer to move these into; superdev does, which is why the overlap shows
up here and would not elsewhere.

### `architecture` — a different problem

`architecture` is not miscategorised. Its four sections match the schema, and
"Serving the canonical knowledge", "Content resolves before planning" and
"Capabilities and providers" are genuinely architecture. Three narrower issues:

**A1 — The capabilities table is malformed.** Lines 83–88 of
`knowledge/architecture.md`:

```
| Capability   | Provider          | Delivered as                    |
|--------------|-------------------|---------------------------------|
                                        ← blank line
| `code-index` | `codegraph`       | … |
| `frontend`   | `frontend-design` | … |
```

A blank line between the separator and the body ends the table. Every renderer
shows a two-row empty table followed by two lines of literal pipe text. Nothing
catches it: `schema-architecture` declares these subsystem headings as
`content: prose`, so no `content: table` or `columns:` rule applies.

**A2 — The table is missing a row.** `Capability::ALL` in
`crates/lib/superdev-core/src/capability.rs` has three members — `Frontend`,
`Skills`, `CodeIndex`. The table lists two. `skills` has no row, despite
carrying a registry entry of its own: `provider: "superdev-skills"`,
`Cardinality::Many`, version pinned to the crate version
(`crates/lib/superdev-core/src/registry.rs:62`). It is not an omission of
something unbuilt; it is the one capability whose provider ships inside the
binary.

**A3 — A third count disagrees.** `software-components` line 21 says
"`capability` — the four slots". The code says three, the glossary names three,
the table shows two. Three documents, three numbers, none of them checked.

**A4 — Overlap with `contract-003-api-sokf`.** "Serving the canonical knowledge"
is 35 lines, the longest section in the document, and describes MCP tools,
resolution order, and Pi's routing behaviour in operational detail. The
contract that binds that surface is `contract-003-api-sokf`. The schema asks for
"what it does, how it talks to the rest, and **where its detail lives**" — the
section is carrying the detail rather than pointing at it.

- **Outcome:** _(to record)_

---

## 6. Recommended actions, in value order

Each is a proposal, not a decision.

1. **Move the plan work-block contract into `schema-plan`** as `item-pattern` and
   `item-prohibited-pattern`; strip `sokf.rs` and `fix.rs` back to what the
   schema vocabulary cannot express. Closes D1 and part of D2. _(Outcome: )_
2. **Add `item-prohibited-pattern` to `schema-issue`** for `P_`/`AC_` keys, EARS
   tags and `TBD`. The rule is already written three times in prose. Closes X3.
   _(Outcome: )_
3. **Name the producer for each unexercised report schema** (U3). The schemas
   stay — they are generic and the artifact classes are ordinary. What is
   missing is the skill that invokes them, and for `migration-guide`,
   `investigation`, `security-review` and `code-review` the prior question is
   whether the Pi migration meant to drop the capability. _(Outcome: )_
4. **Create `knowledge/reports/index.md`, or strike the claim** from the seven
   schemas that assert it. Closes D5. _(Outcome: )_
5. **Add the two missing schemas to `knowledge/schemas/index.md`**, and add
   `documentation` and `reports` to `knowledge/index.md`. Closes O1, O2, O3.
   _(Outcome: )_
6. **Fix the two substantive index drifts** (`schema-plan`, `schema-research`),
   then either enforce the equality rule in the validator or restate it as
   "summarises". Closes D4. _(Outcome: )_
7. **Add one boundary sentence to every schema with no exclusion language**,
   starting with the four in X2 and the overlapping concept schemas in X6.
   _(Outcome: )_
8. **Settle the `Discoveries` / `Implementation decisions` / `Follow-up issues`
   boundary** and state it once. Closes D3. _(Outcome: )_
9. **Decide how the repeated frontmatter block is shared** — lift the universal
   keys into the tool as the grammar intends, or materialize them with a
   fragment. The fragment mechanism is proven and stays either way; the only
   open question is whether an include block works inside a fenced contract.
   Closes D6 and U1. _(Outcome: )_
10. **Add `schema|schema` to `duplication.crossPairs`** so findings of this shape
    surface without a manual review next time. Closes O6. _(Outcome: )_
11. **Derive each `line-limit`, or document 800 as a deliberate default.** Closes
    O4. _(Outcome: )_
12. **Reconcile `development-procedure`, `definition-of-done` and `issue-tracker`
    with the SCOPE → BUILD → ACCEPT workflow**, and audit the remaining 31
    pre-workflow schemas for the same problem. Closes O7. _(Outcome: )_
13. **Settle whether the workflow's review lands as a `CodeReview` document**
    (U4), and whether `schema-changelog` is documented as a weak guard or given
    nested repetition (U5). _(Outcome: )_
14. **Guard the pack mirror** — generate it, or assert the whole directory equal
    instead of three files. 36 of 39 pairs are currently unguarded, and this
    doubles the edit cost of items 2, 4, 5, 6, 7 and 9. Closes D7. _(Outcome: )_
15. **Skeletonise `pack/knowledge/concepts/documentation.md` and
    `issue-tracker.md`**, which currently ship superdev's own content to adopting
    projects. Closes O8. _(Outcome: )_
16. **Repoint the three vacuous test roots** at
    `pack/pi/extensions/superdev/skills`, and fail on a missing root. Closes O9.
    _(Outcome: )_
17. **Add what a plan must not contain to `schema-plan`** — no restated issue
    problem statement, no `P_`/`AC_` promise text, no `TBD` — alongside the
    `manual:` prohibition item 1 moves there. Closes X4. _(Outcome: )_
18. **Bind "not a finding" in both review schemas**, via an `item-pattern`
    requiring the `Failure scenario:` / `Attack scenario:` line, and delete the
    hedge in `code-review`. Closes X5. Conditional on the item 3 decision.
    _(Outcome: )_
19. **Accept that `visual-system` cannot be validated here**, and decide whether
    that is tolerable for shipped template content or whether a fixture project
    should exercise the unused portion of the set. Closes U2. `fragment` needs
    only its index entry (item 5) and the D6 decision (item 9). _(Outcome: )_
20. **Note the variant scoping on `schema-issue`'s `Resolution` rule**, so
    `required: true` beside `variants:` does not read as unconditional. Closes
    O5. _(Outcome: )_

**From section 5 — the `architectural-rules` question:**

21. **Open an `interface` contract for the pipeline/engine boundary**, and move
    R1, R2 and R6 into it as keyed, test-bound promises. This is the structural
    answer to the question: the document type exists, its three required
    sections are exactly these statements, and a promise is bound where a bullet
    is not. _(Outcome: )_
22. **Rehome R3 and R4** — the two mise conventions — to `coding-standards`, or
    to a spawn-seam interface contract alongside ADR-015. They are bound nowhere
    today. _(Outcome: )_
23. **Collapse R5 into `glossary`** and reverse the reference, which currently
    points from the better-structured document to the looser one. _(Outcome: )_
24. **Decide what `architectural-rules` becomes** once 21–23 land: deleted, or
    kept as a one-page index of invariant → binding contract key. Three ADRs
    cite it as the authority, so deletion is not free. _(Outcome: )_
25. **Fix the malformed capabilities table** in `knowledge/architecture.md`
    (A1), **add the missing `skills` row** (A2), and **reconcile the capability
    count** across `software-components` ("four"), the glossary (three), the
    table (two) and the code (three) (A3). _(Outcome: )_
26. **Move the MCP operational detail** out of `architecture`'s "Serving the
    canonical knowledge" and cite `contract-003-api-sokf` instead, as the schema
    asks (A4). _(Outcome: )_

X1 is the measurement behind items 2, 7, 17 and 18 rather than a task of its own:
it closes when those land.

---

## 7. Sequencing note

Item 14 should come first. Every other fix that touches a schema file has to be
made twice while the mirror is unguarded, and a half-applied fix is exactly the
divergence the missing parity test would not catch.

Item 3 should come second, and before items 4, 7 and 18. All four touch the
report schemas, and a boundary sentence added to a schema whose producer is then
dropped is wasted work.

Items 21–24 are one change, not four. Moving R1, R2 and R6 into a contract is
only worth doing if 24 is settled first: if `architectural-rules` survives as an
index, the move is a restructure; if it does not, the move is a migration and
the three citing ADRs need repointing in the same pass.

Item 25 is independent and cheap — a rendering defect and a count disagreement,
fixable now.

---

## 8. Coverage

Thirty-seven findings, all with an action item:

| Section | Findings | Actions |
| ------- | -------- | ------- |
| 1. Duplication | D1–D7 | 1, 4, 5, 6, 8, 9, 14 |
| 2. Unexercised | U1–U5 | 3, 9, 13, 19 |
| 3. Missing prohibitions | X1–X6 | 2, 7, 17, 18 |
| 4. Other | O1–O9 | 5, 10, 11, 12, 15, 16, 20 |
| 5. Rehoming | R1–R6, A1–A4 | 21, 22, 23, 24, 25, 26 |

---

## 9. Corrections to the first draft

Recorded so the reasoning is auditable.

- **Section 2 was rewritten.** The first draft judged `fragment`, `visual-system`
  and seven report schemas "unnecessary" on the evidence that superdev carries no
  instances. The schemas are generic and ship to any managed project, so instance
  count in this repository is not evidence of redundancy. No schema is now
  proposed for deletion; the finding is that much of the set has never been
  exercised, which makes it unreviewed rather than unwanted.
- **U1 was corrected twice.** The first draft called `schema-fragment` "a
  mechanism with no instances"; the second still described the concept-include
  path as untested and asked whether it had been abandoned. Both were wrong. The
  fragment shipped 2026-09-01 with seventeen carriers, survived two revisions,
  and was inlined on 2026-09-02 when ADR-043 collapsed those seventeen schemas
  into one. Twenty concept-include tests pass today. It is idle because its
  duplication was designed away, not because it failed.
- **Section 5 was added.** The first draft observed that `architectural-rules`
  was a loose list but did not answer whether the content had a better home. It
  does, for all six bullets.
- **X2 was corrected** from twelve schemas to four; the first search pattern
  missed boundary statements phrased as "Distinct from…", "only what the tree
  cannot say" and "not a finding".
- **U3 line count** corrected 851 → 944. **D6 repetition count** corrected 38 →
  35. **O7 staleness count** corrected 25 → 34, after using the wrong cutoff
  commit date.

---

## 10. Second review — verification, corrections, and the vocabulary ceiling

Added 2026-09-11 by a second reviewer. Method: every structural claim below was
checked by mutating a real document or schema, running `cargo run -- validate`,
and restoring the tree. Findings are quoted as the validator emitted them. The
tree was left clean; `git status` shows only this file.

Baseline drift worth noting: section 0 records 240 documents against 38 schemas.
The tree now reports **243 documents, 279 concepts, 38 schemas**, because
`plan-078` landed mid-review. The schema figures — 38 schemas, 39 files,
5,151 lines, byte-identical pack mirror — all reproduce exactly.

### 10.1 What stacks up

Verified, with the check that confirmed each:

| Finding | Verified how | Verdict |
| ------- | ------------ | ------- |
| D1 | `sokf.rs:333-375` carries the seven bullets, the Done checkbox and the `manual:` ban; `schema-plan` carries `content: bullet-list` and nothing else | **Holds** |
| D4 | `schema-plan` index entry vs frontmatter differ substantively; `schema-research` index still cites "the research skill", frontmatter does not | **Holds, verbatim** |
| D5 | `ls knowledge/reports/index.md` → no such file | **Holds** |
| D7 | `diff -rq knowledge/schemas pack/knowledge/schemas` → empty; three parity assertions for 39 pairs | **Holds** |
| O1 | Index lists 36 ids; 38 exist; missing are exactly `schema-documentation` and `schema-fragment` | **Holds, exactly** |
| O3 | No `knowledge/reports/index.md`, no root index entry | **Holds** |
| A1 | Blank line between separator and body at `architecture.md:83-88` ends the table | **Holds** |
| A2/A3 | `Capability::ALL` = 3 (`Frontend`, `Skills`, `CodeIndex`); table shows 2; `software-components:21` says "four"; glossary says 3 | **Holds — four sources, three numbers** |

The corrections in section 9 are sound, and the framing note is right: instance
count in this repository is not evidence about a generic schema. I confirmed the
consequence the framing implies but the review does not state — see M-6.

### 10.2 What does not stack up

**R-1 — Recommendation 2 cannot be implemented as written. The grammar refuses
it.**

This is the review's flagship fix for X3, and it fails on contact. `schema-issue`
declares `Behaviour` and `Scope` as `content: prose`. Adding the proposed rule:

```
knowledge/schemas/issue.md: schema yaml: sections[3].item-prohibited-pattern:
  only allowed with content: bullet-list or numbered-list
```

`item-prohibited-pattern` *requires* a list kind — the grammar says so under
`requires:`, and the checker enforces it. The obvious fallback also fails:
`content-pattern` has no negative form, and expressing "must not contain" needs
lookahead, which the dialect rejects:

```
error: look-around, including look-ahead and look-behind, is not supported
```

So **there is today no way to say "this prose section must not contain X"** —
not for the issue, not for anything. Recommendation 2 requires either converting
`Behaviour` and `Scope` to lists, which changes the artifact to suit the tool, or
a grammar addition. The review presents it as a one-line change; it is neither.

**R-2 — Recommendation 1 is only half implementable. The vocabulary has no
required-item set.**

Moving the block contract into `schema-plan` needs "all seven of `Dependencies:`,
`Areas:`, `Outcome:`, `Verification:`, `Tests:`, `Structural evidence:`,
`Documentation:` are present". The vocabulary offers `item-pattern` (*every item
matches this*), `item-only-pattern`, `item-prohibited-pattern`, `item-key`, and
`nested.required` (*at least one child*). None expresses *this set of items must
exist*. One regex cannot say it either without permutation or lookahead.

So the closed half — "only these seven labels may appear", and the `manual:`
ban — moves cleanly. The open half — "all seven must appear", which is the part
`sokf.rs` actually enforces and the part an author needs — cannot move. D1's
diagnosis is right; its fix is blocked on the same gap as R-1.

**R-3 — X1 overcounts. `documentation`'s exclusion rule binds nothing.**

The X1 table credits three schemas with enforceable exclusion. The third,
`schema-documentation`, declares:

```yaml
item-only-pattern: '^- (Audience and purpose|Kind|Authored sources|…): .+$'
```

`item-only-pattern` fires when the regex matches **outside** a top-level item.
This pattern is anchored to `^- `, so it can essentially never match a non-item
line, and the rule is a no-op. Three mutations to `knowledge/documentation.md`,
each `PASS (0 errors)`:

- an undeclared field — `- Totally Bogus Field: nonsense.` — inside a Surface;
- a declared field smuggled in as standalone prose;
- a declared field smuggled into a nested item.

The author's evident intent was "only these fields may appear", which is
`item-pattern`. As written the schema constrains neither which fields appear nor
where. **Enforceable exclusion is used by two schemas out of 38, not three** —
and `schema-documentation` should be added to X2's list of schemas with no
working boundary, since its only other candidate ("deployment is not part of the
local workflow") scopes the workflow rather than the document.

**R-4 — D6's open question is closed: the fragment route does not work.**

The review leaves this undecided ("may not survive the include machinery"). I
tested it: a fragment carrying the two universal keys, included inside a
schema's fenced ````yaml```` contract.

```
schema yaml: could not find expected ':' at line 14 column 1
schema: the contract does not deserialize — could not find expected ':' …
```

The include markers are HTML comments, which are not YAML. **Recommendation 9
collapses to its first branch**: lift the universal keys into the tool, as
`grammar.yaml` already instructs. U1's remaining question is settled too — the
fragment mechanism is sound, and simply cannot reach inside a fenced contract.

### 10.3 What the review misses — and it is the answer to the brief

**M-1 — The schema layer is open-world. Nothing is excluded by default.**

This is the finding the whole of section 3 circles without naming. Three
mutations, each `PASS (0 errors)`:

- `## Totally Undeclared Section` appended to an ADR — a schema with
  `sections-ordered: true` and a full section list;
- `bogus_key: nonsense` added to `glossary.md`'s frontmatter;
- an undeclared bullet field inside a governed Surface (R-3).

A schema states what **must** be present and what **may** be present. It has no
way to say *and nothing else* — not for sections, not for frontmatter keys, not
for list items except through the two working mechanisms. The source comment is
explicit: "other content beside the form is tolerated."

This reframes section 3 entirely. X1–X6 read as *35 authors neglected to write
prohibitions*. The truth is that **the model has no default-deny**, so every
"must not" has to be enumerated by hand, in advance, forever — and enumerating
prohibitions is unbounded work, while enumerating permissions is finite. For a
schema set whose stated purpose is "flexible but consistent", this is the
load-bearing design decision, and it is currently made by omission rather than
on purpose.

**M-2 — Nine schemas are structurally mute: the preamble has no exclusion
vocabulary at all.**

A preamble rule accepts exactly three keys:

```
schema yaml: preamble: unknown key "item-prohibited-pattern"
  (the grammar declares content, columns, description)
```

Nine schemas are preamble-only — `architectural-rules`, `definition-of-done`,
`dependency-policy`, `development-commands`, `directory-structure`, `fragment`,
`glossary`, `release-procedure`, `technology-stack`. They cannot state a
machine-checkable prohibition of any kind, however well their authors write.
Four of X2's exemplars sit in this list. Recommendation 7 asks for a boundary
*sentence*, which is right and cheap — but for these nine a sentence is the only
option the grammar permits, and the review does not say so.

**M-3 — Seventeen of 38 schemas cannot carry `item-prohibited-pattern` at all.**

It requires a list-kind **section**; a list in a preamble does not qualify.
Seventeen schemas declare no list-kind section: the nine above plus
`architecture`, `coding-standards`, `configuration`, `error-handling`,
`project-overview`, `readme`, `research`, `software-components`. The enforceable
half of recommendations 7, 17 and 18 is unavailable to 45 per cent of the set.

**M-4 — `sections-prohibited` has no pattern form.**

`struct Prohibited { heading: String, variants: Vec<String> }`, matched by `==`.
A schema may *require* a heading by pattern (`heading-pattern`) but may not
*forbid* one. `schema-plan` cannot say "no heading matching `^Block \d+:` may
appear outside Work blocks". The asymmetry is invisible in the grammar docs.

**M-5 — There is no required-concept set; omission is silent.**

I removed `knowledge/architecture.md` entirely. The validator reported 24 errors
— every one a **dangling link** from other documents. Not one said the concept
was missing. Validation is document-driven: a schema binds a document that
exists. A project adopting the pack may ship with no `testing-strategy`, no
`security-requirements`, no `definition-of-done`, and pass cleanly, provided
nothing links to them. For a set whose job is consistency across arbitrary
projects, the absence of a declared minimum is a larger hole than any individual
schema's missing prohibition — and it is invisible in this repository precisely
because superdev carries all 21 singletons.

**M-6 — "Must not" is three different problems, and the review treats them as
one.**

Separating them is what makes the brief actionable:

| Kind | Example | Mechanism | Status |
| ---- | ------- | --------- | ------ |
| **(a) Content** — this *fact* belongs elsewhere | "dependency admission belongs to `dependency-policy`" | none exists | **Prose only, permanently.** Not mechanisable; a validator cannot judge subject matter. |
| **(b) Form** — this *notation* may not appear | no `P_`/`AC_` keys, no `TBD`, no `manual:` | `item-prohibited-pattern` | Available in 21 of 38, list sections only (M-3), unusable on prose (R-1) |
| **(c) Structure** — this *heading* may not appear | no `Resolution` on an open issue | `sections-prohibited` | Available everywhere, literal headings only (M-4), used once |

The review's recommendations 2, 7, 17 and 18 mix all three. (a) can only ever be
a well-written sentence, and the model to copy is `technology-stack`'s — which
names the other document. (b) and (c) are the only ones worth spending grammar
on. Stating this split in the schemas themselves is, I think, exactly what the
brief is asking for: each schema should carry a **Boundaries** note distinguishing
"what lives elsewhere" (prose, with the target named) from "what may not appear"
(bound, where the vocabulary allows).

### 10.4 Revised recommendations

Replacing items 1, 2, 7, 9, 17 and 18, and adding four:

27. **Decide whether the schema layer is open-world or closed-world.** This
    precedes every other exclusion fix. A `sections-closed: true` flag and a
    frontmatter equivalent would convert dozens of unwritten prohibitions into
    one declaration per schema, and would make "what must not be included"
    answerable by construction rather than by enumeration. Closes M-1. _(Outcome: )_
28. **Add the missing item vocabulary** before attempting items 1 and 2: a
    required-item set (R-2), and a negative form usable on `content: prose`
    (R-1). Without these, D1 and X3 cannot close. _(Outcome: )_
29. **Give `preamble` the item declarations `section` already has**, or accept
    that nine schemas are prose-only and say so in each. Closes M-2. _(Outcome: )_
30. **Fix `schema-documentation`'s `item-only-pattern`** — it is almost certainly
    meant to be `item-pattern`, and today binds nothing. Closes R-3. _(Outcome: )_
31. **Declare a minimum concept set** the validator requires of a managed
    project, or state deliberately that every concept is optional. Closes M-5.
    _(Outcome: )_
32. **Recommendation 9 resolves to "lift into the tool"** — the fragment route is
    closed by R-4, and needs no further investigation. _(Outcome: )_

### 10.5 Overall judgement

The review is accurate where it reports and weakest where it prescribes. Its
evidence is sound — I could not falsify a single factual claim beyond R-3's
overcount — and section 5 is the strongest part of the document: the argument
that `architectural-rules` has no residue is correct, and option 2 is the right
call for the reason given.

What it lacks is a check that its own fixes are expressible. Two of the top three
recommendations are blocked on missing grammar, and the review's sequencing note
puts item 14 first when the real blocker is item 28. The deeper miss is M-1: the
brief asks for schemas that state what must **not** be included, and the honest
answer is that the current model cannot state it in general — only enumerate it,
in two of three categories, in half the set. That is a design decision worth
making deliberately, and it is the one decision the review does not surface.

- **Outcome:** _(to record)_
