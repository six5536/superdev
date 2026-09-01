---
type: FeatureRequest
id: issue-036-feature-request-validate-prints-warnings-by-default
title: validate prints warnings by default, which is context noise for an agent
description: Every validate run — the CLI and the PostToolUse hook alike — prints its warnings, so an agent editing one file reads unrelated advisory findings on every pass; warnings should be off by default and asked for.
lifecycle: open
---

# Feature: validate prints warnings by default

## Summary

`superdev validate` prints warnings alongside errors on every run. An
agent that edits one file gets the whole repository's advisory findings
back through the hook, none of them actionable for the edit in hand.

## Motivation

The knowledge carries five standing warnings today — four skills with
frontmatter keys outside the Agent Skills spec — reprinted on every
hook-triggered run. Filing the issues on this branch surfaced them
roughly a dozen times without once being acted on.

## Proposed behaviour

A run reports errors and the counts; warnings are shown only when asked
for. The summary still states how many warnings were suppressed, so
nothing is hidden. Surface: TBD — a flag, a config key, or a difference
between the CLI and the hook.

## Acceptance criteria

1. TBD — what a default run prints, and whether the warning count stays
   in the summary line.
2. TBD — how a caller asks for warnings, and whether the CLI and the
   PostToolUse hook default alike.
3. TBD — whether `--json` is affected, given a machine consumer has no
   context cost.

## Alternatives considered

- Fix or suppress the five standing warnings individually — treats the
  instance, not the default.
- Promote the decidable warnings to errors, as I012 proposes — a
  different question about a different set of findings.

## Scope

- In: TBD.
- Out: TBD.
