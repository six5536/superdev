---
type: Plan
id: plan-managed-entry
title: Managed-Entry Interface
description: One managed-entry interface with file, pin and JSON adapters; one drift-guarded removal driver replaces three appliers and the orphan arms.
status: draft
---

From the 2026-08-17 architecture review, candidate 2 (Strong). Second
plan of the engine track, on top of the
[transaction primitive](2026-08-17-engine-transaction-primitive.md).

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

# Tasks

1. Define the shape interface — extend `Claim` or introduce a
   `ManagedEntry` enum; run design-it-twice on this at execution.
   Per-shape operations: `read_current(root)`, `remove(content)`;
   `lock_key()`/`parse_key()` live together so encode and decode meet
   at one seam.
2. One drift-guarded removal driver in the engine using
   `tx.remove_if_unchanged`: record key → read → absent ⇒ already-gone
   skip → hash mismatch ⇒ released skip → journal + remove. One arm per
   shape for the shape-specific step only.
3. Collapse the three engine removal appliers onto the driver.
4. Rewrite `orphan.rs` `classify`/`current_value`/`removal` against the
   shape interface, deleting its parse-the-key logic.
5. Tests: released/gone/removed classification once against the
   interface; one small adapter test per shape; the orphan-sweep e2e
   stays as-is.

# Done

The six-step removal ritual exists once. `orphan.rs` contains no lock-key
string parsing. A grep for the "changed since superdev wrote it" guard
matches one site. `npm test` passes; orphan e2e unchanged.

# Sequencing

After the
[transaction primitive](2026-08-17-engine-transaction-primitive.md).
Independent of the verb track. Delete this file in the commit that
completes the work.
