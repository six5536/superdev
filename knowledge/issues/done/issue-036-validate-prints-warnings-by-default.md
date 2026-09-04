---
type: Issue
id: issue-036-validate-prints-warnings-by-default
title: validate prints warnings by default, which is context noise for an agent
description: Every validate run — the CLI and the PostToolUse hook alike — prints its warnings, so an agent editing one file reads unrelated advisory findings on every pass; warnings should be off by default and asked for.
kind: feature
lifecycle: done
links:
  - rel: references
    to: adr-040-a-warning-is-counted-by-default-and-listed-on-request
    note: Settles the surface, the default for both callers, and what `--json` carries.
  - rel: references
    to: contract-002-cli-superdev
    note: Adds the `--warnings` flag, both counts, and the two `json` keys the binary emitted undeclared.
---

# Feature: validate prints warnings by default

## Summary

`superdev validate` prints warnings alongside errors on every run. An
agent that edits one file gets the whole repository's advisory findings
back through the hook, none of them actionable for the edit in hand.

## Context

The knowledge carries five standing warnings today — four skills with
frontmatter keys outside the Agent Skills spec — reprinted on every
hook-triggered run. Filing the issues on this branch surfaced them
roughly a dozen times without once being acted on.

## Behaviour

A run reports errors and both counts; warnings are shown only when asked
for
([ADR-040][sokf:adr-040-a-warning-is-counted-by-default-and-listed-on-request]).
The summary states how many warnings stand, so nothing is hidden — a
reader always knows something is there.

`--warnings` asks for them, and the CLI and the PostToolUse hook default
alike: one rule, whoever ran it. What is shown changes; what the run
decides does not, so an exit code never moves.

`--json` carries the same information as the text output — both counts
always, and the findings the text run listed. It has no counts today, so
a consumer derives them from `findings`; that has to change first, or
suppressing a warning would lose its count entirely.

The feature is done when validate meets these expectations:

- When `superdev validate` runs without `--warnings`, validate lists
  every error, lists no warning, and states both counts.
- When `superdev validate --warnings` runs, validate lists every finding
  of both severities.
- The PostToolUse hook has the same default as a bare `validate`, so
  one rule governs both.
- `--json` reports the same information as text: both counts always,
  and the findings the text run listed.
- [contract-002][sokf:contract-002-cli-superdev] declares every key
  `--json` emits.
- Validate decides pass and fail exactly as it does today, because the
  default changes what is shown and never what is found.

## Scope

The work is the flag, the shared default and the counts, and stops at
what a warning is.

- In: the `--warnings` flag, the default for the CLI and the PostToolUse
  hook, both counts in the text summary and in `--json`, and the two keys
  `--json` already emits without declaring them — `documents` and
  `schemas`.
- Out: a config key, which decides once for a repository where a flag
  decides per run; a drift test binding the contract's `json` keys to
  what the binary emits, which is what let the two undeclared keys stand
  and is its own piece of work; and which findings are warnings at all,
  which is I012's question and is settled.

Alternatives considered:

- Fix or suppress the five standing warnings individually — treats the
  instance, not the default.
- Promote the decidable warnings to errors, as I012 proposes — a
  different question about a different set of findings.

## Resolution

Delivered by plan-023 (A warning is counted by default and listed on
request) in three slices: the report lists a warning only under
`--warnings`, `--json` carries both counts and the same findings, and
the hooks default like the command line (ADR-040). The `--warnings`
flag, both counts and the `documents` and `schemas` keys are declared in
[contract-002][sokf:contract-002-cli-superdev].

<!-- sokf:links -->
[sokf:adr-040-a-warning-is-counted-by-default-and-listed-on-request]: /knowledge/adrs/active/adr-040-a-warning-is-counted-by-default-and-listed-on-request.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
