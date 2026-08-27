---
type: Schema
id: schema-security-requirements
title: Security Requirements Schema
description: The vulnerability policy in brief and the security guarantees the design makes, in knowledge/security-requirements.md.
---

# Security Requirements Schema

Structural rules for `knowledge/security-requirements.md`, the bundle's
Policy concept for security. Each guarantee carries its enforcing mechanism,
which is what makes a change that weakens one recognisable in review.

````yaml
target-files: "knowledge/security-requirements.md"
description: >
  Where vulnerability reports go and which versions get fixes, and the
  security-relevant guarantees the design makes, each with its mechanism.
line-limit: 800

frontmatter:
  type:
    const: Policy
  id:
    const: security-requirements
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: prose
  description: >
    The reporting policy in brief: where vulnerability reports go, and which
    versions get fixes. Link the full policy file.

sections-ordered: true
sections:
  - heading: "Guarantees the design makes"
    level: 1
    required: true
    content: bullet-list
    description: >
      One bullet per guarantee: the security-relevant guarantee, then the
      mechanism that enforces it, so a change that weakens it is recognisable.

example: |
  ---
  type: Policy
  id: security-requirements
  title: Security Requirements
  description: The vulnerability policy in brief, and the guarantees the design makes.
  status: stable
  ---

  Vulnerability reports go to security@example.org, not to the issue
  tracker. The current minor version gets fixes; older minors do not. The
  full policy is in `SECURITY.md`.

  # Guarantees the design makes

  - Pack content never arrives over an unauthenticated channel. The manifest
    parser refuses any transport outside https, ssh and file, at parse time
    and before any fetch — so widening the allowlist is a visible one-line
    change rather than an emergent behaviour.
  - A pack can never write outside the cache directory. Every extracted path
    is normalised and re-rooted before it is opened, so an archive carrying
    `../` escapes nothing.
````
