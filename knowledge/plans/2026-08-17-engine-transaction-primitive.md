---
type: Plan
id: plan-engine-tx
title: Engine Transaction Primitive
description: Extract a transaction type owning backup, journal and unwind; appliers, the pin policy and materialise become callers.
status: draft
---

From the 2026-08-17 architecture review, candidate 4 (Worth exploring).
First plan of the engine track; the
[managed-entry interface](2026-08-17-managed-entry-interface.md) builds
its removal driver on the primitive this extracts.

# Friction

`engine.rs` is 2,815 lines (1,182 code + 1,633 in-file tests). The
journal/unwind machinery is genuinely deep, but its invariant — journal
before write, always — is re-established by hand at 8
`journal.push(Undo::RestoreFile…)` sites (`engine.rs:244/470/493/536/
705/746/781/821`), so it sits in every applier's interface instead of
behind one. `Session` also carries the mise ordering policy
(`engine.rs:209–327`, which fabricates actions the plan never contained
and re-attributes them via `pins: Vec<Vec<…>>` index arithmetic) and the
128-line `materialise_skills` (`engine.rs:549–676`), wired through
sentinel plumbing (`probe`, the `unreachable!` at `:646`). The engine
also doubles as the pure-utils home: `orphan.rs:13` imports
`json_value_at` and `read_text` from the effector module.

# Tasks

1. Extract a `Tx` type owning backup-dir management, journal, and
   unwind: `tx.write(path, content)`,
   `tx.remove_if_unchanged(path, prior_hash)`, plus the run/record hook
   commands need. Scope note: file operations first; command journalling
   moves only if it falls out naturally.
2. Rewrite the appliers as content computation calling `Tx` — the six
   whose bodies are the shared read → backup → journal → write ritual
   collapse to their 3–6 variant-specific lines.
3. Move the mise pin policy into its own module using `Tx`. Open
   question for execution: replace the entry-index arithmetic with
   explicit attribution.
4. Move `materialise_skills` into its own module using `Tx`; delete the
   sentinel plumbing.
5. Move the pure utilities (the JSON mini-library, `read_text`,
   `collect_files`) out of the engine so `orphan.rs` stops importing
   from the effector.
6. Split the in-file tests along the same seams: rollback tested once
   against `Tx`; pin ordering without temp files; materialise without
   exercising unrelated appliers.

The public `engine::plan`/`engine::apply` interface does not change.
Observation-at-apply-time for materialised skills stays — the spec chose
it (planning needs no checkout); only the code moves.

# Done

One `journal.push(Undo::RestoreFile…)` site, inside `Tx`. `engine.rs`
under ~600 code lines. `npm test` passes; the unwind e2e tests pass
unchanged.

# Sequencing

Before the
[managed-entry interface](2026-08-17-managed-entry-interface.md), whose
removal driver wants `remove_if_unchanged`. Independent of the verb
track. Delete this file in the commit that completes the work.
