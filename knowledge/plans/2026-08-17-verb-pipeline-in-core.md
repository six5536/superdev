---
type: Plan
id: plan-verb-pipeline
title: Verb Pipeline in Core
description: Move the verb orchestration — prune, repo entry, orphan ordering, lock reconciliation, stamping — from manage.rs into one core pipeline.
status: draft
---

From the 2026-08-17 architecture review, candidate 1 (Strong, top
recommendation). Restores the architectural rule that domain logic lives
in `superdev-core` and the binary stays thin — `manage.rs` currently
carries ~350 of its 718 code lines as domain logic.

# Friction

Everything between manifest and plan lives in the binary, re-orchestrated
per verb, with the invariants carried by comments:

- `plan_all` (`manage.rs:342`) — repo `.gitignore` entry + `engine::plan`
  + claims + `orphan::plan`, orphans-last encoded by a comment.
- `prune_custom` (`manage.rs:491–523`) — lock surgery; must run before
  planning, documented twice (`:148–150`, `:186–189`), enforced nowhere.
- sync's released/gone/disabled lock reconciliation (`manage.rs:208–221`)
  and `stamp_blueprint`; status alone plans against a `plannable()` copy.
- the version-policy cluster (`behind_pins`, `pin_mismatch`,
  `checksum_pin_mismatch`, `plannable`, `is_behind`) and the adoption
  policy (`adopt_existing_skills`, `adopt_existing_mattskills`,
  `manage.rs:442–486`).

Deletion test: delete these and the logic reappears — it earns its keep,
in the wrong crate, split into verb-shaped fragments.

# Design (settled by grilling, 2026-08-17)

One core module, `pipeline`, with one plan entry and one apply entry.
Core owns the printable lines (the `orphans.released_lines()`
precedent); the binary keeps CLI parsing, printing order, hints and
exit codes. Note: `prune_custom` mutates the lock, not the manifest —
sync never writes the manifest.

# Tasks

1. `crates/lib/superdev-core/src/pipeline.rs`:
   `plan_repo(root, runner, manifest, lock, PlanMode) -> RepoPlan`.
   `PlanMode::Status` substitutes the plannable copy (checksum pins
   reset to the registry default); `PlanMode::Sync` errors on a
   mismatched pin. Absorbs `plan_all`, `repo_entry`, the in-memory
   `prune_custom` and the orphans-last ordering. `RepoPlan` carries the
   planned actions, the line producers (behind, custom, switch,
   released, blueprint) and the facts the binary needs (`has_actions`,
   behind non-empty, materialising).
2. `apply_repo(root, runner, manifest, plan, lock) -> outcome`:
   reconciles released/gone/disabled lock keys, then either saves the
   changed lock and stamps (no actions) or applies, stamps and reports
   — both branches inside; returns a materialised flag for the binary's
   `SETUP_HINT`. Dry-run never reaches it (the verb stops after
   printing the plan).
3. Move the version-policy cluster (`behind_pins`, `pin_mismatch`,
   `checksum_pin_mismatch`, `plannable`) into `pipeline.rs`, on the
   registry queries the
   [checksum-pin planner](2026-08-17-checksum-pin-planner.md)
   introduced. `parse_target` stays in the binary (CLI parsing) using
   those queries.
4. Move `adopt_existing_skills` into `components/skillpack.rs` and
   `adopt_existing_mattskills` into `components/mattskills.rs`, beside
   the constants they read; init calls one core wrapper.
5. Rewrite `init`/`status`/`sync`/`update` as load → call → render →
   exit code. Init keeps its git/manifest guards and manifest build,
   then reuses `plan_repo(Sync)` + `apply_repo`. Exit codes are
   binary-computed from `RepoPlan` facts.
6. Move the manage.rs unit tests covering moved logic into core; they
   use the crate-internal `FakeRunner` (visible to core unit tests
   today — no dependency on the
   [runner seam](2026-08-17-runner-seam-for-verbs.md) plan). Binary
   tests keep flag parsing and rendering.

# Done

`manage.rs` under ~300 lines, none of them domain logic. The
prune-before-plan and orphans-last invariants hold by construction (one
call site each). `npm test`, `npm run check:blueprint` pass; CLI output
byte-identical for the fixture repos.

# Sequencing

After [one checksum-pin planner](2026-08-17-checksum-pin-planner.md)
(smaller moves). The [runner seam](2026-08-17-runner-seam-for-verbs.md)
lands immediately after — `plan_repo`/`apply_repo` taking
`&dyn CommandRunner` is what it needs. Delete this file in the commit
that completes the work.
