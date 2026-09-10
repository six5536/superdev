---
type: Idea
id: idea-017-a-blocked-session-shows-what-the-child-is-doing
title: A blocked session shows what the child is doing
description: While an isolated role runs, show enough of its reasoning and actions for a user to judge whether it is on track, and let any session answer that question about a role another session owns.
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

One episode on 2026-09-09 confirms both which facts answer the question and
how expensive they currently are to obtain. A `requirements-review` role for
`plan-077-sokf-file-tool-parity` had run for 14 minutes and the user asked
whether to let it continue. Answering took reading `/proc` for the child's
process state, then parsing a 2.6 MB `events.jsonl` artifact by hand.

Three facts settled the question, and each was already parsed in the running
parent:

- The role's own narration, three lines across 18 turns: it read the issue and
  plan, then the contracts, ADR, and originating idea. That is the role stating
  its plan in its own words.
- The tool trajectory: 27 reads, 6 searches, one graph traversal, no errors and
  no repeats, so not stalled and not looping.
- The trajectory's direction. The most recent reads had left the plan's subject
  for Pi's installed package internals, and two searches named repository-root
  discovery, which issue 77 does not ask about. That drift was the only
  actionable signal in fourteen minutes, and it was invisible.

Elapsed time said none of this. The narration and the trajectory did, and both
were in memory the whole time.

The same episode showed a second gap. The role belonged to a different Pi
session, and the asking session had no way to see its state. It reconstructed
the answer from the transient claim on disk, `/proc`, and a private artifact
directory. `superdev workflow status` reports the owning session, the child
role, and the child process, which is enough to know a role is running and
whether it is alive, and nothing about what it is doing.

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

The cross-session case wants the same facts through a different door. A
running role already writes its event stream to an artifact directory, and the
service already records which session owns it and which process is running it.
A read-only status view could summarize that stream for whoever asks: the
role, its elapsed time, its turn and tool counts, its recent tool targets, and
its latest narration, without joining or disturbing the run.

Where the summary is produced matters more than where it is shown. The parent
that owns the child has the parse in memory but cannot be interrupted while
blocked. A second session has neither the parse nor any right to the artifact,
which is owner-private today. Either the owner publishes a bounded rolling
summary that any session may read, or the service learns to summarize the
artifact on demand. The first keeps parsing in one place; the second keeps the
owner free of a duty it cannot perform while blocked.

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
- A cross-session status view widens what a non-owning session may read. The
  artifact is owner-private today, and a rolling summary that any session can
  read is a deliberate relaxation rather than a free addition.
- A summary produced by the owner is cheap but unavailable exactly when the
  owner is blocked, which is when it is wanted. A summary produced on demand
  by the service duplicates parsing that already happens once.

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
- Should the owning parent publish a rolling summary, or should the service
  summarize the artifact when a session asks?
- Does a running role's narration belong in `superdev workflow status`, or in
  a separate read-only inspection surface?
- What may a non-owning session read? The artifact is owner-private, and
  widening that is a decision rather than a detail.
- Is drift from the plan's subject something the parent can report, or only
  something a human recognises from the trajectory?

## Next step

Decide the unit before the surface. If a completed assistant turn is the right
granularity, the parser already produces it and the remaining work is
presentation. If it is not, the terminating-tool vocabulary changes first and
the display follows.

The confirming episode suggests the unit: narration plus the recent tool
targets answered the question, and neither needed a new event. Deciding
whether the owner publishes that or the service derives it settles the
cross-session half at the same time.
