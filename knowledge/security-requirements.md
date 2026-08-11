---
type: Policy
id: security-requirements
title: Security Requirements
description: The vulnerability policy in brief; the security surface is TBD with the design.
status: draft
sources:
  - id: security-md
    resource: /SECURITY.md
    title: Security policy
---

The full policy is [SECURITY.md](/SECURITY.md).[^security-md] In brief:
vulnerabilities are reported privately via GitHub's private vulnerability
reporting, never as public issues; fixes target the latest release and `main`
only (pre-1.0, no backports).

What counts as superdev's attack surface depends on the
[architecture](architecture.md), which is not yet defined. Record the
security-relevant guarantees here as they are designed.

[^security-md]: Security policy
