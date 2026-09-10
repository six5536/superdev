---
type: Issue
id: issue-083-sokf-validation-follow-ups
title: SOKF validation findings ride inside file-tool results and reset their own retry cap
description: Validation feedback after a knowledge mutation is delivered as file-tool content and counted in a variable that every reload, resume, fork, or compaction clears, so an agent reads mutation internals and can be asked to correct the same failure without bound.
kind: feature
lifecycle: open
links:
  - rel: references
    to: issue-082-sokf-mutation-parity
    note: The mutation outcome boundary lands first; this issue consumes its acknowledged applied state and bounded findings.
  - rel: references
    to: idea-012-sokf-mutations-survive-validation-failures
    note: Originating idea; this issue settles the follow-up state machine and its reset rules.
---

# Feature: SOKF validation follow-ups are bounded, separate, and durable

## Summary

After a knowledge mutation, an agent should read Pi's ordinary tool result and,
separately, a concise list of what still fails validation. It instead reads
mutation internals inside the tool result, and the correction allowance lives in
a variable that any session lifecycle event resets, so the same failure can
trigger unbounded correction turns.

## Context

`.pi/extensions/sokf.ts` holds `validationFollowUps` as an in-process counter and
resets it on every valid result. Nothing persists it, so reload, resume, fork,
compaction, and tree navigation all restore a session whose allowance has
silently returned to full. The `turn_end` handler validates and then sends the
report, with no guard against two handlers validating the same pending state.

The same file returns MCP mutation JSON as model-visible tool content, so
validation findings, repair reports, and whole-file patches reach the model
through the file-tool result rather than through a separate message.

The [SOKF mutations survive validation failures][sokf:idea-012-sokf-mutations-survive-validation-failures]
idea originated the requirement that findings arrive without discarding the
changed file. This issue settles how those findings reach the agent.

Pi supplies the delivery mechanism. `pi.sendMessage()` accepts a custom message
with `{ deliverAs: "followUp", triggerTurn: boolean }`, and `pi.appendEntry()`
persists a non-context entry that `session_start` and `session_tree` can
reconstruct on the active branch.

## Behaviour

Validation feedback is a bounded, separate, durable sequence.

- An acknowledged mutation records changed paths and pending validation state
  before the file-tool result resolves. The result itself stays exactly Pi's.
- The extension persists each repository's sequence as an exact closed, versioned,
  discriminated snapshot. A clean snapshot carries only its version, canonical
  absolute repository root, and `clean` state. Each unresolved snapshot carries
  those fields plus its state, unique lexical changed paths, bounded findings, and
  an explicit truncation flag; only `pending-auto` also carries a follow-up count
  from 0 through 2.
- At `turn_end`, the extension validates one coalesced unresolved sequence exactly
  once, even when the preceding correction turn mutated nothing. A guard prevents
  two handlers validating the same state concurrently.
- A valid result enters `clean` and clears the count, changed paths, and findings.
- An invalid, unknown, or execution-failure result always yields an actionable
  finding. Below the cap it sends one visible triggering follow-up and increments
  the count. At the cap it enters `manual` and sends non-triggering guidance.
- A mutation in `manual` enters `pending-manual` and schedules one validation
  without restoring automatic turns. Further mutations coalesce.
- Only a valid final-validation result resets the allowance. Additional mutations,
  failed validations, cancellation, reload, resume, fork, compaction, and tree
  navigation never reset it. Reconstruction restores the latest active-branch
  snapshot and never resends a follow-up already recorded as sent.
- A failed snapshot append enters an explicit repository-scoped in-memory degraded
  mode that carries the intended durable state and whether validation is due,
  suppresses triggered follow-ups, and emits separate bounded non-triggering
  guidance. An append failure after `applied: true` never changes the file-tool
  success. An append failure before a triggered follow-up suppresses that follow-up
  without incrementing its count. Failures while entering `manual` or `clean`
  preserve those intended in-memory states without claiming a durable transition.
- Every visible follow-up and every non-context entry carries only concise paths,
  locations when available, messages, and corrective actions. Each stops at 200
  lines or 8 KiB, omits patches and mutation envelopes, and directs the agent to
  `superdev validate` when truncated.
- Validation state, its execution, and its snapshots stay inside the active
  checkout, including a linked Git worktree.
- `read`, `edit`, and `write` retain Pi's built-in prompt snippets, guidelines,
  descriptions, schemas, and renderers, adding only three flat SOKF routing rules.
  `sokf_search` and `sokf_graph` own knowledge discovery wording.

## Scope

Post-persistence validation delivery and file-tool prompt metadata.

- In: the persisted validation-state schema and state machine, degraded mode,
  session recovery, bounded diagnostics, and the `turn_end` guard.
- In: file-tool prompt metadata, the authoring skill and its pack copy, and the
  public documentation that describes the routed file tools.
- In: the follow-up promises on `contract-012-api-sokf-pi-file-tools` and the
  state-machine decision in `adr-053-sokf-file-tools-delegate-to-pi`.
- Out: the mutation outcome boundary itself, owned by
  [issue-082][sokf:issue-082-sokf-mutation-parity].
- Out: source resolution, read parity, and the MCP retrieval split, owned by
  [issue-077][sokf:issue-077-sokf-file-tool-parity].
- Out: validation content inside file-tool results, rollback after acknowledged
  persistence, and a new extension packaging model.

## Comments

Separated from [issue-077][sokf:issue-077-sokf-file-tool-parity] because its
requirements review exceeded the isolated-role timeout. The follow-up state
machine depends on the acknowledged applied state that issue-082 establishes.

<!-- sokf:links -->
[sokf:idea-012-sokf-mutations-survive-validation-failures]: /knowledge/ideas/idea-012-sokf-mutations-survive-validation-failures.md
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/open/issue-077-sokf-file-tool-parity.md
[sokf:issue-082-sokf-mutation-parity]: /knowledge/issues/open/issue-082-sokf-mutation-parity.md
