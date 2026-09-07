---
type: Idea
id: idea-014-superdev-subprocesses-show-progress-and-control
title: Superdev subprocesses show progress and control
description: Make every user-awaited Superdev and Pi subprocess visibly report its stage, elapsed time, cancellation path, timeout, and final outcome.
status: draft
---

# Idea: Superdev subprocesses show progress and control

Give every user-awaited Superdev or nested Pi subprocess a visible lifecycle in
the Pi interface. A slash command must acknowledge dispatch immediately, show
which subprocess and stage are active, display elapsed time and cancellation
instructions, enforce a declared deadline, and report the final outcome.

## Motivation

`/scope` currently clears the editor and silently awaits an isolated Pi child.
The child can work for several minutes and modify the proposal without any
visible progress, making healthy execution indistinguishable from a hang. The
same failure mode applies to requirements review, BUILD, code review, ACCEPT,
and other commands that await hidden subprocesses.

## Sketch

Wrap user-awaited subprocess execution in one shared progress presenter. Use
Pi's existing notification, status, widget, and prompt APIs to expose stable
stages such as initialization, role execution, validation, review, and gate
approval. Include the plan or task identity, elapsed time, and the exact cancel
command. Consume structured child events to publish bounded progress without
showing model output or flooding the transcript.

Give each subprocess class an explicit timeout and distinguish timeout,
cancellation, child failure, malformed output, and successful completion.
Always clear or replace the progress UI in a `finally` path. Keep daemon-like
transport processes quiet unless startup fails or health changes.

## Open questions

- Which child events provide useful progress without exposing private reasoning
  or unstable model text?
- Should progress remain only in a status or widget, or should stage changes
  also create durable transcript entries?
- What timeout applies to each isolated role and non-model subprocess?
- Should one shared presenter also cover direct `superdev` service calls and
  Git operations that exceed a latency threshold?
