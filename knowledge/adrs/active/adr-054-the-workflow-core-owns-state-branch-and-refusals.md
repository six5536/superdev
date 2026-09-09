---
type: Decision
id: adr-054-the-workflow-core-owns-state-branch-and-refusals
title: The workflow core owns state, branch, and refusals
description: The deterministic workflow core owns durable phase, branch identity, and a Git refusal list; the canonical plan carries progress that nothing parses, and isolated LLM roles own review, correction order, and evidence.
lifecycle: active
links:
  - rel: supersedes
    to: adr-052-the-workflow-is-scope-build-accept-under-a-durable-core
    note: The workflow, its phases, and its safety rules stand; the core stops parsing plan prose, accounting for work blocks, and scheduling corrections.
---

# ADR-054: The workflow core owns state, branch, and refusals

- Date: 2026-09-09
- Deciders: superdev maintainers

## Context

[ADR-052][sokf:adr-052-the-workflow-is-scope-build-accept-under-a-durable-core] made the canonical plan carry durable progress and evidence, and made the Rust core read both. The core therefore parses prose that an isolated role wrote: 846 lines across `records.rs` and `evidence.rs` prefix-match strings such as `"Final verification: "` inside markdown.

The workflow now spans about 7,900 lines: 5,400 in the Rust workflow modules and 2,485 in the Pi extension. Every workflow failure observed during several days of use originated in that harness rather than in an isolated role:

- The parent refused a valid SCOPE correction because a resolved finding accompanied status `complete`.
- A reviewer completed its analysis, submitted nothing the parent accepted, and the failure reported only a submission count.
- An `ask` request without a finding ID surfaced as a phase failure offering recovery operations.
- The correction scheduler corrected a mechanical finding before the two substantive findings it declared as dependencies.

The isolated roles returned complete, actionable findings on every one of those runs. The core distrusts the model's competence, so it re-derives, re-validates, and re-schedules decisions the model already made. A core that distrusts the model's authority instead constrains what the model may do, which is a smaller and decidable job.

## Decision

We will reduce the deterministic workflow core to durable state, branch identity, and a refusal list, and let isolated roles own every judgement.

The core keeps the Git refusal list, branch identity, identity reservation under the repository lock, the `start`, `resume`, `cancel`, and `status` operations, the `Phase` value with its legal transitions, human-only abandonment, and bounded child output. Gate evidence records human authority alone: scope approval, acceptance approval, and abandonment approval. It no longer records whether a requirements review was clean, because that judgement belongs to the reviewer.

The canonical plan remains the durable record of progress and evidence, and nothing parses it. The core tracks phase only, and isolated roles read and write plan progress with ordinary tools. The core stops accounting for work blocks, attempts, corrections, scope baselines, and scope checkpoints, and stops performing candidate attestation, default-branch synchronization, and integration bookkeeping. The extension stops scheduling corrections, arbitrating review results, and policing terminal submission; each role orders its own corrections and honours the finding dependencies it declared.

One SCOPE → BUILD → ACCEPT workflow remains. SCOPE alone obtains human approval for behavioural and architectural decisions. BUILD owns its work blocks, verification, and isolated review. ACCEPT follows the project-wide human-acceptance setting, closes accepted records, and releases checkout ownership; the accepted work branch stays checked out and humans merge separately. Cancellation pauses, and only a human abandons a plan. Interface declarations remain source-owned: SCOPE authors pending prose promises, and BUILD changes source and materializes contract Definition sections. The issue and its independently numbered initial plan are reserved on the discovered default branch under a repository lock before the shared checkout switches to the work branch, which is persisted for recovery. Several unfinished work branches may exist, but one workflow executes per checkout, and parent and children write sequentially. `/skill:file` remains an independent capture skill with no dedicated service, tool, or child role. Contracts continue to carry keyed EARS promises with nested criteria, and every plan implements one issue.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Minimal core of state, branch, and refusals | Removes the layer where every observed failure occurred; each decision sits with the party competent to make it | One large migration that shrinks a public CLI contract |
| Keep the ADR-052 parsing core | No migration | Prose parsing, block accounting, and correction scheduling keep producing the observed failures |
| Patch the correction scheduler alone | Smallest possible change | Leaves the cycle budgets, fingerprint reuse, and mechanical-first shortcut that produced the dependency-order defect |
| Move durable state into the Pi extension | Fewer moving parts | A lost session loses the workflow, and TypeScript becomes a second authority over project state |

## Consequences

- Positive: about 6,000 lines leave the harness, and the rules that remain are decidable without reading prose.
- Positive: a review returns one finding set to the user in the order its author declared, rather than an order the harness imposes.
- Positive: every remaining core rule is testable against a real Git repository.
- Negative: plan progress becomes advisory, so a role that records it poorly is no longer corrected by the core.
- Negative: `contract-002-cli-superdev` loses workflow subcommands, which changes a public surface in one release.
- Negative: issue-077 stays paused until this work lands.
- Follow-ups: the raw intent that the main session always stays on the default branch, and that branches carry a feature, bug, or chore category, remains unimplemented and needs its own decision.
- Follow-ups: `contract-002-cli-superdev`, `contract-004-config-superdev`, `configuration.md`, and the user documentation need regeneration against the reduced surface.

<!-- sokf:links -->
[sokf:adr-052-the-workflow-is-scope-build-accept-under-a-durable-core]: /knowledge/adrs/deprecated/adr-052-the-workflow-is-scope-build-accept-under-a-durable-core.md
