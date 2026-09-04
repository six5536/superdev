---
type: Issue
id: issue-055-criterion-number-citations-resolve-to-nothing
title: 55 "criterion N" citations across 19 documents resolve to nothing after the issue rewrite
description: The I052 rewrite dropped the numbering of every acceptance-criteria list along with its keys, and merged each feature request's proposed-behaviour bullets ahead of its criteria in one unnumbered list, so no citation of a criterion by number can be resolved.
kind: bug
lifecycle: open
links:
  - rel: references
    to: issue-052-the-workflow-carries-more-process-than-it-needs
    note: Found at acceptance of I052.
---

# Bug: 55 criterion-number citations resolve to nothing

## Summary

Every citation of the form "criterion N" in the canonical knowledge
points at a numbered list that no longer exists. A reader following one
lands in an unnumbered bullet list where the numbers, where they can be
reconstructed at all, name a different bullet.

## Context

Commit `232986a` rewrote the 52 issues on file into `schema-issue`. An
old feature request carried a "Proposed behaviour" bullet list and a
separate "Acceptance criteria" numbered list, each item opening with an
`AC_` key. The rewrite merged the two into one unnumbered bullet list
under Behaviour. Dropping the keys was the point of
[I052][sokf:issue-052-the-workflow-carries-more-process-than-it-needs];
dropping the numbering was not.

`grep -rEo "criteri(on|a) [0-9]+" knowledge/` returns 55 matches in 19
documents.

`knowledge/issues/done/issue-035-a-contract-does-not-define-its-interface.md`
is the worst case. Its six proposal bullets now precede its fifteen
former criteria in one list of 21, so every citation of it is off by
six. Former criterion 4, the demand for a drift test, is bullet 10;
former criterion 12, the demand that a drift failure name its direction,
is bullet 18.

Live documents carry these citations, not history alone.

- `knowledge/issues/index.md` cites "I035 criterion 4" and "I035
  criterion 12" in the entries for I038 and I044.
- `knowledge/glossary.md` cites "I049 criterion 23" under the drift-test
  term.
- `knowledge/issues/open/issue-026-rehearse-the-driver-on-a-real-feature.md`
  cites "criteria 5, 7, 8 and 9" of I024 in a link note.
- `knowledge/issues/open/issue-049-a-contract-cannot-point-at-its-definition.md`
  cites five ranges of its own criteria in its link notes.
- I035's own Comments section cites "criterion 4".

## Behaviour

A reader resolves every citation of a criterion. The repository decides
how, and the choice binds every citing document at once.

Restoring the numbering makes each Behaviour list numbered again, and
every citation resolves by counting — but the merge has already moved
the numbers, so each of the 55 citations must be recomputed against the
list it names. Rewriting the citations leaves the lists unnumbered and
replaces each "criterion N" with the bullet's own words, which survives
any later edit to the list. Under either choice the merged lists that
mix proposal bullets with former criteria must be read once to establish
what each citation meant.

## Scope

The 55 citations, and the lists they point into.

- In: the 19 documents the grep names, live and settled alike.
- In: `knowledge/issues/index.md`, `knowledge/glossary.md`, I026, I049
  and I035, whose citations a reader follows today.
- In: the merged Behaviour lists of the rewritten feature requests,
  where a proposal bullet now sits among former criteria.
- Out: the removal of the `AC_` keys, which I052 decided.
- Out: citations of a contract promise or criterion by key, which
  resolve as they always did.
- Out: any new rule about how a criterion is cited; the existing
  convention is the contract's key.

<!-- sokf:links -->
[sokf:issue-052-the-workflow-carries-more-process-than-it-needs]: /knowledge/issues/done/issue-052-the-workflow-carries-more-process-than-it-needs.md
