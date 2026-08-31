---
type: FeatureRequest
id: issue-030-feature-request-filing-an-issue-requires-framing-it
title: filing an issue requires framing it, so framings go stale before the work starts
description: The workflow has no lightweight filing — /frame does the full framing at creation, but framing belongs at the point the issue is taken up, because a framing made at filing can be out of date by the time the work starts.
lifecycle: open
---

# Feature: filing an issue requires framing it

## Summary

The only path into the tracker is `/frame`, which frames the issue in
full — goal, interview, EARS criteria, branch — at creation. A user who
just wants an issue recorded has no lighter path.

## Motivation

Framing captures decisions against the project as it stands. An issue
may sit in the tracker for a long time before it is worked on, and the
project moves meanwhile, so a framing made at filing can be out of date
when the work starts. The three issues filed today (I028–I030) each
needed exactly this: a brief record now, framing later.

## Proposed behaviour

Asking for an issue creates the tracker record without framing — a
brief report with TBDs where the shape's sections are not yet settled.
Framing runs by default when the issue is taken up for work, and an
issue's framing may be revised over time. Details: TBD at framing.

## Acceptance criteria

1. TBD — whether this is a new skill, a mode of `/frame`, or a change
   to the workflow's entry conditions.
2. TBD — what the minimum filed issue must contain.
3. TBD — how "taken up for work" triggers framing.

## Alternatives considered

- Not yet thought through; alternatives are settled at framing.

## Scope

- In: TBD.
- Out: TBD.
