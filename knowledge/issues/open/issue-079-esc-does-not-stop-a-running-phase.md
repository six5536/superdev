---
type: Issue
id: issue-079-esc-does-not-stop-a-running-phase
title: Esc does not stop a running workflow phase
description: The progress panel invites the user to press Esc to stop a running phase, but the key is matched by a raw byte comparison that misses the terminal's actual escape sequences, so the phase continues.
kind: bug
lifecycle: open
---

# Bug: Esc does not stop a running workflow phase

## Summary

`superdev_run_phase` tells the user that Esc stops the running phase and
preserves partial work. Pressing Esc does not stop it. A user who wants to
interrupt a long review or build has no working control and waits for the
timeout or kills the session.

## Context

Reported while a SCOPE requirements review was running on
`work/077-sokf-file-tool-parity`. The progress panel offered the control on
its last line:

```text
SCOPE plan-077-sokf-file-tool-parity
requirements review · 1:44
read
Esc to stop and preserve partial work
```

The cancellation path itself is wired. `withProgress` in
`.pi/extensions/superdev/lib/progress.ts` owns an `AbortController` and
aborts it when its custom component reports `cancel`. That abort reaches the
phase drivers through `newAbort` and `phaseSignal` in
`.pi/extensions/superdev/lib/phases.ts`, and reaches each child process
through the `signal` argument of `runPinnedSuperdev` in
`.pi/extensions/superdev/lib/process.ts`. Only the key detection is at fault.

The component compares the raw input to one byte:

```typescript
handleInput(data: string) { if (data === "\u001b") done("cancel"); }
```

Pi's own guidance is to match keys with `matchesKey(data, Key.escape)` from
`@earendil-works/pi-tui`, which the extension imports nowhere. A bare `\u001b`
arrives only when the terminal sends an unadorned escape byte. Terminals that
send an escape sequence, and the Kitty keyboard protocol that Pi supports,
encode the key differently, so the comparison fails and `done("cancel")` never
runs.

## Behaviour

Pressing Esc while a phase is running stops that phase, releases workflow
ownership, and preserves partial work, exactly as the panel states. The same
key works across supported terminals and under the Kitty keyboard protocol.

Instead the keypress is ignored and the phase runs to completion or timeout.

The status line offers `Esc or /superdev-cancel to interrupt`
(`.pi/extensions/superdev/index.ts`). `/superdev-cancel` does work, so a user
who reads that line has a way out; a user who reads the progress panel alone
does not.

Two further points bear checking while the key is fixed. The panel promises
Esc unconditionally, including where no interactive UI exists and the panel is
never drawn. Cancellation is also offered by the phase tool as a recovery
operation, so the key, the command, and the tool operation should agree on what
stopping means.

## Scope

The interruption control for a running phase.

- In: how the progress component detects the key, and any place that promises
  the key without honouring it.
- Out: what cancellation does once triggered. Ownership release, partial-work
  preservation, and child termination already behave correctly and are proven
  by the existing cancellation paths.
- Out: the redundant status presentation filed as issue 78, which touches the
  same panel for an unrelated reason.

## Comments

Filed from a live session. The user pressed Esc during an active SCOPE
requirements review and the review continued.
