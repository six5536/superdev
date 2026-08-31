---
type: Schema
id: schema-visual-system
title: Visual System Schema
description: The design tokens later slices build against, in knowledge/visual-system.md.
---

# Visual System Schema

Structural rules for `knowledge/visual-system.md`, the canonical knowledge's Convention
concept for the design tokens a UI is built against. Only projects with a UI
carry this concept; changing a token here is a decision, not a tweak.

````yaml
description: >
  The design tokens every UI slice builds against — palette, type roles,
  layout and spacing, the signature element, and the component library.
line-limit: 800

frontmatter:
  type:
    required: true
    const: VisualSystem
  id:
    required: true
    const: visual-system
  title:
    required: true
  description:
    required: true
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: prose
  description: >
    The aesthetic direction in one or two sentences: the point of view, and
    the one risk it takes. Every UI slice builds against these tokens;
    changing them is a decision, not a tweak.

sections-ordered: true
sections:
  - heading: "Palette"
    level: 1
    required: true
    content: bullet-list
    description: >
      One bullet per token: the token name, its hex value, and its role —
      background, text, accent. Four to six named values in all.
  - heading: "Type"
    level: 1
    required: true
    content: bullet-list
    description: >
      Display, Body and, where needed, Utility: the typeface and where it
      appears, with the restraint on it. Then the scale — sizes, weights and
      spacing worth naming.
  - heading: "Layout & spacing"
    level: 1
    required: true
    content: prose
    description: >
      The layout concept in a sentence, and the spacing scale or grid the
      components sit on.
  - heading: "Signature"
    level: 1
    required: true
    content: prose
    description: >
      The single element this design is remembered by, and where it appears.
  - heading: "Component library"
    level: 1
    required: true
    content: prose
    description: >
      The library or component set in use, and the rule for adding to it.

example: |
  ---
  type: VisualSystem
  id: visual-system
  title: Visual System
  description: The design tokens the UI is built against — quiet, dense, one accent.
  status: stable
  ---

  Quiet and dense: a reader should see data, not chrome. The one risk it
  takes is a single saturated accent on an otherwise neutral ground, which
  means an accidental second accent reads as a bug.

  # Palette

  - `ground` `#0F1115` — page background
  - `surface` `#171A21` — cards and panels
  - `ink` `#E6E8EC` — primary text
  - `ink-muted` `#8A909B` — secondary text and labels
  - `accent` `#3D7EFF` — the one accent: links, focus, primary action

  # Type

  - Display: Inter Tight — headings only, never below 20px.
  - Body: Inter — everything else.
  - Utility: JetBrains Mono — code, ids and tabular figures.

  Scale 12 / 14 / 16 / 20 / 28, weights 400 and 600 only.

  # Layout & spacing

  A single 12-column grid with a 1200px cap, on a 4px spacing scale. Nothing
  sits on a value off that scale.

  # Signature

  The accent left-edge rule on the active row — the one place colour appears
  in a list, and how a reader finds their place after scrolling.

  # Component library

  Radix primitives, styled locally. A new component is added only when two
  screens need it; the first screen styles it inline.
````
