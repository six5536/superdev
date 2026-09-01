---
type: FeatureRequest
id: issue-036-feature-request-validate-prints-warnings-by-default
title: validate prints warnings by default, which is context noise for an agent
description: Every validate run — the CLI and the PostToolUse hook alike — prints its warnings, so an agent editing one file reads unrelated advisory findings on every pass; warnings should be off by default and asked for.
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

## Motivation

The knowledge carries five standing warnings today — four skills with
frontmatter keys outside the Agent Skills spec — reprinted on every
hook-triggered run. Filing the issues on this branch surfaced them
roughly a dozen times without once being acted on.

## Proposed behaviour

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

## Acceptance criteria

1. [event] WHEN `superdev validate` runs without `--warnings` THE SYSTEM
   SHALL list every error, list no warning, and state both counts.
2. [event] WHEN `superdev validate --warnings` runs THE SYSTEM SHALL list
   every finding of both severities.
3. [ubiquitous] THE SYSTEM SHALL give the PostToolUse hook the same
   default as a bare `validate`, so one rule governs both.
4. [ubiquitous] THE SYSTEM SHALL report the same information in `--json`
   as in text: both counts always, and the findings the text run listed.
5. [ubiquitous] THE SYSTEM SHALL declare in
   [contract-002][sokf:contract-002-cli-superdev] every key `--json`
   emits.
6. [ubiquitous] THE SYSTEM SHALL decide pass and fail exactly as it does
   today, because the default changes what is shown and never what is
   found.

## Alternatives considered

- Fix or suppress the five standing warnings individually — treats the
  instance, not the default.
- Promote the decidable warnings to errors, as I012 proposes — a
  different question about a different set of findings.

## Scope

- In: the `--warnings` flag, the default for the CLI and the PostToolUse
  hook, both counts in the text summary and in `--json`, and the two keys
  `--json` already emits without declaring them — `documents` and
  `schemas`.
- Out: a config key, which decides once for a repository where a flag
  decides per run; a drift test binding the contract's `json` keys to
  what the binary emits, which is what let the two undeclared keys stand
  and is its own piece of work; and which findings are warnings at all,
  which is I012's question and is settled.

<!-- sokf:links -->
[sokf:adr-040-a-warning-is-counted-by-default-and-listed-on-request]: /knowledge/adrs/active/adr-040-a-warning-is-counted-by-default-and-listed-on-request.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
