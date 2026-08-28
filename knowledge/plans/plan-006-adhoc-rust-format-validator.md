---
type: AdhocPlan
id: adhoc-plan-006-rust-format-validator
title: Fold the superdev-format validator into the Rust validator
description: The grammar-driven format validator moves from a Node script into superdev-core and merges with the AOKF validator behind one command, one report and one hook, proved against goldens captured from the reference while it still ran.
status: done
---

# Plan: Fold the superdev-format validator into the Rust validator

## Context

The superdev format — the XML pseudo-language skills are written in, and the
YAML contract inside each schema — is defined by a grammar file and enforced
by a Node script that reads it. The script is the only thing that can check a
skill or a schema, so the check runs nowhere that matters: not in CI, not at
edit time, not on a contributor's machine unless they have Node. It also
overlaps the AOKF validator, which already walks `knowledge/` and already
reports findings in a shape the format check would otherwise copy. Moving the
format checks into the binary and merging the two behind one command is what
makes the format enforceable without creating a second validator to keep in
step with the first.

## Facts

- `scripts/superdev-format/validate-superdev.mjs` is 1079 lines over 29
  functions; the grammar it reads is 702 lines and the meta-schema 708
  (`wc -l scripts/superdev-format/*`).
- It depends only on `node:fs`, `node:path` and the `yaml` package — its four
  import lines. There is no JSON Schema library: `schemaErrors` at line 733 is
  a hand-rolled subset of draft 2020-12.
- Its flags are `--kind`, `--grammar`, `--meta` and `--doc` (lines 969-975).
  There is no `--json`: output is text lines, plus a `DUPLICATION` block.
- Of the 61 files it checks, 40 are in the canonical knowledge — the 39 schemas and their
  index — and 22 are not: `.agents/core.md` and 21 `.claude/skills/*/SKILL.md`.
  The format check is therefore wider than the canonical knowledge, not a subset of it.
- `knowledge/schemas/*.md` is already walked by both checkers, as an AOKF
  concept and as a `schema` kind. They read different properties of the same
  file, so neither subsumes the other, but nothing merges the two reports.
- A full run over the 61 files takes 123 ms in Node, including process start
  and parsing the grammar (`time node …`).
- The grammar declares four kinds. Three are live; `ledger` is not — the tree
  holds no `*.ledger.json` outside `awa_experiment/`, which has 21 left from
  the one-time conversion.
- The grammar's `match` rules claim 277 files repo-wide, of which 61 are the
  live set. The rest are under `awa_experiment/` (98), `.superdev/` (45),
  `submodules/` (33), `__old/` (21) and `pack/` (19) — including old-format
  copies like `__old/skills/accept/SKILL.md` that would fail.
- Nothing consumes `--doc`, and nothing states the format's rules to the
  agent. `.agents/core.md` is written in the format; there is no document
  about it.
- `crates/lib/superdev-core/tests/validator_parity.rs` pins `aokf::validate`
  verbatim: `let ours = validate(&bundle, &dir).to_json();` compared to a
  golden, with three documented normalisations and "Every other message
  compares verbatim". Its header records that the reference `validator.py` "is
  gone; the goldens are the only remaining record of the behaviour it defined,
  and there is nothing left to regenerate them from."
- The AOKF report already carries what a merged one needs:
  `Finding { path, message, fatal }` at `aokf/validate.rs:38`, with
  `Report::to_json` and `render_human`, and `passed()` as the whole verdict —
  ADR-017 removed the conformance ladder, so there is no level to reconcile.
- `superdev aokf hook validate` reloads the whole knowledge on every edit —
  `validate(&load_bundle(&bundle)?, &root)` in `aokf_cli.rs` — so whole-set
  revalidation on edit is established here.
- The hook entry is keyed by its command string:
  `".claude/settings.json:hooks.PostToolUse[superdev aokf hook validate]"` in
  `lock.rs:284`, with the marker in `components/aokf.rs:85`. Changing the
  marker orphans that lock entry in every managed repo.
- `serde_yaml_ng` is already a workspace dependency of superdev-core
  (`crates/lib/superdev-core/Cargo.toml:30`). `regex` is not.
- The grammar's own `pattern-dialect` names the Rust dialect — "no lookaround,
  no backreferences… Anchor it explicitly" — so every pattern across the 39
  schemas is already written to compile there.
