---
type: Idea
id: idea-016-a-dead-session-claim-always-releases
title: A dead session claim always releases
description: Record the owner process for every workflow claim, and let cancel release a claim that a stopped Pi session left behind.
status: draft
---

# Idea: a dead session claim always releases

Give a user a way to continue work after a Pi session stops without
cancellation. Record the owner process for each claim. Let `cancel` release a
claim that a different session holds. Do not make the user edit
`.superdev/cache/workflow.toml` by hand.

## Motivation

Superdev keeps transient ownership in `.superdev/cache/workflow.toml`. The file
records the owning Pi session. A session that stops without cancellation leaves
its claim in the file.

The Rust core can release such a claim, but only in one condition.
`cache::release()` accepts a different session if `cache::abandoned()` is true
(`crates/lib/superdev-core/src/workflow/cache.rs:321`). `abandoned()` calls
`liveness(owner_pid, owner_started)`. That function gives `Dead` only if the
file records both values. If the file records no process, the function gives
`Unknown`. The comment is correct that uncertainty must favour the incumbent.
But the effect is a claim that no session can release.

The Pi adapter stops the recovery a second time. `phase-tool.ts` refuses each
action except `inspect` when the owner session is different
(`.pi/extensions/superdev/lib/phase-tool.ts:48`). This test occurs before the
`cancel` branch. Thus `cancel` cannot release a foreign claim, although the
core permits it for a dead owner.

On 2026-09-09 the workflow for `plan-077-sokf-file-tool-parity` showed both
faults together. The file named session `01a0880f-24bc-7095-b6c3-c3ea227db82d`.
That process was not in the process list. The file recorded no `owner_pid`,
thus `abandoned()` gave false. The operations `run`, `retry`, and `cancel` all
failed with the same text: "This checkout is owned by another Pi session; pause
that session first." The message tells the user to pause a session that does
not exist. No tool operation could correct the state.

`start` and `resume` now send `--owner-pid`
(`.pi/extensions/superdev/index.ts:317` and `:512`). Thus a new claim is
decidable. A claim from an older version stays undecidable. A claim that comes
from a path with no process ID also stays undecidable.

## Sketch

Make three changes.

First, let `cancel` pass the adapter guard. Move the owner test after the
`cancel` branch, or exclude `cancel` from the test. The core keeps the safety
rule, because `release_unlocked()` continues to refuse a live foreign owner.
This agrees with `contract-011-interface-workflow P_cancel-pauses`: cancellation
releases transient ownership and does not change the canonical phase.

Second, make the process identity necessary at acquisition. Refuse a claim that
supplies no usable process identity, in the same way that `bind_unlocked()`
refuses an empty session or a bad authority digest. Then each new claim is
decidable, and `Unknown` shows only a claim from an older version.

Third, show the true condition to the user. `superdev workflow status` already
reports `abandonedOwner` (`crates/app/superdev/src/workflow_cli/dispatch.rs:30`),
but the Pi adapter does not read this field. Report a dead or undecidable owner
in the diagnostic. Name the recovery operation that is correct for that
condition.

## Trade-offs

- A claim that records no process stays undecidable. The change repairs new
  claims, but a user with an old claim still needs a manual step.
- A necessary process identity can refuse a claim on a platform that gives no
  process data. `start_identity()` gives `None` on each system that is not Unix
  (`crates/lib/superdev-core/src/workflow/process.rs:55`). Such a system needs
  a different rule, or it loses the ability to start a workflow.
- A relaxed guard on `cancel` increases the risk of a wrong release. The risk
  stays small, because the core continues to compare the session and the
  liveness before it removes the file.
- A process ID can repeat after a restart of the machine. `owner_started`
  limits this risk but does not remove it.

## Open questions

- Which recovery is correct for a claim that records no process? Options are a
  `--force` flag, a question in the trusted Pi UI, or a time limit.
- Must `status` report `abandonedOwner` to the adapter, and must the adapter
  show it in each failure message?
- Is a necessary process identity safe for each supported platform?
- Does the same fault stop `superdev workflow abandon` and the other operations
  that compare the session?
- Must the adapter release a dead claim automatically at `session_start`? It
  already does this for an orphaned child role
  (`.pi/extensions/superdev/index.ts:381`).

## Next step

Read `.pi/extensions/superdev/lib/phase-tool.ts:48` together with
`cache::release_unlocked()`. Decide if `cancel` can safely go through the guard.
This decision is small and gives the recovery path. The other two changes can
follow it.
