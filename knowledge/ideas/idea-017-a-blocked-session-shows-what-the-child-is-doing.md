---
type: Idea
id: idea-017-a-blocked-session-shows-what-the-child-is-doing
title: A blocked session shows what the child is doing
description: While an isolated role holds the session, show enough of its reasoning and actions for the user to judge whether it is on track, rather than a timer and a tool name.
status: draft
---

# Idea: a blocked session shows what the child is doing

While `superdev_run_phase` runs an isolated role, the main session is blocked
and shows a timer, a stage, and the current tool name. The user cannot tell a
role that is working from one that has misunderstood the task, so the only
cancellation signal available is elapsed time. Show enough of the child's
activity, in the space the blocked session already occupies, for the user to
decide whether to let it continue.

## Motivation

A phase can hold the session for ten minutes. During that time the progress
panel offers `scope authoring · 1:55` and a line such as
`read knowledge/plans/open/plan-077-sokf-file-tool-parity.md`. That says the
role is alive and touching a file. It does not say what the role concluded,
what it is trying to do, or whether it has gone somewhere the user would have
redirected at minute one.

The user pays for the whole run either way, because the session is blocked. A
role that misread the outcome burns the full budget before returning a result
the user then discards. Cancelling early is only useful if the user can tell
early, and the current display gives no basis for that judgement.

The information already exists. `isolated()` in
`.pi/extensions/superdev/lib/process.ts` parses the child's event stream
incrementally and already extracts, per assistant turn, the bounded assistant
text, the stop reason, the tool-call count, and the tool names. It records a
terminal timeline of tool starts and ends. All of it is retained for the
failure diagnostic and the artifact, and none of it reaches the user while the
run is in progress. Only `onActivity` is forwarded live, carrying one
sanitized tool name and path.

## Sketch

Forward more of what the stream already yields, rather than parsing anything
new. The parser could emit each completed assistant turn's bounded text
alongside the existing activity callback, and the panel could show the most
recent one or two lines beneath the stage.

Where that text belongs is the open part. The blocked session has room the
panel does not currently use, and Pi offers more than one idiomatic home for
it: a taller custom component, a transcript entry rendered by
`registerEntryRenderer` so it stays out of model context, or a widget through
`ctx.ui.setWidget`. A per-turn transcript entry is attractive because it
survives scrollback, so a user who looks away can still read what happened,
and it costs the parent no context.

Whatever the surface, the same three bounds apply that the artifact path
already respects. Reasoning, prompts, and tool arguments stay out. Text stays
bounded per turn and in total. Nothing shown enters the parent model's
context, because the point is to inform the human, not to feed the parent.

An expansion control would let the default stay compact: one line while the
user is content, more on request. Pi's keybinding and component facilities
supply that without a new mechanism.

## Trade-offs

- Streaming a role's intermediate reasoning invites the user to intervene on a
  turn that the role would have corrected by itself, which trades one kind of
  wasted run for another.
- More visible text is more transcript noise, so the default view has to stay
  compact and the detail has to be opt-in.
- Bounded per-turn text can mislead by omission: a truncated conclusion may
  read as more certain, or more wrong, than the full one.
- Any live surface competes for the same rows as the progress panel, the
  prompt, and the footer, so this is coupled to whatever shape that panel
  settles on.
- Showing the child's text raises the chance a user acts on an intermediate
  claim that never became part of the typed result.

## Open questions

- Which surface is idiomatic here: a taller custom component, a rendered
  transcript entry, a widget, or a combination?
- Is the right unit a completed assistant turn, or a coarser one such as a
  stage change or a decision the role reports deliberately?
- Should a role emit progress explicitly through its terminating tool's
  vocabulary, instead of the parent inferring it from assistant text?
- Does the parent risk drifting into judging a role's intermediate work, when
  judging belongs to the role that performs it?
- Does this belong to the progress panel's shape, filed as its own issue, or
  is it separate enough to stand alone?

## Next step

Decide the unit before the surface. If a completed assistant turn is the right
granularity, the parser already produces it and the remaining work is
presentation. If it is not, the terminating-tool vocabulary changes first and
the display follows.
