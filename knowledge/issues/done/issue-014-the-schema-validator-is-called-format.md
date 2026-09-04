---
type: Issue
id: issue-014-the-schema-validator-is-called-format
title: The schema validator is called "format", which already means three other things here
description: The grammar-driven validator lived at src/format/, shipped its grammar at .agents/format/ and called its files "superdev-format", while format! is 457 lines away in the same crate, "pack format" is a glossary term and the knowledge format is itself a format.
kind: chore
lifecycle: done
---

# Chore: rename the schema validator off the word "format"

## Summary

The grammar-driven validator — 3,434 lines checking documents against their
schemas, and skills and the core file against the grammar — was named
`format` at every level: the module, the grammar's home, the tests, and the
phrase "superdev-format files" in the CLI help. The word was taken three
times over in this repository, and none of the three was this validator.

## Context

The surfaces the rename reaches:

- The module `crates/lib/superdev-core/src/format/` — six files, 3,527 lines
  (`wc -l src/format/*.rs`).
- 457 `format!` calls in the same crate
  (`grep -rc 'format!' crates --include=*.rs`), so `use crate::format` and
  `std::fmt` sat a character apart in a reader's head.
- **Pack format** in `knowledge/glossary.md`, 12 uses: the version a pack
  manifest declares. A version number, not a language.
- The knowledge format itself. `.agents/format/` sat directly beside
  `.agents/aokf/SPEC.md`, so the directory read as that format's home,
  which is precisely what it was not.
- `GRAMMAR_PATH` at `src/format/mod.rs:26`, which fixed `.agents/format/`
  into a path other repositories hold.
- The test suite and fixtures: `tests/format_parity.rs`,
  `tests/fixtures/format/`.

## Behaviour

Done means:

- `git grep -n 'crate::format\|superdev_core::format'` returns nothing.
- The grammar is read from a path that does not say "format", and
  `the_embedded_grammar_equals_the_repository_copy` still passes — it is
  what catches a grammar path updated on one side only.
- The word "format" in the tree means formatting, a pack version, or the
  knowledge format, and never this validator.

## Resolution

P008. The module is `superdev_core::validate::schema`, under a `validate`
parent that owns both halves of the check; the grammar moved to
`.agents/sokf/grammar.yaml`, beside the specification; the suites and
fixtures became `schema_snapshots` and `tests/fixtures/schema/`.

The name this issue proposed — "schema-validator" — was adopted for the
half, and its own objection was answered rather than dodged: `schema` is
also one of the grammar's three kinds. The objection turned out to be weak,
because those three kinds describe what the checker happened to do when
nothing dispatched documents to schemas at all. Once P008 made document
against schema the dominant relation, `schema` was the right name for the
half and `unit`/`core` the ancillary cases.

The silent-fallback risk this issue named was real and is unchanged:
`load_grammar` still treats a missing grammar as `NotFound` and falls back
to the embedded copy, so a repository carrying its own grammar at the old
path now validates against the embedded one without saying so. That is not
fixed here and wants an issue of its own.

The stale `doc` credit to `validate-awa.mjs` was fixed separately, before
this plan.
