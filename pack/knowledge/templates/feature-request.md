---
type: Template
id: template-feature-request
title: Feature Request Template
description: Motivation, proposed behaviour, alternatives considered and scope. One of the three shapes the issue tracker holds.
status: stable
---

---
type: FeatureRequest
id: issue-nnn-<slug>
title: <one line — what is missing>
description: <one line — what does not exist, and who it blocks>.
status: draft
tags: [needs-triage]
---

# Feature: <what is missing>

## Summary

<One or two sentences: what does not exist, and who is blocked or slowed
by its absence.>

## Motivation

<Why this is wanted now, with the evidence: the case that hit it, the
count that makes it worth doing, or the rule it would let the project
keep. Measure the absence rather than asserting it.>

## Proposed behaviour

<What exists once this is done, described so a reader could recognise
it. Behaviour, not implementation.>

## Alternatives considered

- <Option not taken> — <the single reason it lost>.
- <Option not taken> — <the single reason it lost>.

## Scope

- In: <what this covers>.
- Out: <what is deliberately excluded, and where it is handled instead>.

---

Notes on usage (not part of the document):

- File as `knowledge/issues/issue-<nnn>-feature-request-<slug>.md`, numbered after the
  highest existing issue. Declare the feature, when there is one, with
  an `implements` or `references` link to its spec.
- The `issue-tracker` concept holds the triage labels and lifecycle:
  the role tag rides in `tags`, and a resolved issue stays, retagged
  `done` or `wontfix`.
