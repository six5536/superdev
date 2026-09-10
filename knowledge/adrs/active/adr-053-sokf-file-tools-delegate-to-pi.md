---
type: Decision
id: adr-053-sokf-file-tools-delegate-to-pi
title: SOKF file tools delegate semantics and presentation to Pi
description: SOKF resolves and protects canonical knowledge targets while Pi 0.85.1 remains authoritative for file-tool semantics, results, and rendering.
lifecycle: active
status: draft
---

# ADR-053: SOKF file tools delegate semantics and presentation to Pi

- Date: 2026-09-06
- Deciders: superdev maintainers

## Context

The Pi extension currently routes SOKF identities through separate read and mutation result dialects. The adapter reconstructs edit details, exposes mutation envelopes, and couples semantic retrieval to exact file reads. Those choices diverge from Pi 0.85.1 and make equivalent physical and virtual targets behave differently.

SOKF still owns identity resolution, repository containment, generated ownership, agent-safe policy, stable identity, verification preservation, automatic repair, refiling, and validation. Applied mutations can leave actionable validation findings, and Pi sessions need bounded correction follow-ups that survive active-branch lifecycle events.

## Decision

We will resolve SOKF identities before execution, then delegate file semantics and presentation to Pi 0.85.1 factories and exported helpers. A narrow adapter is permitted only where package exports expose no preparation or presentation seam, and paired tests will prove that adapter byte-for-byte against Pi's operation boundaries.

We will follow Pi's working-directory scope in normal checkouts and linked Git worktrees. The canonical active checkout root, discovered from either a `.git` directory or worktree pointer file, is the repository boundary; Git's shared common directory and main checkout do not replace it. Discovery keeps today's `.superdev/config.toml` acceptance as a ranked fallback: a `.git` marker anywhere on the upward walk always wins, and the configuration marker selects the root only when the walk finds no `.git` marker at all, so a managed non-Git repository keeps SOKF routing while one precedence rule makes the chosen root canonical. Every SOKF operation, MCP client and index, mutation queue target, validation run, and persisted follow-up sequence remains isolated by that active root. Another checkout is outside its containment boundary.

We will queue virtual and physical aliases by canonical physical target. Edits will persist through an agent-safe compare-and-swap source transport that re-resolves the logical ingress and rejects target drift. A missing physical destination will canonicalize through its nearest existing ancestor. Repository-contained symlink targets are accepted, including targets outside `knowledge/`; repository escapes are rejected.

Symlink handling stops at containment. Resolution accepts a repository-contained target and refuses one that escapes. Whether the file behind a contained target is SOKF knowledge stays as it is today, decided by the physical `knowledge/` tree. Issue-081 proposed a logical-ingress membership model and was declined.

An authoritative `applied: true` response is the mutation outcome boundary. The adapter will retain the requested bytes and every successfully persisted repair or refiling change, shield dispatched persistence from late cancellation, hold the queue through mutation-state capture, and return Pi's unchanged success result. Failures without an authoritative applied response remain tool errors. An indeterminate local MCP failure requires target inspection and `superdev validate`, not durable outcome reconciliation.

Validation diagnostics will remain outside file-tool content. Follow-up state will be session memory keyed by canonical repository root: one pending flag and one flag recording whether a report has already triggered a correction turn. Nothing persists, so no lifecycle event needs a recovery rule and no append can fail.

The adapter will re-check its findings against the working tree immediately before sending, and report only those the tree still carries. Validation runs at `turn_end` and delivery happens afterward, so a repair, merge, or later mutation can resolve a finding in between. Reporting a resolved finding costs the agent a turn and teaches it to distrust the channel.

The first report for a pending sequence triggers one correction turn. A later report is visible and non-triggering, leaving the agent to decide. The adapter will not count attempts or refuse to report, because an agent that reads a repeated report can check the tree itself.

