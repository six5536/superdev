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

# Design (settled by grilling, 2026-08-17)

`Tx` owns the whole journal — backups, both `Undo` variants
(`RestoreFile`, `RunCommand`), the irreversible list, and unwind — so
replay ordering holds by construction. `Session` keeps lock bookkeeping
and reports. `engine.rs` becomes an `engine/` directory. Pin effects
move by ownership handoff, not index arithmetic.

# Tasks

1. `engine/tx.rs`: extract `Tx` from `Session` —
   `tx.write(path, content)`, `tx.remove(path)`,
   `tx.record_command_undo(program, args)`,
   `tx.mark_irreversible(line)`, and `unwind`. Backup-dir and stamp
   management come with it. Tx stays dumb: the drift guards ("changed
   since superdev wrote it") remain in the appliers, so the
   [managed-entry plan](2026-08-17-managed-entry-interface.md) can
   collapse them into its one driver without moving them twice.
2. Rewrite the appliers in `engine/mod.rs` as content computation
   calling `Tx` — the six whose bodies are the shared read → backup →
   journal → write ritual collapse to their 3–6 variant-specific lines.
3. `engine/pins.rs`: the mise pin policy. `apply_pins` returns
   per-entry `PinEffects`; the apply loop hands each entry its own
   effects at completion and a `&mut ComponentReport` for the
   trust/install commands it fabricates. The
   `pins: Vec<Vec<(String, String)>>` parallel structure goes; reports
   stay byte-identical.
4. `engine/materialise.rs`: `materialise_skills` using `Tx` directly;
   delete the sentinel plumbing (`probe`, the `unreachable!` at
   `engine.rs:646`).
5. New pure modules at the crate root: `json_edit.rs` (the JSON pointer
   mini-library) and `fsutil.rs` (`read_text`, `collect_files`), so
   `orphan.rs` imports purity, not the effector.
6. Split the in-file tests along the same seams: rollback once against
   `Tx`; pin ordering without temp files; materialise without
   exercising unrelated appliers.

The public `engine::plan`/`engine::apply` interface does not change.
Observation-at-apply-time for materialised skills stays — the spec chose
it (planning needs no checkout); only the code moves.

# Done

One `journal.push(Undo::RestoreFile…)` site, inside `Tx`. No module
under `engine/` over ~400 code lines. `npm test` passes; the unwind
e2e tests and the apply reports are byte-identical.

# Sequencing

Before the
[managed-entry interface](2026-08-17-managed-entry-interface.md), whose
removal driver builds on `Tx`. Independent of the verb track. Delete this file in the commit that completes the work.
