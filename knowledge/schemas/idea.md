---
type: Schema
id: schema-idea
title: Idea Schema
description: A thought captured for considering later — what it is, plus whatever reasoning exists at the time — filed in knowledge/ideas/.
---

# Idea Schema

Structural rules for one idea, filed at
`knowledge/ideas/idea-{nnn}-{slug}.md` and listed in that directory's index.

This is a capture bin, and nothing here is official. An idea is a thought
worth writing down before it is lost, kept for considering later. It is not
candidate work, so it does not belong in the
[issue tracker][sokf:issue-tracker], and it carries no obligation to appear
there. An idea that is taken up stays on file: `/file` promotes it into an
open issue, and the issue links the idea with `references` (ADR-050).

One section is required, because the cost of capture decides how much gets
captured, and a thought is lost while its author is filling in headings. Most
ideas will carry the first section and nothing else. The rest exist so that an
idea someone does work out has agreed headings, in an agreed order, instead of
a shape invented each time.

````yaml
description: >
  A thought captured for considering later — what it is, plus as much of the
  motivation, sketch, trade-offs and next step as exists at the time.
line-limit: 400

frontmatter:
  type:
    required: true
    const: Idea
  id:
    required: true
    pattern: '^idea-\d{3}-[a-z0-9-]+$'
    description: >
      idea-{nnn}-{slug}, the slug naming the idea rather than the problem, so
      two ideas about one problem stay apart.
  title:
    required: true
  description:
    required: true
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^Idea: .+$'
    level: 1
    required: true
    content: prose
    description: >
      The idea itself, in as many sentences as it takes and no more. Written
      so its author recognises it months later, which is the only thing this
      document has to do.
  - heading: "Motivation"
    level: 2
    content: prose
    description: >
      What is wrong or missing now, where it came up, or simply what prompted
      the thought. Optional, like everything below it.
  - heading: "Sketch"
    level: 2
    content: prose
    description: >
      How it might work — the mechanism, where it would live, what would have
      to change. Enough to judge, never enough to build: a design belongs in a
      feature request and its contracts, which is where an idea goes once
      it is taken up.
  - heading: "Trade-offs"
    level: 2
    content: bullet-list
    description: >
      What it would cost, what it risks, and what it rules out. Worth adding
      the moment one is obvious, because that is what a later reader needs and
      what the author will have forgotten.
  - heading: "Open questions"
    level: 2
    content: bullet-list
    description: >
      What is not yet known, one per line. Often the whole of an idea on the
      day it is captured.
  - heading: "Next step"
    level: 2
    content: prose
    description: >
      The one thing that would move this — a spike, a measurement, a decision
      to drop it. Absent until someone has thought about it, which is the
      normal state of an idea.

example: |
  ---
  type: Idea
  id: idea-001-widgets-cache-their-render
  title: Widgets cache their render
  description: Keep the rendered form of a widget beside it, so a list of a thousand does not re-render each.
  status: draft
  ---

  # Idea: widgets cache their render

  Store each widget's rendered HTML beside the widget, written when the widget
  is saved rather than when it is read. A list view then concatenates strings
  instead of running the renderer once per row.

  ## Motivation

  The list view renders every widget on every request. At a thousand widgets it
  spends most of its time in the renderer, and the renderer's output depends
  only on the widget — the same input produces the same output every time.

  ## Sketch

  A `rendered` column on `widget`, written in the same transaction as the
  widget itself, so the two cannot disagree. The list view reads it directly.
  A renderer change means a backfill, which is a migration like any other.

  ## Trade-offs

  - The store grows by roughly the size of the corpus again.
  - Every renderer change becomes a migration, which is a cost paid by whoever
    changes the renderer rather than by the list view.
  - It rules out per-viewer rendering: a widget that renders differently for
    different readers cannot be cached this way without a key per reader.

  ## Open questions

  - Is the renderer genuinely pure? It reads the tenant's locale today.
  - Would an in-memory cache do, given the list view is the only hot reader?

  ## Next step

  Measure. A profile of the list view at a thousand widgets will say whether
  the renderer is the cost, and the locale question decides whether the cache
  key is the widget alone.
````

<!-- sokf:links -->
[sokf:issue-tracker]: /knowledge/issue-tracker.md
