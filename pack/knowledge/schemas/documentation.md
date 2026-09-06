---
type: Schema
id: schema-documentation
title: Documentation Map Schema
description: The project-specific inventory of documentation audiences, sources, generated targets, triggers, and executable generation and verification commands.
---

# Documentation Map Schema

Every managed project carries one `Documentation` concept at
`knowledge/documentation.md`. It is the reviewed boundary used by SCOPE to
identify affected documentation and by BUILD and ACCEPT to require executable
evidence. Publication ownership is informational; deployment is not part of the
local workflow.

````yaml
description: >
  The project-specific inventory of documentation surfaces and their objective
  applicability, source-of-truth, generation, and verification rules.
line-limit: 1500
frontmatter:
  type:
    required: true
    const: Documentation
  id:
    required: true
    const: documentation
  title:
    required: true
  description:
    required: true
sections-ordered: true
sections:
  - heading: "Documentation map"
    level: 1
    required: true
    description: The document title.
  - heading: "Policy"
    level: 2
    required: true
    content: prose
    description: How SCOPE, BUILD, and ACCEPT consume this map.
  - heading: "Surfaces"
    level: 2
    required: true
    description: Container for one or more stable named surfaces.
  - heading-pattern: '^Surface: [a-z0-9-]+$'
    level: 3
    required: true
    repeatable: true
    content: bullet-list
    description: One audience-facing surface with all ten declaration fields.
    item-only-pattern: '^- (Audience and purpose|Kind|Authored sources|Generated outputs|Source of truth|Triggers|Generation command|Verification command|Generated output|Publication owner): .+$'
example: |
  ---
  type: Documentation
  id: documentation
  title: Documentation map
  description: Project documentation surfaces and their executable maintenance policy.
  ---

  # Documentation map

  ## Policy

  Every declared trigger is evaluated during SCOPE. BUILD changes authored
  sources, regenerates derived output, and runs the named checks.

  ## Surfaces

  ### Surface: readme

  - Audience and purpose: prospective and current users learning the product.
  - Kind: handwritten.
  - Authored sources: `/README.md`.
  - Generated outputs: none.
  - Source of truth: `/README.md`.
  - Triggers: public contracts, CLI, API, configuration, and migration changes.
  - Generation command: none.
  - Verification command: `check-docs` in `development-commands`.
  - Generated output: not applicable.
  - Publication owner: maintainers; publication is release work.
````
