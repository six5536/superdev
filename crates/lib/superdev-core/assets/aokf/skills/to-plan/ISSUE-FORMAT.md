# Issue Format

Issues are AOKF concepts under `knowledge/issues/`, one file per issue,
named `Innn-<slug>.md`. Scan the directory for the highest number and
increment by one. The `id` is `issue-<slug>` and never changes. The
triage role is a frontmatter tag; the bundle's issue-tracker concept
carries the vocabulary.

## Template

```md
---
type: Issue
id: issue-{slug}
title: {Title}
description: {One line: the behaviour this issue makes work.}
tags: [ready-for-agent]
links:
  - rel: implements
    to: spec-{topic}
  - rel: blocked-by
    to: issue-{other}
---

# What to build

{The end-to-end behaviour this issue makes work, from the user's
perspective — not layer-by-layer implementation. Body links mirror the
`implements` and `blocked-by` edges.}

# Acceptance criteria

- [ ] Criterion 1
- [ ] Criterion 2

# Blocked by

- [{other issue}](Innn-{other}.md), or "None — can start immediately".
```

## Rules

- Publish in dependency order (blockers first), so every `blocked-by`
  edge targets a concept that exists.
- `blocked-by` is a custom rel: the frontier logic reads it exactly;
  consumers that don't know it read it as `relates-to`, which is safe.
- An issue produced by `/to-plan` is agent-grabbable by construction:
  tag it `ready-for-agent` unless instructed otherwise. Issues from
  other sources enter through triage and earn the tag there.
- A completed issue is deleted in the commit that completes it — git
  history is the archive, and the spec stays as the permanent record.
  An issue rejected as wontfix is deleted too; when the rejection
  reasoning is load-bearing, record it as a Decision concept first.
