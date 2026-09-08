---
type: Idea
id: idea-012-sokf-mutations-survive-validation-failures
title: SOKF mutations survive validation failures
description: Apply SOKF write and edit mutations before validation, then report validation findings without discarding the changed file or replacing Pi's file-tool contract.
status: draft
---

# Idea: SOKF mutations survive validation failures

Apply an SOKF-routed `write` or `edit` even when Superdev validation finds an
error. Preserve Pi's successful file-tool result after persistence, and report
the validation reason through a separate visible follow-up.

## Motivation

Rejecting the mutation forces the language model to reconstruct and resend the
entire write or edit after addressing the validation finding. Preserving the
mutation keeps the attempted content in the working tree, where the model can
make a targeted correction without spending context on another full rewrite.

## Sketch

Separate mutation application from validation acceptance. Complete the normal
Pi-compatible file mutation first, run Superdev repair and validation against
the resulting file, and retain the resulting bytes when validation fails.
Surface concise, actionable findings outside model-visible file-tool content.

Cancellation before persistence dispatch leaves the target unchanged. After
persistence dispatch, stop forwarding cancellation to the mutation and keep
the canonical target queued until the mutation settles. If the mutation
applies, retain the requested bytes and every successfully persisted repair or
refiling change. Return Pi's normal success result even when cancellation or a
finding arrives after dispatch. If persistence fails without applying, report
the persistence failure and schedule no final validation.

At the next `turn_end`, validate all applied mutations once. A valid result
clears the sequence. A remaining finding produces a bounded visible Pi custom
message that triggers a targeted correction turn. Trigger at most two such
turns. After the second correction turn, report a remaining finding without
triggering another turn. Mutations in an unresolved sequence do not reset the
cap. A mutation after the cap schedules one non-triggering manual validation.
A valid result resets the cap, paths, and findings before a later mutation
starts a new sequence. Persist the sequence in non-context session entries so
reload, resume, fork, compaction, and tree navigation do not reset it.

## Open questions

- None. [Issue 077][sokf:issue-077-sokf-file-tool-parity] settles the result
  channel, repair survival, cancellation boundary, follow-up cap, and reset
  rules.

<!-- sokf:links -->
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/open/issue-077-sokf-file-tool-parity.md
