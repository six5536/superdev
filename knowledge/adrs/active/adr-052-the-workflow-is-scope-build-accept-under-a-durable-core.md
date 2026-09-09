---
type: Decision
id: adr-052-the-workflow-is-scope-build-accept-under-a-durable-core
title: The workflow is SCOPE, BUILD, ACCEPT under a durable core
description: Superdev's one workflow is SCOPE → BUILD → ACCEPT; plans carry durable progress, the Rust core enforces transitions, Pi orchestrates isolated roles, and project-declared documentation is build evidence.
lifecycle: active
links:
  - rel: supersedes
    to: adr-018-loop-in-the-skill-enforcement-in-the-hook
    note: Pi orchestration and the Rust workflow service replace the Claude skill and Stop-hook loop.
  - rel: supersedes
    to: adr-019-run-state-is-a-session-owned-file-behind-cli-verbs
    note: The plan owns durable progress and cache state owns transient Pi-session exclusion.
  - rel: supersedes
    to: adr-020-a-blocked-run-ends
    note: Discoveries accumulate on the issue and return the same open plan to SCOPE.
  - rel: supersedes
    to: adr-021-nothing-unattended-reaches-the-default-branch
    note: ACCEPT closes records after configured gates; humans merge separately.
  - rel: supersedes
    to: adr-028-the-contract-design-go-ahead-is-an-explicit-interaction
    note: SCOPE owns one complete explicit approval boundary.
  - rel: supersedes
    to: adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source
    note: SCOPE authors pending prose promises but BUILD changes source declarations and materializes Definition sections.
  - rel: supersedes
    to: adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept
    note: The contract-key decision remains; FILE is a utility and the workflow is only SCOPE, BUILD, ACCEPT.
---

# ADR-052: The workflow is SCOPE, BUILD, ACCEPT under a durable core

- Date: 2026-09-06
- Deciders: superdev maintainers

## Context

The knowledge-carried workflow assigns its loop to both BUILD and execute-plan, binds continuation to Claude Code Stop hooks, leaves manual plan cases without an executor, closes plans before acceptance, and treats user documentation as a generic final reminder. The superseded decisions are:

- [ADR-018][sokf:adr-018-loop-in-the-skill-enforcement-in-the-hook]
- [ADR-019][sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs]
- [ADR-020][sokf:adr-020-a-blocked-run-ends]
- [ADR-021][sokf:adr-021-nothing-unattended-reaches-the-default-branch]
- [ADR-028][sokf:adr-028-the-contract-design-go-ahead-is-an-explicit-interaction]
- [ADR-044][sokf:adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source]
- [ADR-050][sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]

A long-lived workflow also needs to resume after a Pi session or child agent ends without making TypeScript a second authority over project state.

## Decision

We will use one SCOPE → BUILD → ACCEPT workflow: a canonical plan carries durable phase, progress and evidence; the Rust core enforces legal transitions and Git safety; a Pi extension owns interaction and isolated agents; `/file` remains an independent capture utility; and a project documentation map makes every applicable handwritten, generated or site surface an executable BUILD obligation.

SCOPE alone obtains human approval for behavioural and architectural decisions. BUILD owns every work block, final verification and an isolated review. ACCEPT follows the project-wide human-acceptance setting, closes accepted records, and releases checkout ownership. The accepted work branch remains checked out. Humans merge separately; acceptance never merges automatically. Cancellation pauses; only a human may abandon a plan. Interface declarations remain source-owned: SCOPE may author pending prose promises, while BUILD changes source and materializes contract Definition sections.

Reserve the issue and independently numbered initial plan on the discovered default branch under a repository lock. Then switch the shared parent-and-child checkout to the work branch. Persist the default branch in plan evidence for recovery. Multiple unfinished work branches may exist, but only one workflow executes per checkout. Parent and modifying children write sequentially. Require a clean explicit return to the default branch before filing unrelated work.

Contracts continue to carry keyed EARS promises and nested criteria; plans cite those keys with executable evidence, and a pending marker applies only to unbuilt prose promises. Issues remain one `bug | feature | chore` shape, and plans remain one shape, but every plan implements one issue and no manual-case escape remains.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Rust authority with Pi orchestration and plan-carried progress | Resumable, enforceable, one supported flow | More core and adapter work |
| Pi extension owns all state and prompts | Fastest initial implementation | Session loss and another source of truth |
| Repair the Claude skills and Stop hooks | Smaller migration | Retains duplicate loops and unsupported harness coupling |
| Keep documentation as a final reminder | No new project inventory | User-facing surfaces continue to drift or be forgotten |

## Consequences

- Positive: one phase owns each loop, mutation boundary and approval.
- Positive: project-specific documentation is planned and verified without hard-coding README or one site generator.
- Positive: a restarted Pi session reconstructs work from the plan and Git.
- Negative: issues, plans, packs, contracts and active workflow documentation require one strict migration.
- Negative: Claude Code workflow support is archived until a future implementation derives a shared adapter from the proven Pi flow.
- Follow-ups: release tags and customizable release/publication orchestration need a separate issue and plan.

<!-- sokf:links -->
[sokf:adr-018-loop-in-the-skill-enforcement-in-the-hook]: /knowledge/adrs/deprecated/adr-018-loop-in-the-skill-enforcement-in-the-hook.md
[sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs]: /knowledge/adrs/deprecated/adr-019-run-state-is-a-session-owned-file-behind-cli-verbs.md
[sokf:adr-020-a-blocked-run-ends]: /knowledge/adrs/deprecated/adr-020-a-blocked-run-ends.md
[sokf:adr-021-nothing-unattended-reaches-the-default-branch]: /knowledge/adrs/deprecated/adr-021-nothing-unattended-reaches-the-default-branch.md
[sokf:adr-028-the-contract-design-go-ahead-is-an-explicit-interaction]: /knowledge/adrs/deprecated/adr-028-the-contract-design-go-ahead-is-an-explicit-interaction.md
[sokf:adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source]: /knowledge/adrs/deprecated/adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source.md
[sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]: /knowledge/adrs/deprecated/adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept.md
