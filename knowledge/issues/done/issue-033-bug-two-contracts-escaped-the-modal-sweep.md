---
type: BugReport
id: issue-033-bug-two-contracts-escaped-the-modal-sweep
title: Contracts 002 and 003 carry no RFC 2119 keyword after the sweep
description: The binding-surface sweep trimmed the CLI and MCP contracts but never gave their promises modal verbs, so I029's criterion 1 fails on both and criterion 5 fails with them — or the standard needs to learn the difference between describing behaviour and stating a requirement.
lifecycle: done
links:
  - rel: references
    to: issue-029-bug-contract-design-writes-verbose-prose
    note: The acceptance gap on criteria 1 and 5; found walking I029's criteria on the merged feature head.
---

# Bug: contracts 002 and 003 carry no RFC 2119 keyword after the sweep

## Resolved

Resolved by P019 slice 6: the strict remedy — modal verbs on every
promise in 002 and 003, criterion 1 kept as written; the standard
refinement was rejected.

## Summary

The plan-019 sweep trimmed reasoning from
[contract-002][sokf:contract-002-cli-superdev] and
[contract-003][sokf:contract-003-mcp-sokf] but left both without a
single RFC 2119 keyword, while their bodies are full of promises
callers rely on — "`status` never writes", "`--fix` is the one way
`validate` writes". Under
[I029][sokf:issue-029-bug-contract-design-writes-verbose-prose]'s
criterion 1 every normative statement uses a modal verb, so criteria 1
and 5 fail on the feature head.

## Environment

- Version/commit: feature/contract-design-review head, 2026-09-01
- Platform: any; the defect is contract prose

## Steps to reproduce

1. Run `grep -c -E 'MUST|SHALL|SHOULD|MAY ' ` over the nine active
   contracts.
2. Observe 002 and 003 at zero while the other seven carry keywords.

## Expected behaviour

Every active contract conforms to I029's criteria 1–4. Two remedies
compete, settled at framing: give 002 and 003 modal verbs on their
promises, or refine the standard (an ADR-029 amendment carried into the
contract-style fragment) to distinguish behaviour descriptions —
present tense, the code canonical — from requirements, which alone
take modal verbs. The second remedy revisits what criterion 1 counts
as a normative statement.

## Actual behaviour

Zero RFC 2119 keywords in 002 and 003; promises stated as bare
declaratives.

## Root cause (if known)

The sweep read 002 and 003 as "dense but sound" and trimmed reasoning
clauses only; the feature-wide review's census flagged 004 and 008 —
the two untouched files — and the touched-but-unmodalised pair escaped
both passes.

## Proposed fix / workaround

- Fix: settled at framing — modal verbs on the promises, or the
  description/requirement distinction added to ADR-029 and the
  fragment, then applied consistently.
- Workaround: none needed; the contracts' meaning is unchanged.

## Regression risk

Contract prose only; `superdev validate` pins the documents' shape and
the goldens pin nothing here. A standard refinement touches the
fragment, which rematerializes into all 15 schemas on the next `--fix`.

## Comments

2026-09-01, at acceptance. The user chose the strict remedy: keep
criterion 1 as written and give 002 and 003 modal verbs on their
promises; the description/requirement refinement of ADR-029 was
rejected. Delivered as plan-019 slice 6.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-003-mcp-sokf]: /knowledge/contracts/public/active/contract-003-mcp-sokf.md
[sokf:issue-029-bug-contract-design-writes-verbose-prose]: /knowledge/issues/done/issue-029-bug-contract-design-writes-verbose-prose.md
