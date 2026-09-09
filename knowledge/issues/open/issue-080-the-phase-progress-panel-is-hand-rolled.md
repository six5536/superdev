---
type: Issue
id: issue-080-the-phase-progress-panel-is-hand-rolled
title: The phase progress panel is hand-rolled instead of framed like the rest of Pi
description: superdev_run_phase draws its own unframed stack of lines with no spinner and a hardcoded key hint, so a running phase looks unlike every other Pi activity display and reads as loose text above the prompt.
kind: feature
lifecycle: open
links:
  - rel: references
    to: issue-078-workflow-progress-repeats-plan-and-stage
    note: The same panel, for the duplicated plan and stage rather than its shape.
  - rel: references
    to: issue-079-esc-does-not-stop-a-running-phase
    note: Adopting Pi's cancellable loader would replace the key comparison that bug reports.
---

# Feature: the phase progress panel is framed like the rest of Pi

## Summary

`superdev_run_phase` draws a running phase as a bare stack of lines with no
frame, no spinner, and a hardcoded key hint. A user watching a long phase sees
loose text that does not read as one unit and does not resemble any other Pi
activity display. Frame the panel so its start and end are visible and its
motion shows the phase is alive.

## Context

Observed while SCOPE authored `plan-077-sokf-file-tool-parity` on
`work/077-sokf-file-tool-parity`:

```text
SCOPE plan-077-sokf-file-tool-parity
scope authoring · 1:55
read knowledge/plans/open/plan-077-sokf-file-tool-parity.md
Esc to stop and preserve partial work
/workspaces/superdev (work/077-sokf-file-tool-parity)
[Pi usage and model footer]
SCOPE plan-077-sokf-file-tool-parity · scope authoring
```

The user proposed this shape instead:

```text
-- <spinner> scope authoring · 1:55 --------------------------- Esc cancels ----
read knowledge/plans/open/plan-077-sokf-file-tool-parity.md
--------------------------------------------------------------------------------
/workspaces/superdev (work/077-sokf-file-tool-parity)
[Pi usage and model footer]
SCOPE plan-077-sokf-file-tool-parity · scope authoring
```

The panel is built by hand in `withProgress`
(`.pi/extensions/superdev/lib/progress.ts`). It pushes a title line, a
`stage · clock` line, an optional activity line, and a literal
`Esc to stop and preserve partial work` line, then returns them from `render`.
It draws no border and no spinner, and a one-second interval only re-renders
the clock.

Pi already ships the shape the user drew. `BorderedLoader`, exported from
`@earendil-works/pi-coding-agent` and documented in `docs/tui.md` as the
pattern for a cancellable async operation, composes a `DynamicBorder` that
spans the viewport width, a `CancellableLoader` that animates a spinner beside
a message, and a `keyHint` line. The extension imports none of it.

## Behaviour

A running phase appears as one framed unit: a rule above and below, a spinner
that moves while work continues, the stage and elapsed time, the current
activity, and a cancellation hint. The frame makes the panel's extent obvious
against the surrounding prompt and footer, and the spinner distinguishes a
working phase from a stalled one.

The cancellation hint names whatever key is currently bound rather than the
string `Esc`. Pi resolves that through `keyHint`, so a user who has remapped
the binding sees their own key. The present panel hardcodes the word and would
mislead such a user even once the key itself works.

Adopting Pi's component is worth preferring over restyling the hand-rolled
lines, because the same change removes two other defects at their source:

- `CancellableLoader` matches the key with `getKeybindings().matches(data,
  "tui.select.cancel")`, which is the correct detection that
  [the Esc bug][sokf:issue-079-esc-does-not-stop-a-running-phase] reports
  missing.
- A framed panel that owns the stage and elapsed time gives
  [the duplicated status bug][sokf:issue-078-workflow-progress-repeats-plan-and-stage]
  a natural place to stop repeating them in the status line.

Where a loader message cannot carry the stage, clock, and activity together,
the panel may keep its own component; the framing, motion, and resolved key
hint are what this issue asks for.

## Scope

The visual shape of the running-phase panel.

- In: framing, spinner, layout of stage, elapsed time and activity, and how
  the cancellation hint is produced.
- Out: what cancellation does once triggered, and how the key is detected.
  [The Esc bug][sokf:issue-079-esc-does-not-stop-a-running-phase] owns the key;
  this issue only observes that Pi's component would supply a correct one.
- Out: which text the status line repeats.
  [The duplicated status bug][sokf:issue-078-workflow-progress-repeats-plan-and-stage]
  owns that.
- Out: workflow transitions, review requirements, and issue 77's SOKF
  file-tool work.

## Comments

Filed from a live session, with the user's proposed layout preserved verbatim
above. Filed as a feature rather than a bug: the panel behaves as written, and
what is absent is a display shape the project has not yet adopted.

<!-- sokf:links -->
[sokf:issue-078-workflow-progress-repeats-plan-and-stage]: /knowledge/issues/open/issue-078-workflow-progress-repeats-plan-and-stage.md
[sokf:issue-079-esc-does-not-stop-a-running-phase]: /knowledge/issues/open/issue-079-esc-does-not-stop-a-running-phase.md
