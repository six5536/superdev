---
type: Schema
id: schema-research
title: Research Schema
description: Research findings filed as footnote-cited concepts in knowledge/research/, derived from the research skill and the AOKF spec.
sources:
  - id: skill-research
    resource: /.claude/skills/research/SKILL.md
    title: Research Skill
  - id: aokf-spec
    resource: /.agents/aokf/SPEC.md
    title: AOKF Specification
---

# Research Schema

Structural rules for research findings filed at
`knowledge/research/research-{nnn}-{topic}.md` and listed in the bundle's
`index.md`. Derived from the research skill's own filing
statements[^skill-research] and the AOKF spec's frontmatter, sources, and
footnote mechanics[^aokf-spec]. Alone among these schemas it fixes no body
sections and declares no order: AOKF mandates neither, and the shape of an
answer follows the question.

````yaml
target-files: "knowledge/research/research-*.md"
description: >
  The findings of one researched question, with each claim attributed to
  a listed source by footnote.
line-limit: 800

frontmatter:
  type:
    const: Research
  id:
    pattern: '^research-\d{3}-[a-z0-9-]+$'
  sources:
    description: >
      Required here, unlike most concepts: every entry carries an id, and
      that id is the label of the footnote that cites it. The label is the
      join key into sources — keys, not positions.

sections:
  - heading-pattern: '^.+$'
    level: 1
    required: true
    repeatable: true
    description: >
      The findings, under headings of the writer's choosing — AOKF
      mandates no fixed body sections and prefers headings, lists, and
      tables over freeform prose. Each claim carries a footnote whose
      label matches a sources[].id entry, per the AOKF spec.

example: |
  ---
  type: Research
  id: research-001-git-scheme-matching
  title: How git matches pack source schemes
  description: How git parses and matches a URL transport scheme.
  sources:
    - id: git-clone-urls
      resource: https://git-scm.com/docs/git-clone#URLS
      title: git-clone URLS documentation
  ---

  # Scheme matching

  git takes the transport from the part of the URL before `://` and
  matches it against its known transports.[^git-clone-urls]

  [^git-clone-urls]: git-clone URLS documentation
````

[^skill-research]: Research Skill
[^aokf-spec]: AOKF Specification
