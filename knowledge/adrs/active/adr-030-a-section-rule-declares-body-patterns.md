---
type: Decision
id: adr-030-a-section-rule-declares-body-patterns
title: A section rule declares body patterns
description: A section rule may declare item-pattern, checked against each top-level item of its declared list kind, and content-pattern, checked against the section's whole body — regexes matched found-anywhere with explicit anchors, and a mis-declaration is a finding on the schema.
lifecycle: active
---

# ADR-030: A section rule declares body patterns

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

The shapes normative text takes — EARS criteria, RFC 2119 requirements
— are stated in schema `description` prose the validator never reads
(I034). The declaration vocabulary stops at the section level: content
kinds, heading patterns and table columns are enforced, and nothing
below them is decidable. The engine's regex checks already use search
semantics (`Regex::is_match`), and every pattern on file carries
explicit anchors.

## Decision

A section rule gains two declarations. `item-pattern` binds each
top-level item of the section's declared list kind: the item's text is
its own lines with the marker stripped and continuations joined, nested
items excluded; declaring it on a section whose `content` is not
`bullet-list` or `numbered-list` is a finding on the schema.
`content-pattern` binds the section's body: the raw lines from below
the heading to the next heading at the same or shallower level. Both
are regexes matched found-anywhere; authors write `^` and `$`
explicitly. A pattern that does not compile is a finding on the schema
and binds nothing. An item-pattern finding names the file, the section
and the item's first line; a content-pattern finding names the file and
the section.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Found-anywhere body patterns, both levels | Matches the engine's existing semantics and the JSON Schema idiom; one mechanism serves items, sections and subsections | A pattern with a forgotten anchor is weaker than intended |
| Whole-match anchoring | An unanchored pattern cannot under-match | Different semantics from what the engine does today; "contains" rules become `.*X.*` |
| Per-line matching | Simple to implement | A wrapped list item breaks mid-sentence and cannot match |
| A built-in named grammar (`ears: true`) | Exact for one shape | Special-cases one grammar; a regex expresses the same check today |
| item-pattern only | Smaller vocabulary | Prose- and subsection-level shapes stay unenforceable — the I034 gap, one level up |

## Consequences

- Positive: a schema states a shape once and the PostToolUse hook
  enforces it on every edit, in this repository and every managed one.
- Negative: a regex gates form, not meaning — a well-shaped wrong
  sentence still passes.
- Follow-ups: ADR-031 applies the vocabulary to EARS criteria, ADR-032
  to the contract kinds; contract-010 gains the rows.
