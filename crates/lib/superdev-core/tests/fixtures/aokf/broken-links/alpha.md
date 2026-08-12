---
type: Module
id: alpha
resource: /nowhere.rs
sources:
  - id: ghost-src
    resource: /also-missing.rs
    title: Missing source
links:
  - rel: depends-on
    to: nowhere-at-all
  - rel: references
    to: /ghost.md
---

# Role

A [broken](ghost.md) body link and an [external](https://example.com) one.[^ghost-src]

[^ghost-src]: Missing source
