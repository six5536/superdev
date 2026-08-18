---
type: Plan
id: plan-managed-entry
title: Managed-Entry Interface
description: Claim gains the per-shape operations and one removal action; one drift-guarded removal driver replaces three appliers and the orphan arms.
status: draft
---

From the 2026-08-17 architecture review, candidate 2 (Strong). Second
plan of the engine track, on top of the transaction primitive (landed
2026-08-18).

# Friction

"An entry superdev owns inside a possibly-shared file" exists in three
shapes — whole file, mise pin, managed JSON key — and each shape is
spelled out in five parallel places: a set-`Action`, a remove-`Action`,
a `Claim` arm (`component.rs:34`), an orphan
`classify`/`current_value`/`removal` arm (`orphan.rs:67–105`), and an
engine applier (`engine.rs:511/680/716`) whose body is the same
six-step ritual. The lock key is a stringly seam: `Claim::lock_key`
(`component.rs:51`) encodes what `orphan::classify` (`orphan.rs:67`)
decodes, kept sound only by the invariant that superdev never writes a
path containing `:` (`orphan.rs:65`). The drift rule is written twice
per shape — plan-time hash compare in `orphan::plan` (`:55`) and
apply-time guard in each removal applier (`engine.rs:523/696/737`).
Adding a fourth shape touches ~9 sites across 4 files. Deletion test:
any one removal applier deleted reappears near-verbatim from its
siblings.

# Design (settled by grilling, 2026-08-17)

`Claim` is the interface — it already models the three shapes; no new
trait. It gains the per-shape operations, and the three removal `Action`
variants collapse into one `Action::Remove { claim, reason }`, so the
engine has one removal arm by construction. The drift check stays at
both plan time and apply time — same helper, two calls, guarding the
plan→apply gap.

# Tasks

1. Methods on `Claim` (in `component.rs`, importing the pure
   `mise`/`json_edit` helpers): `parse_key` (moved from
   `orphan::classify`, beside `lock_key` so encode and decode meet at
   one seam), `read_current(root)` (absorbing `orphan::current_value`),
   and the remove-from-content operation for the shared-file shapes.
2. Collapse `RemoveFile`/`RemoveMisePin`/`RemoveJsonKey` into
   `Action::Remove { claim, reason }`; `describe()` derives today's
   exact wordings from the shape. Verify plan/apply report strings stay
   byte-identical.
3. One drift-guarded removal applier in `engine/mod.rs` — the guard's
   only home, for all three shapes: read via `Claim` → absent ⇒
   already-gone skip → hash mismatch ⇒ released skip → remove via Tx's
   plain journaled operations (file delete vs shared-file rewrite stays
   a visible match in this one place). Delete the three old appliers.
4. Rewrite `orphan.rs` onto the `Claim` methods, deleting its
   `classify`/`current_value`/`removal` trio and the lock-key string
   parsing.
5. Tests: released/gone/removed classification once against the `Claim`
   interface; one small per-shape test; the orphan-sweep e2e stays
   as-is.

# Done

The six-step removal ritual exists once. `orphan.rs` contains no
lock-key string parsing. A grep for the "changed since superdev wrote
it" guard matches one site. `npm test` passes; orphan e2e and report
strings unchanged.

# Sequencing

After the transaction primitive (landed 2026-08-18). Independent of
the verb track. Delete this file in the commit that
completes the work.
