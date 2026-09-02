---
type: FeatureRequest
id: issue-027-feature-request-the-unattended-run-is-undocumented-for-users
title: The README says nothing about the unattended run
description: The run verbs, the Stop hook and /execute-feature-plan exist on the CLI, in the man page and in the workflow skills, but the README never mentions them, so a user cannot discover unattended delivery.
lifecycle: open
links:
  - rel: references
    to: issue-024-feature-request-the-workflow-cannot-run-unattended
    note: The feature the documentation is missing for; found by its acceptance.
---

# Feature: the README says nothing about the unattended run

## Summary

`superdev run`, `superdev hook run` and `/execute-feature-plan` are
documented in `--help`, the man page and the CLI contract, but the
README — the one user-facing document — never mentions unattended
delivery. A user reading it cannot discover the feature
[I024][sokf:issue-024-feature-request-the-workflow-cannot-run-unattended]
shipped. The same gap for content packs was
[I006][sokf:issue-006-feature-request-content-packs-are-undocumented-for-users].

## Motivation

Acceptance's documentation gate failed: `grep` over the README finds no
`superdev run`, no Stop hook, and no `/execute-feature-plan`, while the
feature adds three verbs to the command tour's surface and a new entry in
every managed repo's `.claude/settings.json` that users will see and
should be able to look up.

## Proposed behaviour

The README describes unattended delivery where it describes the
workflow: what `/execute-feature-plan` does, the branch it works on, the
`superdev run` verbs it drives, the Stop hook that enforces the loop and
its watchdog, and that a repo with no run in progress sees no behaviour
change.

## Acceptance criteria

1. `AC_c1` [ubiquitous] THE SYSTEM SHALL describe the unattended run in the
   README: the driver skill, the run verbs, the Stop hook, the watchdog
   cap, and the do-nothing default without a run.
2. `AC_c2` [event] WHEN the README's command tour lists verbs THE SYSTEM SHALL
   list `superdev run` and `superdev hook run` with the same brevity as
   their neighbours.

## Alternatives considered

- Leaving it to `--help` and the man page — the README is the document a
  user reads first, and I006 already established the standard.

## Scope

- In: the README's workflow and command-tour sections.
- Out: the CLI contract and man page, which already describe the verbs.

<!-- sokf:links -->
[sokf:issue-006-feature-request-content-packs-are-undocumented-for-users]: /knowledge/issues/done/issue-006-feature-request-content-packs-are-undocumented-for-users.md
[sokf:issue-024-feature-request-the-workflow-cannot-run-unattended]: /knowledge/issues/done/issue-024-feature-request-the-workflow-cannot-run-unattended.md
