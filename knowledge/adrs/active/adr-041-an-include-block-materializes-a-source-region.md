---
type: Decision
id: adr-041-an-include-block-materializes-a-source-region
title: An include block materializes a source region
description: An include block may name a repository file and a region of it, bounded by sokf:begin and sokf:end markers in the file's own comment syntax — validate --fix splices the region in as a fenced block and validate errors on a stale copy — so a document carries source it never parses and cannot drift from.
lifecycle: active
links:
  - rel: references
    to: adr-027-an-include-block-materializes-shared-content-in-place
    note: The mechanism this extends, from a concept's body to a file's region; every rule there holds here.
  - rel: references
    to: adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate
    note: A stale, empty or unresolvable source include is decidable from the tree, so it is an error and the turn cannot end on it.
---

# ADR-041: An include block materializes a source region

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

[ADR-027][sokf:adr-027-an-include-block-materializes-shared-content-in-place]
gave the knowledge format a generated block: a document names a
concept between `sokf:include` markers, `validate --fix` splices the
concept's body in, and `validate` fails the run when the copy differs.
The content has one authored home and every copy is enforced.

A contract's definition needs exactly that, from a different home. The
interface a contract describes is declared in source — a clap tree, a
`.proto` file, an OpenAPI document, a migrations directory — and every
definition block on file is a hand-written copy of one, kept honest by
a drift test that exists only because the copy does. The include block
cannot reach a file, so the copy is authored, and it drifts.

A file usually holds more than the promised surface. `main.rs` carries
the clap tree beside everything else in the binary's entry point. An
include has to name the part that is the interface, and it has to do
so in a way that works for every language a managed repository might
use, because superdev parses none of them.

## Decision

We will let an include block name a `/`-rooted repository path in
place of a concept id, optionally followed by `#` and a region name:
`<!-- sokf:include /crates/app/superdev/src/main.rs#cli -->`.

A region is bounded by a line containing `sokf:begin <name>` and a
later line containing `sokf:end <name>`, matched by substring, so the
markers sit in whatever comment syntax the file uses and superdev reads
none of it. Regions sharing a name concatenate in file order, so a
surface scattered through one file is one include. A path with no `#`
includes the whole file.

`validate --fix` materialises the region as a fenced block. The fence
tag is the file's extension, mapped where the conventional tag differs
— `rs` to `rust`, `yml` to `yaml`, `ts` to `typescript`, `py` to
`python` — and bare when the file has no extension. A
`sokf:generated-by` line in the file's leading lines is carried into
the block unchanged, so a reader of the document sees that the file
was itself rendered from something else.

`validate` reports an error when the block is absent, empty or differs
from its region; when the path does not exist or resolves outside the
repository; and when the file carries no region of the named name.
Each error names the path and the region, and each is decidable from
the tree, so the turn cannot end on one
([ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]).
Nothing inside a region is parsed: the validator moves bytes and
compares them.

The SOKF SPEC remains the normative home for the block's authoring
rules, beside the concept form ADR-027 placed there.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Markers in the source, matched by substring | Works in every language and every structured file with no parser; the source declares its own contract boundary; the same mechanism superdev already uses for its own markers | Two comment lines in the source per region |
| Symbol names resolved through the code index | No markers; exact item boundaries | Binds every contract to the optional `code-index` capability, and stops being language-agnostic the moment codegraph does not know the language |
| Line ranges | Nothing in the source | Every edit above the range shifts it, and `--fix` regenerates the wrong lines without noticing — the one form whose check cannot catch its own failure |
| Whole file only | The simplest rule | Forces source layout to follow contract layout: a file must hold only the surface |
| A tag named in the marker, `as rust` | Never wrong | One more thing to type and keep in step with the file, for a case a four-row table covers |
| `--fix` runs a generator instead of including its output | Always current, no intermediate file | Arbitrary project commands on the PostToolUse hook's path at every edit |

## Consequences

- Positive: a document carries a declaration from source, readable in
  place, that cannot drift; the mechanism is one the validator already
  runs at every edit; a region's markers tell a reader of the source
  what is promised.
- Negative: an edit inside a region leaves the including document
  stale until `--fix` runs, which the Stop hook forces before the turn
  ends; a marker pair sits in the source.
- Follow-ups: the SPEC's include section gains the source form; the
  `contract-002` `--fix` description names it; `sokf_read` serves the
  materialised copy with no new code, as it does for a concept include.

<!-- sokf:links -->
[sokf:adr-027-an-include-block-materializes-shared-content-in-place]: /knowledge/adrs/active/adr-027-an-include-block-materializes-shared-content-in-place.md
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