- `npm run test` is `cargo nextest run --workspace && npm run check:aokf`,
  where `check:aokf` is `cargo run --quiet -- aokf validate knowledge`.
- `npm run coverage:check` fails under 90% lines for `crates/lib` and
  `crates/app` separately, so every module this adds carries that gate.

## Goal

One command enforces both the AOKF spec and the superdev format, so a
malformed skill or schema fails a check that runs without Node and cannot
disagree with the canonical knowledge check beside it.

## Outcomes

- O1 — the binary reports, for every file the Node script checks, the same
  findings in the same words.
- O2 — the grammar is still one hand-editable YAML file, and the meta-schema's
  job is done by the Rust types that read it.
- O3 — nothing in the repo needs Node to validate the format, and
  `scripts/superdev-format/` is gone.
- O4 — one command, one report and one hook cover the canonical knowledge and the format
  together, so the two cannot drift and no file is walked twice.

## Non-goals

- Applying schemas to the documents they govern. `target-files` stays
  unenforced; this port moves the existing behaviour and adds none beyond the
  grammar changes W1 settles. That work is filed as issue-011.
- A conformance ladder for the format. ADR-017 removed AOKF's, and adding one
  here would reintroduce the model it rejected.
- Generating an agent-facing `.agents/format.md` from the doc renderer. It is
  ported so the option survives, but wiring it up decides what that file is
  and who owns it, which is a pack question.
- Backporting any of this into `/pack/`. The live tree is the target here, and
  the pack follows separately.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | The binary validates unit, schema and core files, choosing the kind by the grammar's own `match` rules including `except` | O1 |
| FR-2 | For any file, the finding texts the binary emits equal those the Node script emits | O1 |
| FR-3 | The grammar is read from YAML at run time into types that reject unknown keys | O2 |
| FR-4 | A grammar that violates its own constraints fails the run before any file is read, naming the offending key | O2 |
| FR-5 | One command emits one report covering the AOKF checks and the format checks, with one `--json` shape and one exit code | O4 |
| FR-6 | Given no paths, the command checks the canonical knowledge and the trees the grammar's `roots` names; a positional path overrides both | O4 |
| FR-7 | One PostToolUse hook runs that same whole-set check on an edit under the canonical knowledge or any root, so hook and CI verdicts cannot differ | O4 |
| FR-8 | A format finding fails the run without altering any AOKF finding | O4 |
| FR-9 | The command renders the grammar as prose, as the reference's `--doc` does today | O1 |
| FR-10 | `scripts/superdev-format/` no longer exists | O3 |
| FR-11 | With no `.agents/format/grammar.yaml` present, the binary validates from its embedded copy | O3 |

### Non-functional

