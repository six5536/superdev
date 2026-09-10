---
type: Issue
id: issue-089-an-exact-identifier-does-not-win-its-own-search
title: An exact identifier does not win its own search
description: A query naming a concept by its identifier ranks that concept sixth behind topical neighbours, because rank fusion spreads scores too narrowly, no field boosts an identifier match, and settled reports carry no lifecycle to downrank.
kind: bug
lifecycle: open
---

# Bug: an exact identifier does not win its own search

## Summary

`sokf_search` ranks a concept below unrelated neighbours when the query names
that concept by its identifier. A reviewer searching for `contract-002` reads
five other documents before reaching it. Three ranking behaviours combine to
produce that, and each is decidable from the index code.

## Context

Observed across fourteen searches in three requirements reviews of
`plan-077-sokf-file-tool-parity`. One query names its target exactly:

```text
query: CLI sokf read command promises knowledge-relative path offset limit contract-002

1. architecture.md
2. code-review-005-normative-shape-enforcement
3. documentation.md
4. plan-077-sokf-file-tool-parity > Requirements
5. plan-077-sokf-file-tool-parity > Goal and boundaries
6. contract-002-cli-superdev
7. development-commands.md
8. adr-026-a-named-document-is-checked-with-bare-run-parity
```

The literal string `contract-002` appears in the query. The concept it names
ranks sixth, below a historical code-review report about an unrelated plan.

Three causes are visible in `crates/lib/superdev-core/src/sokf/index.rs`.

Rank fusion compresses the result. `rrf()` scores a hit as `1.0 / (RRF_K +
rank)` with `RRF_K = 60`, so first place scores 0.0167 and eighth scores
0.0149, a spread of about eleven percent. A section ranked mediocrely by both
the lexical and the semantic half outscores one ranked first by a single half.
That constant suits long web result lists rather than eight sections drawn from
265 files.

Nothing boosts an identifier. `lexical_keys()` parses the query against one
field, `fields.text`. The schema also stores `id`, `title`, `kind`,
`lifecycle`, and `tags`, and none participates in scoring. A concept's own
identifier is therefore worth exactly as much as the same string appearing in
another document's prose.

Settled work is not always downranked. `downrank()` multiplies a settled
section's score by `DOWNRANK_FACTOR`, and `settled()` decides that from
`lifecycle` not being `open` or `active`, or from `status` being `deprecated`.
Ten of the eleven documents in `knowledge/reports/` carry no `lifecycle` field,
so `is_some_and` returns false and no penalty applies. A finished code review
from an earlier plan competes on equal terms with live knowledge, which is how
`code-review-005` reached second place.

## Behaviour

A query naming a concept by its identifier returns that concept first.

Identifier and title matches must carry more weight than a match in body
prose. The fields are already indexed and stored, so the change is in what
scoring consults rather than in what the index holds.

Rank fusion must separate its inputs enough for a strong single-half match to
beat a weak agreement between halves. `RRF_K` is the lever, and the right
value depends on result-list length rather than convention: eight hits from a
265-file corpus is a different regime from the web-scale lists the constant of
60 comes from.

A finished code review must rank below live knowledge. Either those documents
declare a lifecycle, or `settled()` learns to treat a concept type that is
inherently historical as settled without one. The first is a knowledge change
and the second is an index change, and choosing between them decides whether
every future report has to remember a field.

Hybrid retrieval itself is not the problem, and the record should say so. The
same fourteen searches show semantic retrieval doing its job: a query about the
documentation map returned `documentation.md` and `schema-documentation`
without either being named. What fails is the case where the caller already
knows the answer's name, and the ranking has no way to prefer it.

## Scope

How `sokf_search` ranks what it has already found.

- In: the fields lexical scoring consults, the fusion constant, and whether a
  settled concept without a `lifecycle` value is recognised as settled.
- In: whether an exact identifier in a query should short-circuit ranking
  rather than merely outweigh prose.
- Out: recall. Every observed query returned the relevant concept somewhere in
  its results; the defect is where.
- Out: what the tool tells its caller about phrasing and filters, which is a
  separate issue about how the tool is used rather than how it ranks.
- Out: what the index covers. Extending it beyond `knowledge/` is a different
  question from ordering what is already there.

## Comments

Filed from a live session after analysing the search calls in three
requirements reviews. The ranking constants, the scoring fields, and the
settled-work rule were read before filing, and the missing `lifecycle` on ten
of eleven reports was confirmed directly rather than inferred from the result
order.

One limitation of the evidence is worth recording. Fourteen queries from one
plan in one domain establish that identifier-anchored queries rank badly; they
do not establish how the ranking behaves across the corpus generally. No
counterfactual was run, so what a lexical-only or semantic-only search would
have returned for the same queries is unknown.
