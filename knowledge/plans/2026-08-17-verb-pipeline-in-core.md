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

# Tasks

1. New core module (working name `pipeline`) with
   `plan_repo(root, runner, manifest, lock) -> RepoPlan`: absorbs
   `plan_all`, `repo_entry`, the in-memory prune and the orphan
   ordering. `RepoPlan` carries the component plans, prune/orphan report
   lines and behind-pins facts the verbs render. Decide at execution how
   the pruned manifest is surfaced (status renders it, sync persists it).
2. `apply_repo(plan, lock, …) -> ApplyResult` absorbing sync's lock
   reconciliation and the blueprint stamp; lock saves only on ok, as now.
3. Move the version-policy cluster into core — into the registry where
   [one checksum-pin planner](2026-08-17-checksum-pin-planner.md) has
   already put the binary-pinned flag.
4. Move the init adoption policy into core beside the components whose
   constants it reads.
5. Rewrite `init`/`status`/`sync`/`update` as load → call → render →
   exit code. Rendering, hints and clap stay in the binary.
6. Move the manage.rs unit tests covering moved logic into core against
   `FakeRunner`; binary tests keep flag parsing and rendering.

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
