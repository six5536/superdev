---
type: ConstraintsNonGoals
id: constraints-non-goals
title: Known Constraints & Non-Goals
description: What superdev deliberately does not do, and the accepted limitations of the inherited machinery.
status: stable
---

# Non-goals

- **Generalisation, yet.** superdev is opinionated for one stack — Claude
  Code, mise, SOKF. Other agent harnesses, tool managers and knowledge
  formats stay out of scope until the one stack is proven; the
  capability/provider split ([architecture][sokf:architecture]) is where
  alternatives would slot in later.
- **Managing toolchains.** superdev pins tools in `.mise.toml` and
  delegates installation to mise; it never downloads or manages a
  toolchain itself. (Rejecting a node pin for codegraph's sake is this
  rule applied — see [backlog][sokf:backlog].)
- **A service component.** superdev is a local CLI: no daemon, no hosted
  service, no telemetry. The MCP servers are local stdio processes the
  agent harness spawns per session.
- **Testing a project's behaviour.** superdev binds a contract's
  definition — the include is materialised from source and a stale one
  fails the run — and stops there. Whether the code does what the
  Behaviour section promises is the project's to test, in its own
  language and test runner, which superdev cannot know
  ([ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]
  draws the line). superdev supplies no harness, runs no project
  command and parses no source: it moves bytes between markers.

# Constraints

Inherited machinery, and what it fixes.

- **Pre-1.0**: minor versions may carry breaking changes; `superdev-core`'s
  API is not stable.
- **Cross-registry publishing is not atomic**; the release pipeline is
  ordered, dry-run-gated and recoverable instead — see
  [release-procedure][sokf:release-procedure].
- zigbuild's musl output is non-PIE; accepted for a local CLI with no network
  input.
- **No auto-download or build-from-source fallback** in the npm launcher on
  unsupported platforms; the message points at `cargo install superdev`.
- **An unmarked element is not in the contract.** A definition is what
  its source regions carry, and a public item outside every
  `sokf:begin`/`sokf:end` pair is promised nowhere. Whether the marked
  regions are the whole surface is not decidable from the tree — it
  needs the language's own notion of public — so it falls on the far
  side of the line
  [ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]
  drew, and the judgement step at integration asks it at the
  reliability an LLM judgement carries rather than a check's.

<!-- sokf:links -->
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:architecture]: /knowledge/architecture.md
[sokf:backlog]: /knowledge/backlog.md
[sokf:release-procedure]: /knowledge/release-procedure.md