| ID | Constraint | Budget |
|----|------------|--------|
| NFR-1 | A whole-set run stays fast enough to sit in an edit-time hook unnoticed | under 50 ms for the canonical knowledge and the 61 format files together, against Node's measured 123 ms for the format half alone |
| NFR-2 | The port adds no runtime dependency beyond the regex engine the dialect already assumes | one new crate |
| NFR-3 | The new code clears the repo's coverage gate | 90% lines in each of `crates/lib` and `crates/app` |
| NFR-4 | `aokf::validate` keeps emitting exactly what it emits today | `validator_parity` passes with its goldens untouched |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | A new `superdev format validate` verb — **reversed by D-17** | fold it into `superdev aokf validate` | it read as two specs over two trees; the reversal is that they are two checks over one repo, and separating them buys nothing while costing a second report, a second hook and a second file walk |
| D-2 | The grammar stays YAML read at run time | generate Rust constants from it at build time | the agent and the user both edit it; a build step puts every grammar change behind a rebuild |
| D-3 | The Rust types replace the meta-schema | port `schemaErrors` as well | `#[serde(deny_unknown_fields)]` plus required fields is the same contract, enforced by the compiler instead of at run time |
| D-4 | Parity is proved against goldens captured from the Node script | port it and read the diffs by eye | the AOKF port left goldens nothing can regenerate; this reference still runs, so capture while it does |
| D-5 | The Node script is deleted last | delete it as soon as Rust compiles | it is the only thing that can produce a golden, so it outlives the code that replaces it |
| D-6 | The grammar lives at `.agents/format/grammar.yaml`, read at run time, with an embedded copy as the fallback | `include_str!` alone, or read from the resolved pack | it is the same shape `.agents/aokf/SPEC.md` already has — a capability's spec beside its instruction file — so the agent edits it without a toolchain, and the fallback keeps a pack-less repo able to validate |
| D-7 | Format findings are pass or fail | give them a conformance ladder of their own | levels let a repo adopt the format gradually, which is a feature and not a port; ADR-017 has since removed AOKF's ladder too, so there is now one model |
| D-8 | The command is documented in `api-contracts` as stable | leave it undocumented until it settles | adding a verb is additive within a major version, and an undocumented verb is one nobody finds |
| D-9 | The `ledger` kind leaves the grammar and the port | keep it dormant against a future conversion | the awa conversion is a finished one-off; a kind nothing writes still costs fixtures, goldens and 90% coverage, and the script survives in git history if it is ever wanted |
| D-10 | The grammar names the roots it governs | hardcode them in the CLI as `aokf_cli.rs` does for `knowledge` | the binary runs in other people's repos where `.claude/skills` is a Claude Code convention rather than ours, and an allowlist of roots stays finite where a denylist of caches, backups, submodules and vendored packs does not |
| D-11 | Duplication stays a hard error, and the hook revalidates the whole set | demote it to a warning, or skip it in the hook | a warning is not a check, and a hook that skipped it would pass a skill the merge gate then fails |
| D-12 | `--json` is added to the reference first, and the goldens are JSON | capture the text output as `.golden.txt` | one artifact then pins both FR-2 and FR-5, and text goldens would also pin layout, so re-ordering two lines would break parity |
| D-13 | The doc renderer is ported | drop it as dead code | deleting the reference is one-way, and a renderer's golden is one snapshot rather than a class, so goldens cannot stand in for it |
| D-14 | The grammar changes land before the goldens are captured | fold them into the golden-capture workstream | goldens recorded against the old rulebook would test rules the Rust no longer has |
| D-15 | A second hook entry beside the aokf one — **reversed by D-17** | one combined hook | it solved marker churn, which was the wrong problem; with one report there is one hook and the existing marker is reused unchanged |
| D-16 | The aokf component owns a second hook entry — **moot under D-17** | a new `format` component | no second entry is created, so nothing new needs an owner; the component-model rethink is unaffected |
| D-17 | The two checks merge into one command and one report | keep `format validate` and `aokf validate` side by side | two `Finding` types, two renderers, two `--json` shapes, two hooks and two file walks is how they drift, and `knowledge/schemas/` is already walked by both |
| D-18 | The merge happens above `aokf::validate`, which is left untouched, in a caller that runs both and concatenates findings | put the format checks inside `validate` | `validator_parity` compares that function's output verbatim against goldens captured from a reference that no longer exists, so new findings inside it break every golden with no way to regenerate them |
| D-19 | Format findings carry an error severity but are excluded from `achieved_level` — **moot under ADR-017** | let them grade the ladder like AOKF findings | there is no `achieved_level` to keep them out of; a format finding is simply fatal, which is what this decision was reaching for |
| D-20 | The verb is promoted to `superdev validate`, with `aokf validate` kept as a hidden alias and the hook marker unchanged | rename the hook and the verb together | the marker is the lock key in every managed repo, so renaming it orphans an entry everywhere for a cosmetic gain |

## Workstreams

### W1: Settle the grammar

Depends on: none.

1. Drop the ledger kind — remove its 30 lines from the grammar, its
   `checkLedger` function, and the suffix rule that claims ledger files,
   per D-9.
2. Add the roots — a `roots` list naming the trees the format governs, and
   have `detectKind` and the file walk read it, per D-10.
3. Prove it against the live tree — the Node script still reports 61 passes
   and no findings, and a bare invocation now finds those 61 without being
   given paths.

### W2: Capture the reference behaviour

Depends on: W1.

1. Add `--json` to the reference — emitting the shape `Report::to_json` uses,
   so one golden pins the finding texts and the JSON keys together, per D-12.
2. Build the fixture set — one small tree per failure class the script can
   produce: a good unit, each element rule broken in turn, a schema with an
   unknown key, a three-backtick fence, a core file, a duplication hit, and a
   doc render. The classes come from reading the remaining 28 functions.
