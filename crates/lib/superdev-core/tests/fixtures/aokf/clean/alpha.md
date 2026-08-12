---
type: Module
id: alpha
title: Alpha
description: The first concept.
status: stable
tags: [parity]
resource: /beta.md
sources:
  - id: beta-src
    resource: /beta.md
    title: Beta source
verified:
  - { by: human:rsewell, at: "2026-08-04T09:00:00Z" }
links:
  - rel: depends-on
    to: beta
    note: Reads beta.
---

# Role

Alpha depends on [beta](beta.md).[^beta-src]

[^beta-src]: Beta source
