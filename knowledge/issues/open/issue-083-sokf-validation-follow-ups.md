---
type: Issue
id: issue-083-sokf-validation-follow-ups
title: SOKF validation findings ride inside file-tool results and can report a repository state that no longer exists
description: Validation feedback after a knowledge mutation is delivered as file-tool content and captured before the message is sent, so an agent reads mutation internals and can be asked to correct findings that the working tree has already resolved.
kind: feature
lifecycle: open
links:
  - rel: references
    to: issue-082-sokf-mutation-parity
    note: The mutation outcome boundary lands first; this issue consumes its acknowledged applied state and bounded findings.
  - rel: references
    to: idea-012-sokf-mutations-survive-validation-failures
    note: Originating idea; this issue settles how findings reach the agent.
---

# Feature: SOKF validation follow-ups are separate, fresh, and bounded

## Summary

After a knowledge mutation, an agent should read Pi's ordinary tool result and,
separately, a concise list of what still fails validation. It instead reads
mutation internals inside the tool result, and the list it receives can describe
findings that the working tree no longer carries.

## Context

Three consecutive follow-ups in one observed session reported findings against a
repository that returned `PASS (0 error(s), 0 warning(s))`. The sequence ran to
its terminal message, `SOKF validation still fails after 2 automatic repair
turns`, and the agent spent three turns confirming that nothing was wrong.

The cause is ordering. The `turn_end` handler in `.pi/extensions/sokf.ts` runs
`superdev validate`, formats the report, and sends it. Between the run and the
delivery, the tree can change — an automatic repair, a merge, or a mutation later
in the same turn. Nothing re-checks the findings against the tree the agent will
actually see.

The same file returns MCP mutation JSON as model-visible tool content, so
validation findings, repair reports, and whole-file patches reach the model
through the file-tool result rather than through a separate message.

The [SOKF mutations survive validation failures][sokf:idea-012-sokf-mutations-survive-validation-failures]
idea originated the requirement that findings arrive without discarding the
changed file. This issue settles how those findings reach the agent.

Pi supplies the delivery mechanism. `pi.sendMessage()` accepts a custom message
with `{ deliverAs: "followUp", triggerTurn: boolean }`.

## Behaviour

Validation feedback is separate from the tool result, fresh at the moment it is
sent, and offered rather than enforced.

- An acknowledged mutation marks validation pending. The file-tool result stays
  exactly Pi's, carrying no findings, mutation envelope, or repair report.
- At `turn_end` with validation pending, the extension runs final validation once.
- Before sending any message, the extension re-checks its findings against the
  current working tree and reports only those the tree still carries. A report
  that survives no finding sends no message.
- A valid result clears the pending flag and sends nothing.
- The first report for a pending sequence sends one visible triggering follow-up,
  so the agent gets one prompted chance to correct.
- A later report sends one visible non-triggering message. The agent decides
  whether to act. The extension does not escalate, count, or refuse.
- Follow-up state is session memory keyed by canonical repository root. Nothing
  persists, so a new session starts with none and no lifecycle event needs a
  recovery rule.
- Every visible message carries only concise paths, locations when available,
  messages, and corrective actions. Each stops at 200 lines or 8 KiB, omits
  patches and mutation envelopes, and directs the agent to `superdev validate`
  when truncated.
- Validation state and its execution stay inside the active checkout, including a
  linked Git worktree.
- `read`, `edit`, and `write` retain Pi's built-in prompt snippets, guidelines,
  descriptions, schemas, and renderers, adding only three flat SOKF routing rules.
  `sokf_search` and `sokf_graph` own knowledge discovery wording.

## Scope

Post-persistence validation delivery and file-tool prompt metadata.

- In: the freshness re-check, the session-memory pending flag, triggering the
  first report only, and bounded message content.
- In: file-tool prompt metadata, the authoring skill and its pack copy, and the
  public documentation that describes the routed file tools.
- In: the follow-up promises on `contract-012-api-sokf-pi-file-tools` and the
  delivery decision in `adr-053-sokf-file-tools-delegate-to-pi`.
- Out: persisted follow-up state, a correction cap, and any recovery rule for
  reload, resume, fork, compaction, or tree navigation. An agent that reads a
  stale or repeated report can check the tree itself, so the machinery that would
  defend a counter across those events costs more than the counter is worth.
- Out: the mutation outcome boundary itself, owned by
  [issue-082][sokf:issue-082-sokf-mutation-parity].
- Out: source resolution, read parity, and the MCP retrieval split, owned by
  [issue-077][sokf:issue-077-sokf-file-tool-parity].
- Out: validation content inside file-tool results, rollback after acknowledged
  persistence, and a new extension packaging model.

## Comments

Separated from [issue-077][sokf:issue-077-sokf-file-tool-parity] because its
requirements review exceeded the isolated-role timeout.

Originally scoped with a persisted `ValidationStateSnapshot` union, a
`clean` → `pending-auto(0..2)` → `manual` state machine, an in-memory degraded
mode, and active-branch recovery: 34 deferral markers. That design added
persistence to the reporting path without addressing the staleness that the
observed session demonstrated, and its state existed to defend a two-count
integer. Narrowed to the freshness re-check, a session-scoped flag, and one
triggered report.

<!-- sokf:links -->
[sokf:idea-012-sokf-mutations-survive-validation-failures]: /knowledge/ideas/idea-012-sokf-mutations-survive-validation-failures.md
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/open/issue-077-sokf-file-tool-parity.md
[sokf:issue-082-sokf-mutation-parity]: /knowledge/issues/open/issue-082-sokf-mutation-parity.md