3. Capture the goldens — run the reference over each fixture and record the
   output. Hard to reverse: once the script goes in W6 these cannot be
   regenerated, which is the exact position the AOKF port left itself in.

### W3: The grammar as types

Depends on: W1.

1. Define the types — a module under `superdev-core` mirroring the grammar's
   shape, every struct `deny_unknown_fields`, so a typo in the grammar is a
   deserialisation error naming the key.
2. Prove the types against the real grammar — a test that loads
   `.agents/format/grammar.yaml` and asserts it round-trips, which is what
   retires the meta-schema.

### W4: The checks

Depends on: W3.

1. Port the readers — `fenceMap`, `splitFrontmatter`, `extractYaml`,
   `proseOnly`, `parseElements`. These carry the subtle bugs already fixed
   once: blanking a multi-line code span while preserving indices, and reading
   comparables from unfenced rather than code-stripped text.
2. Port the per-kind checks — `checkUnit`, `checkSchema`, `checkCore`, each
   against its fixture goldens from W2.
3. Port the cross-file checks — duplication by token-set containment, and the
   core-block reference check that runs over units and schemas both.

### W5: One report, one command

Depends on: W4.

1. Emit AOKF findings — have the format checks build `Finding` values with
   `fatal` set, so one failing skill fails the run. Nothing in
   `crates/lib/superdev-core/src/aokf/validate.rs` needs teaching: ADR-017
   left `passed()` as the whole verdict. `validate` itself is not touched,
   per D-18.
2. Add the merging caller — one function that loads the canonical knowledge once, runs
   `aokf::validate` and the format checks over the roots, and returns a single
   `Report` with findings in file order.
3. Promote the verb — `superdev validate [path...]` with `--json` and the
   doc flag, in `crates/app/superdev/src/main.rs`, keeping
   `superdev aokf validate` as a hidden alias, per D-20.

### W6: Wire it in and retire the reference

Depends on: W2, W5.

1. Run it where it matters — point `check:aokf` at the merged command and
   rename it, and widen `hook_validate` in
   `crates/app/superdev/src/aokf_cli.rs` so it fires on an edit under the
   knowledge or any declared root. The hook marker and its lock entry are
   unchanged, per D-20.
2. Delete the reference — move the grammar to `.agents/format/grammar.yaml`,
   drop the meta-schema, and remove `scripts/superdev-format/`. Hard to
   reverse in the sense that matters: the goldens from W2 must already be
   committed, because this step ends the ability to make more.
3. Say the command exists — add it to `knowledge/api-contracts.md` and to the
   command set, per D-8.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `scripts/superdev-format/superdev-grammar.yaml` | modified — ledger kind out, `roots` in | W1 |
| `scripts/superdev-format/superdev-grammar.meta.yaml` | modified — the same two changes | W1 |
| `scripts/superdev-format/validate-superdev.mjs` | modified — drop `checkLedger`, read `roots`, add `--json` | W1, W2 |
| `crates/lib/superdev-core/tests/fixtures/format/` | new — one fixture tree and golden per failure class | W2 |
| `crates/lib/superdev-core/tests/format_parity.rs` | new — the fixtures against the captured goldens | W2 |
| `crates/lib/superdev-core/src/format/grammar.rs` | new — the grammar types, `deny_unknown_fields` | W3 |
| `crates/lib/superdev-core/src/format/mod.rs` | new — the module's public surface and the merging caller | W3, W5 |
| `crates/lib/superdev-core/src/format/read.rs` | new — fences, frontmatter, elements, prose | W4 |
| `crates/lib/superdev-core/src/format/check.rs` | new — the per-kind and cross-file checks | W4 |
| `crates/lib/superdev-core/src/format/doc.rs` | new — the grammar rendered as prose | W5 |
| `crates/lib/superdev-core/src/format/re.rs` | new — the grammar's patterns, compiled once | W5 |
| `crates/lib/superdev-core/src/format/grammar.yaml` | new — the embedded copy D-6 calls for | W5 |
| `crates/lib/superdev-core/src/lib.rs` | modified — declare the module | W3 |
| `crates/lib/superdev-core/Cargo.toml` | modified — add `regex` | W4 |
| `crates/lib/superdev-core/src/aokf/validate.rs` | modified — `Finding` and `Report` are reused as they are; `validate` untouched | W5 |
| `crates/app/superdev/src/main.rs` | modified — a top-level `Validate` arm, `aokf validate` hidden | W5 |
| `crates/app/superdev/src/aokf_cli.rs` | modified — the verb runs both checks; the hook fires on knowledge or root | W5, W6 |
| `.github/workflows/checks.yml`, `.github/PULL_REQUEST_TEMPLATE.md`, `CONTRIBUTING.md` | modified — the renamed check | W6 |
| `knowledge/development-procedure.md`, `knowledge/definition-of-done.md`, `knowledge/error-handling.md`, `knowledge/software-components.md` | modified — the renamed check and the widened hook | W6 |
| `package.json` | modified — `check:aokf` renamed and pointed at the merged command | W6 |
| `.agents/format/grammar.yaml` | new — the grammar's home, moved from the script's directory | W6 |
| `scripts/superdev-format/` | deleted — script, grammar and meta-schema | W6 |
| `knowledge/development-commands.md` | modified — the renamed check in the command set | W6 |
| `knowledge/api-contracts.md` | modified — the command in the stable CLI surface | W6 |

