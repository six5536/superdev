---
type: Schema
id: schema-thing
title: Thing Schema
description: A governed thing of two kinds, one of which carries a section the other does not.
---

# Thing Schema

Structural rules for a thing. `kind` selects the variant: a kind `a` thing
carries "Only A", a kind `b` thing does not, and both carry "Shared" and
"Tail" in that order (ADR-045).

````yaml
description: A governed thing of two kinds.

variant-key: kind

frontmatter:
  type:
    required: true
    const: Thing
  kind:
    required: true
    enum: [a, b]

sections-ordered: true
sections:
  - heading: "Shared"
    level: 2
    required: true
    content: prose
  - heading: "Only A"
    level: 2
    required: true
    variants: [a]
    content: prose
  - heading: "Tail"
    level: 2
    required: true
    content: prose

example:
  a: |
    ---
    type: Thing
    kind: a
    ---

    # A thing of kind a

    ## Shared

    Every kind carries this.

    ## Only A

    Kind a alone carries this.

    ## Tail

    Every kind ends here.
  b: |
    ---
    type: Thing
    kind: b
    ---

    # A thing of kind b

    ## Shared

    Every kind carries this.

    ## Tail

    Every kind ends here.
````
