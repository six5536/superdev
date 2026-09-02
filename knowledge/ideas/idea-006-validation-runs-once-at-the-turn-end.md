---
type: Idea
id: idea-006-validation-runs-once-at-the-turn-end
title: Validation runs once, at the turn's end
description: Drop the PostToolUse validation hook and leave the Stop hook as the only gate, so an agent pays the validator's cost once a turn instead of once an edit.
status: draft
---

# Idea: validation runs once, at the turn's end

Remove the `PostToolUse` hook matching `Edit|Write`, and leave `hook run` on
`Stop` as the only place validation gates. An agent that writes eight documents
in a turn runs the validator once, at the end, instead of eight times along the
way.

## Motivation

Every `Edit` and `Write` under a governed tree runs `superdev hook validate`,
and the agent waits for it before its next call. The cost is paid per edit and
is most visible in exactly the turns that do the most writing — framing an
issue, cutting a plan, filing a set of ADRs and relinking the indexes that name
them.

The `Stop` hook already covers the same ground, on every turn.
`P_hooks-share-validate-default` holds both hooks to what `validate` reports by
default, and `P_hook-run-holds-on-error` refuses to end the turn while
`validate` reports an error. That gate does not depend on an unattended run:
`hook_run_on` calls `knowledge_hold` before it reads the run state, "whether or
not a run is armed" (`crates/app/superdev/src/run.rs:259`). The turn is already
the gate; the per-edit run mostly reports the same findings earlier and slower.

## Sketch

The SOKF component owns both registrations, each found by its command string
and locked under `.claude/settings.json:hooks.<array>[<marker>]`. Dropping the
per-edit hook means dropping `HOOK_POINTER`, `HOOK_MARKER` and `HOOK_ELEMENT`
from `crates/lib/superdev-core/src/components/sokf.rs`, leaving the `STOP_`
trio, and retiring the lock key so an existing repo's entry is swept rather
than left orphaned. Editing `.claude/settings.json` by hand does nothing
durable: the managed element is restored on the next apply, and the edit is
reported as drift first.

The `hook validate` subcommand stays in the CLI and stays contracted — it is
still the right hook for a project that wants per-edit enforcement — so this
changes what superdev installs, not what the binary can do.

## Trade-offs

- Attribution is lost. A per-edit hook names the edit that caused a finding;
  findings arriving at the turn's end leave the agent to work out which of its
  writes is at fault.
- More work lands inside `HOLD_CAP`. Every finding a turn produces now surfaces
  in one held window, and `P_hook-run-hold-cap` ends the turn once that window
  is spent — so a turn with many findings can end with some unresolved.
- A malformed document survives longer in the working tree, and any tool
  reading it mid-turn reads it broken.
- It cuts against ADR-018, which places enforcement in the hook. The split
  survives — enforcement stays in a hook — but the reasoning that picked the
  edit as its moment needs rereading.

## Open questions

- Does the latency come from process start, from reading the tree, or from the
  checks themselves? A cheaper `hook validate` would settle this without moving
  it.
- Is `HOLD_CAP` large enough for a turn's worth of findings, given it was sized
  against a turn that had already been validating per edit?
- Should the per-edit hook stay for a narrower matcher — schemas and contracts,
  say — rather than go entirely?

## Next step

Measure `superdev hook validate` on this repository: wall time for one edit,
split between startup and check. The number decides whether this is a hook
placement question or a validator performance one.