## Acceptance

| Check | Verifies |
|-------|----------|
| After W1 the Node script still reports 61 passes and no findings, and `rg -n 'ledger' scripts/superdev-format/` returns nothing | FR-1 |
| `cargo test -p superdev-core --test format_parity` passes on every captured golden | FR-2 |
| `cargo test -p superdev-core --test validator_parity` passes with its goldens unedited | NFR-4 |
| `superdev validate` with no arguments reports the canonical knowledge's concepts and the format's 61 files in one report, with one exit code | FR-5, FR-6 |
| `knowledge/schemas/spec.md` appears once in that report, carrying both its AOKF and its format findings | FR-5 |
| A grammar with a key removed and a key misspelled fails the run naming the key, before any file is read | FR-3, FR-4 |
| `superdev validate --json` output parses and carries a single `findings` array in the shape aokf already emits | FR-5 |
| The doc render output equals its captured golden | FR-9 |
| Breaking a skill and breaking a concept's frontmatter each make the one hook exit 2 with the finding on stderr | FR-7 |
| A format error fails the run without changing what the AOKF checks reported | FR-8 |
| Moving `.agents/format/grammar.yaml` aside leaves the command working and reporting the same findings | FR-11 |
| A test asserts the embedded grammar equals `.agents/format/grammar.yaml` byte for byte | FR-11 |
| `rg -n 'superdev-format' --hidden` returns nothing outside this plan and the changelog | FR-10 |
| A whole-set run is timed and stays inside the hook budget | NFR-1 |
| `npm run coverage:check` passes | NFR-3 |

## Definition of done

- Every Acceptance row passes on a clean checkout of the branch.
- The goldens are committed, and `format_parity.rs` carries the same warning
  `validator_parity.rs` does about what editing one means.
- `.claude/settings.json` still carries exactly one PostToolUse entry for
  superdev, under its original marker, and the lock entry is unchanged.
- `knowledge/plans/index.md` lists this plan and its status reads done.
- The changelog has an Unreleased entry for the promoted command.
- The pack question is filed as [issue-016](../issues/issue-016-bug-sync-would-revert-the-schema-migration.md),
  since the live tree and `/pack/` diverge until it is answered.
- The generated-format-docs idea is filed as
  [issue-017](../issues/issue-017-feature-request-the-format-has-no-agent-facing-document.md), since
  the doc renderer now exists in the binary with nothing consuming it.

## Risks

- Risk: the finding texts are the contract, and several were reworded during
  the format work — the `exactlyOneOf` message and the fence message both
  carry reasoning in prose. Mitigation: the goldens fix them in W2, before any
  wording can drift.
- Risk: the merge reaches into `validate` by accident during W5 and breaks the
  AOKF goldens, which cannot be regenerated. Early signal: `validator_parity`
  is in the same `cargo test` run as `format_parity`, so it fails in W5 rather
  than after the reference is gone.
