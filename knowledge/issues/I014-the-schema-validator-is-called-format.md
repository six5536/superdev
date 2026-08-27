---
type: Issue
id: issue-014-the-schema-validator-is-called-format
title: The schema validator is called "format", which already means three other things here
description: The grammar-driven validator lives at src/format/, ships its grammar at .agents/format/ and calls its files "superdev-format", while format! is 457 lines away in the same crate, "pack format" is a glossary term and AOKF is itself a format. It should be called schema-validator.
status: draft
tags: [needs-triage]
---

# Bug: the schema validator is called "format", which already means three other things here

## Summary

The grammar-driven validator — 3,434 lines checking skills, schemas and the
core agent files against `grammar.yaml` — is named `format` at every level:
the module `crates/lib/superdev-core/src/format/`, the grammar's home
`.agents/format/`, the parity test and its fixtures, and the phrase
"superdev-format files" in the CLI help and the grammar's own doc string. The
word is taken three times over in this repository, and none of the three is
this validator. It should be called `schema-validator`.

## Environment

- Version/commit: superdev 0.2.0, grammar 2.0
- Platform: any; a naming defect across the module, the paths and the CLI

## Steps to reproduce

1. Read `crates/lib/superdev-core/src/format/mod.rs:1-8`, the module doc:
   "format — the superdev-format checks: skills, schemas and the core file".
2. Run `grep -c 'format!' -r crates/ --include=*.rs` — 457 hits of the
   standard macro in the same crate as the module.
3. Run `superdev validate --help` and read "Check the knowledge bundle and
   the superdev-format files" and "Print the format grammar as prose".
4. Read the **Pack format** entry in `knowledge/glossary.md`.

## Expected behaviour

The validator is named for what it does. `schema-validator` in the module
path, the grammar's directory, the script directory, the test and fixture
names, and the prose — so that a reader who sees `format` anywhere in the
tree can take it to mean formatting, or a version number, or AOKF, and never
have to ask which.

## Actual behaviour

`format` names four unrelated things at once:

1. **This validator.** `src/format/` (`check.rs`, `grammar.rs`, `read.rs`,
   `doc.rs`, `re.rs`, `mod.rs`), `.agents/format/grammar.yaml`, the embedded
   copy at `src/format/grammar.yaml`,
   `crates/lib/superdev-core/tests/format_parity.rs`, the fixtures at
   `tests/fixtures/format/`, and the plan
   `knowledge/adhoc-plans/P006-rust-format-validator.md`.
2. **String formatting.** 457 `format!` calls in the same crate. `use
   crate::format` and `std::fmt` sit a character apart in a reader's head.
3. **Pack format.** The glossary defines it as "the version a pack manifest
   declares. A binary refuses a format it does not know" — 12 uses. A
   version number, not a language.
4. **AOKF itself** — the Agent Open Knowledge Format. `.agents/format/` sits
   directly beside `.agents/aokf/SPEC.md`, so the directory reads as the
   AOKF format's home, which is precisely what it is not.

The collision is already costing the module doc a paragraph of disambiguation
against sense 4 (`mod.rs:5-8`).

## Root cause (if known)

The name is inherited from the reference implementation, where the checked
files were "format files" and the checker was the thing that read them. It
described the *inputs* rather than the *job*, which is why it collided the
moment those inputs stopped being the only formatted thing in the tree. The
port to Rust (P006) carried the name across without revisiting it, and
`GRAMMAR_PATH = ".agents/format/grammar.yaml"`
(`crates/lib/superdev-core/src/format/mod.rs:26`) fixed it into a path other
repositories now hold.

## Proposed fix / workaround

- Rename the module `crates/lib/superdev-core/src/format/` →
  `schema_validator/`, and every path that echoes it:
  `tests/format_parity.rs` → `tests/schema_validator_parity.rs`,
  `tests/fixtures/format/` → `tests/fixtures/schema-validator/`.
- Move the grammar: `.agents/format/` → `.agents/schema-validator/`, with
  `GRAMMAR_PATH` following. See the regression risk — this one is not a
  rename, it is a migration.
- Replace "superdev-format files" in the CLI help and in `grammar.yaml`'s
  `doc` string with a phrase naming the validator, in both copies of the
  grammar and in the doc golden that pins them.
- Settle one thing before any of it: `schema` is already the name of one of
  the grammar's three kinds (`unit`, `schema`, `core`) and of the concepts
  under `knowledge/schemas/`. "schema-validator" therefore reads as "checks
  the schemas", when it checks all three kinds across `.agents`,
  `.claude/skills` and `knowledge/schemas`. Either the name is adopted with
  a glossary entry stating that it covers all three, or a name is chosen
  that does not overlap a kind it contains.
- Fix the stale reference met on the way: the grammar's `doc` key still
  credits `validate-awa.mjs`, a script this repository never had and no
  longer has anything like, and `superdev validate --doc` prints it to the
  user. It is in `.agents/format/grammar.yaml`, in the embedded copy, and in
  `tests/fixtures/format/doc.golden.txt`, which pins it.
- Workaround: none needed. Nothing is broken at runtime; the cost is paid by
  every reader.

## Regression risk

Moving `.agents/format/grammar.yaml` fails silently in seeded repositories.
`load_grammar` treats a missing grammar file as `NotFound` and falls back to
`EMBEDDED_GRAMMAR` (`mod.rs:54`), so a repository carrying its own grammar
at the old path does not error after the move — it quietly validates against
the embedded copy instead, and any local rule in that grammar stops being
enforced. The move needs either a read of both paths for one release, or a
loud error when the old path exists and the new one does not.

The parity test pins the Rust checks against goldens captured from the Node
reference before P006 deleted it, so the directory rename has to leave every
golden's contents byte-identical. The reference itself recovers from git
history and still reproduces them, which makes an audit possible after the
rename but is not a licence to refresh one. `the_embedded_grammar_equals_the_repository_copy`
(`mod.rs:406`) will catch a grammar path updated on one side only.
