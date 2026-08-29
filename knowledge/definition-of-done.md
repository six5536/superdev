---
type: DefinitionOfDone
id: definition-of-done
title: Definition of Done
description: What a change must satisfy before it merges.
status: stable
sources:
  - id: pr-template
    resource: /.github/PULL_REQUEST_TEMPLATE.md
    title: PR checklist
---

A change is done when (the enforced form is the
[PR checklist](/.github/PULL_REQUEST_TEMPLATE.md)):[^pr-template]

- Formatting, clippy (`--all-targets -- -D warnings`), tests, and doctests
  pass.
- Line coverage stays ≥ 90% **per crate** — see
  [testing-strategy][sokf:testing-strategy].
- Launcher and version-consistency checks pass.
- The SOKF knowledge and every governed file validate (`npm run check:validate`, when
  `knowledge/`, `.agents/` or a skill changed) and the repo matches its own
  blueprint (`npm run check:blueprint`).
- Documentation is updated wherever behaviour changed: README, this
  knowledge, rustdoc.
- New behaviour carries tests at the appropriate layer; bug fixes carry a
  regression test that fails on the unfixed code.

[^pr-template]: PR checklist

<!-- sokf:links -->
[sokf:testing-strategy]: /knowledge/testing-strategy.md
