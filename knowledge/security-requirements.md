---
type: Policy
id: security-requirements
title: Security Requirements
description: The vulnerability policy in brief, and the security-relevant guarantees the design makes.
status: stable
sources:
  - id: security-md
    resource: /SECURITY.md
    title: Security policy
---

The full policy is [SECURITY.md](/SECURITY.md).[^security-md] In brief:
vulnerabilities are reported privately via GitHub's private vulnerability
reporting, never as public issues; fixes target the latest release and `main`
only (pre-1.0, no backports).

# Guarantees the design makes

- **Pinned installs are checksummed.** The code-index bundle installs only
  the registry version this binary carries, verified against its compiled-in
  sha256 — superdev refuses any other version rather than fetch unverified
  content ([architecture](architecture.md)).
- **No secrets in files.** The embeddings API key comes from
  `OPENAI_API_KEY` and is never read from or written to the manifest
  ([configuration](configuration.md)). Publishing uses npm trusted
  publishing (OIDC) — no long-lived npm token exists; only crates.io still
  holds a token, as a CI secret.
- **Destructive writes are recoverable.** Every file superdev overwrites or
  removes is backed up under `.superdev/cache/backup/<timestamp>/` first,
  and a failed apply unwinds ([configuration](configuration.md)).
- **The MCP surface is read-only.** `superdev mcp aokf` exposes four
  read-only tools over stdio; nothing writes through it
  ([api-contracts](api-contracts.md)).
- **Local by default.** The CLI takes no network input at runtime; the
  network is touched only for pinned tool installs and the one-time
  embedding model download (or the explicit embeddings-API opt-in). The
  accepted non-PIE musl binaries rest on exactly this — see
  [constraints-non-goals](constraints-non-goals.md).

[^security-md]: Security policy