- Risk: JavaScript and Rust regex disagree on a pattern that currently passes,
  because the Node script compiles patterns with `new RegExp`. Early signal: a
  fixture whose `heading-pattern` matches under Node and not under `regex`
  fails in W4 rather than after the script is gone.
- Risk: the duplication check is order- and tokenisation-sensitive, so a
  faithful port is harder than it looks. Mitigation: it is the last check in
  W4, with its own fixtures, and it is the one place a golden diff is expected
  to need reading rather than accepting.
- Risk: the grammar has two copies under D-6 — the file and the embedded
  fallback — which can drift. Mitigation: a test asserts the embedded bytes
  equal the file's, so a change to one without the other fails the suite.
- Risk: the 90% coverage gate lands hardest on the CLI glue. Early signal:
  `npm run coverage:check` fails in W5, not at the end. `main.rs` already uses
  `#[cfg_attr(coverage_nightly, coverage(off))]` for exactly this.

## Out-of-band notes

The template provenance is gone from all 38 schemas that carried it, taking
the canonical knowledge from 39 warnings to none. Repointing those citations at `/pack/`
was the obvious-looking fix and the wrong one: the templates are superseded
by the schemas themselves, so the pack copies are deleted as soon as these
changes are backported, and the citation would dangle again.

`.agents/process.md` still tells the agent that document skeletons live in
`knowledge/templates/`. It is pack-owned, so fixing the live copy would be
drift; it wants fixing in the pack.

`awa_experiment/` and `__old/` both hold files the grammar's `match` rules
would claim. The `roots` list from W1 excludes them, but they are the reason
a bare run cannot simply walk the repository.

D-16 is left in the table rather than deleted because its reasoning still
applies the moment anything else needs an owner: the component model wants
rethinking as a whole, and the components already depend on each other.

What W5 and W6 settled that the plan left open, recorded here because each is
a decision the Decisions table did not anticipate.

**NFR-1 is missed, and the number is recorded rather than the budget moved.**
A whole-set run of the release binary takes 82 ms against a budget of 50 ms.
The split: 24 ms for the 61 format files, 28 ms to load the canonical knowledge, 5 ms for
`aokf::validate`, the rest process start and reads. The format half is the one
this plan owns and it is a fifth of the reference's 123 ms; the canonical knowledge load is
the AOKF half's existing cost, which the hook already pays on every edit
today. Two changes got it there from 165 ms, neither altering a finding: the
grammar's patterns compile once rather than once per value checked
(`format/re.rs`), and the duplication check compares interned integers rather
than strings. Bringing the last 32 ms inside the budget means making
`load_bundle` faster, which is AOKF work and belongs to its own plan.

**The canonical knowledge became a flag.** D-20 kept `aokf validate` as an
alias, and the merged verb takes a positional path that is the *scope* of the
run — FR-6's "a positional path overrides both". The two readings collide: the
old verb's positional was the knowledge directory. That directory moved to
`--bundle <DIR>`, and a positional path covers the canonical knowledge when it
is that directory or contains it, so `superdev validate knowledge` and
`superdev validate .` both still check it. Scope is a positional and
configuration is a flag; the alternative was a rule that guessed from the
directory's contents, and it would have hidden the `no manifest (required)`
finding for the canonical knowledge it was guessing about.

**The embedded grammar lives in the crate, not the pack.** D-6 says where the
grammar is read from and that a copy ships inside the binary; it does not say
where that copy lives. `include_str!` cannot reach outside the crate and still
be packaged, so it is at
`crates/lib/superdev-core/src/format/grammar.yaml`, with a test holding it
byte for byte equal to `.agents/format/grammar.yaml`. Writing the grammar into
the repositories superdev manages — the way `.agents/aokf/SPEC.md` is written,
as a binary-owned file — is a pack question, and is
[issue-017](../issues/issue-017-feature-request-the-format-has-no-agent-facing-document.md)'s to
answer alongside the format document.

Both follow-ons are filed as issues rather than as the plans the Definition of
done originally called for: each is one question with a stated recommendation,
and neither has enough settled to cut into workstreams. The pack one is
plan-sized work and will want a plan when it is picked up.

The `--json` report keeps its `bundle` key, naming the canonical knowledge directory the
run was configured with. When a positional path excludes the canonical knowledge, `concepts`
is 0 and no knowledge finding appears; the key names the invocation, not what was
read.
