---
type: Idea
id: idea-012-sokf-mutations-survive-validation-failures
title: SOKF mutations survive validation failures
description: Apply SOKF write and edit mutations before validation, then report validation findings without discarding the changed file or replacing Pi's file-tool contract.
status: draft
---

# Idea: SOKF mutations survive validation failures

Apply an SOKF-routed `write` or `edit` even when Superdev validation finds an
error. Return the validation reason after preserving the changed file, ideally
through Pi's existing file-tool result or diagnostic mechanisms rather than a
custom result contract.

## Motivation

Rejecting the mutation forces the language model to reconstruct and resend the
entire write or edit after addressing the validation finding. Preserving the
mutation keeps the attempted content in the working tree, where the model can
make a targeted correction without spending context on another full rewrite.

## Sketch

Separate mutation application from validation acceptance. Complete the normal
Pi-compatible file mutation first, run Superdev repair and validation against
the resulting file, and retain the resulting bytes when validation fails.
Surface concise, actionable findings through the nearest existing Pi API
channel that can represent a successful mutation with follow-up diagnostics.

## Open questions

- Which Pi result, warning, or diagnostic channel can report validation findings
  while preserving built-in `write` and `edit` compatibility?
- Should repair output remain applied when later validation still fails?
- How should cancellation behave after the mutation succeeds but before
  validation finishes?
