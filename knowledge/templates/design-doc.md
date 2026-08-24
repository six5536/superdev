---
type: Template
id: template-design-doc
title: Design Doc Template
description: Problem, goals, proposed design with architecture and key flows, alternatives considered, and cross-cutting concerns.
status: stable
---

# Design: <feature or system name>

- Status: draft | in review | approved | implemented
- Author: <name>
- Date: <YYYY-MM-DD>
- Reviewers: <names>

## Problem

<What problem exists, for whom, and why it matters now. Concrete symptoms over abstractions.>

## Goals

- <Measurable outcome 1>
- <Measurable outcome 2>

Non-goals:
- <Explicitly out of scope>

## Background

<Existing system behavior a reader needs to evaluate the design. Reference code as `path/to/file.ts:123`. Keep it to what's load-bearing.>

## Proposed design

<The design itself. Start with a one-paragraph overview, then detail:>

### Architecture

<Components and how they interact. A diagram (mermaid) if the topology isn't obvious from prose.>

### Data model / API

<Schemas, types, endpoints, wire formats — whatever contracts this design introduces or changes.>

### Key flows

<Walk through the 1–3 most important scenarios end to end.>

## Alternatives considered

### <Alternative A>

<What it is, and the specific reason it lost — cost, risk, complexity, doesn't meet a goal.>

### <Alternative B>

<...>

## Cross-cutting concerns

- Security: <authn/authz, data exposure, input validation>
- Performance: <expected load, hot paths, scaling limits>
- Migration/rollout: <how we get from current state to this, and how we roll back>
- Observability: <what we log/measure to know it's working>

## Open questions

- <Question> — <recommended answer and who decides>
