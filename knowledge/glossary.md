---
type: Glossary
id: glossary
title: Domain Glossary
description: The terms the blueprint engine uses — blueprint, capability, provider, component, owned file, scaffold.
status: stable
---

- **Blueprint** — superdev's opinion of a managed repo, compiled into the
  binary: the component set plus the registry of default versions tested
  together. The binary's version is the blueprint version.
- **Capability** — a functionality slot in a managed repo: `knowledge`,
  `code-index`, `workflows`, `frontend`, `skills`. Capability names are what
  users type; see [architectural-rules](architectural-rules.md).
- **Provider** — the tool that fills a capability, e.g. `codegraph` for
  `code-index`. Swappable without changing the user-facing surface.
- **Component** — the code implementing one provider. It observes the repo,
  compares against the manifest, and returns actions; it never applies them.
- **Owned file** — a file superdev writes and keeps current, hashed into
  `lock.toml`. `sync` rewrites it, backing up and reporting any user edit.
  The embedded AOKF spec and validator are owned.
- **Scaffold** — a file superdev writes once and never touches again, such as
  `AGENTS.md`. It is the user's from the moment it exists, so it cannot drift.

The files these terms describe are in [configuration](configuration.md); the
layering is in [architecture](architecture.md).
