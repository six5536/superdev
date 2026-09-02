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
- **Binding a project's contracts to its code.** Every contract-kind
  schema obliges the project to bind its implemented interface to the
  contract's declared surface, and superdev supplies none of it: no
  harness, no generator, no drift test, and no gate that one exists.
  The mechanism depends on the project's language, framework and test
  runner, which superdev cannot know
  ([ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation]
  leaves it to the project deliberately). The obligation ships; the
  means does not.

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
- **An unbound contract passes every gate.** Whether a contract is bound
  is not decidable from the knowledge tree — the binding lives in the
  project's test suite, in a language superdev does not read — so it
  falls on the far side of the line
  [ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]
  drew, and no validator check will ever catch it. An agent reading the
  contract beside the tests can detect it, at the reliability an LLM
  judgement carries rather than a check's; no skill asks it to today.
  Until one does, a project that never writes the binding has contracts
  that document rather than bind, and only a reader notices.

<!-- sokf:links -->
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:architecture]: /knowledge/architecture.md
[sokf:backlog]: /knowledge/backlog.md
[sokf:release-procedure]: /knowledge/release-procedure.md