The unreleased MCP surface will replace `sokf_read` with `sokf_resolve_source` and `sokf_retrieve`. The resolver carries no semantic content: a successful result pairs its structured content with exactly one text item holding that content's pretty-printed JSON, as `mutation_result()` already does, so the surface stays idiomatic for any MCP client while the Pi adapter reads the structured half and ignores the text item. The server's initialization instructions name only served tools, so no client is told about `sokf_read` or a file read of the `sokf:` overview address. The retrieval operation retains semantic overview, rendered-concept, section, and line-window behavior. Every replacement request and structured result will use the exact closed schema in `contract-003-api-sokf P_routed-schemas`; the adapter will reject missing and additional fields. Resolver output includes line-bounded generated regions and their authoritative source. Mutation output includes literal applied state, unique lexical changed paths, actionable findings, and an explicit diagnostics-truncation flag without patches.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Delegate to Pi factories with narrow byte-proven adapters and active-worktree isolation | Exact parity, one file-tool experience, and checkout-local behavior | Couples parity tests to pinned Pi 0.85.1 and requires multi-worktree fixtures |
| Scope all worktrees by Git's shared common directory | One process-level repository identity | Can read, mutate, or validate the wrong checkout and merge unrelated follow-up state |
| Require a Git checkout for SOKF routing | One discovery marker | Silently removes routing from managed non-Git repositories that work today |
| Return resolver structured content with an empty content array | Smallest payload | Shapes a general MCP surface around the single known Pi consumer |
| Reimplement Pi file algorithms in the extension | Full local control | Creates an approximate fork that drifts |
| Keep rendered virtual reads and mutation envelopes | Smallest source change | Preserves the current contract divergence |
| Roll back after repair or validation findings | Leaves only valid knowledge | Loses acknowledged work and forces expensive rewrites |
| Persist mutation identities and reconcile outcomes | Resolves indeterminate transport failures | Adds durable protocol and lifecycle complexity outside this issue |
| Preserve `sokf_read` as a compatibility alias | Avoids an MCP rename | Keeps mixed exact-source and semantic semantics in an unreleased API |
| Persist the follow-up sequence and cap automatic corrections | Survives reload, fork, and compaction | Adds a snapshot union, a degraded mode, and recovery rules to defend a two-count integer |
| Report findings without re-checking the tree | One validation run per turn | Sends findings the tree has already resolved, as three observed follow-ups did |
| Give contained symlinks logical-ingress SOKF membership | Aliased concepts become knowledge | Adds ingress selection, cycle detection, and duplicate resolution for an unreported gap |

## Consequences

- Positive: physical paths and equivalent SOKF identities share Pi's tested file semantics, results, metadata, and rendering.
- Positive: normal checkouts and linked worktrees keep independent SOKF files, MCP activity, mutations, validation, and follow-up state, and a managed non-Git repository keeps the routing it has today.
- Positive: the resolver result reads correctly for a text-only MCP client and for a structured-content client, with no second dialect to learn.
- Positive: SOKF safety, repair, refiling, and validation remain authoritative without replacing successful file-tool results.
- Positive: validation reports describe the tree the agent is about to read, so a resolved finding is never sent.
- Positive: session-scoped follow-up state removes every persistence, recovery, and degraded-mode rule from the reporting path.
- Negative: parity evidence pins Pi 0.85.1 and requires synchronization when Pi changes its built-in contracts.
- Negative: an indeterminate local MCP failure requires manual filesystem inspection and validation.
- Negative: schema evolution requires coordinated Rust and TypeScript changes because routed MCP objects reject additional fields.
- Negative: a reload or fork clears the sent-report flag, so an unresolved finding may trigger one further correction turn.
- Negative: the freshness re-check reads the reported paths again after validation, adding one filesystem pass per report.
- Follow-ups: changes to semantic ranking, unrelated MCP lifecycle behavior, durable outcome reconciliation, and packaging design require separate issues.
