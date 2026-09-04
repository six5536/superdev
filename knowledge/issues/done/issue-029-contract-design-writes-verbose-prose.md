---
type: Issue
id: issue-029-contract-design-writes-verbose-prose
title: contract-design writes contracts as verbose prose
description: Contract documents come out as long prose where a contract needs precision — clear normative statements and constructs that aid them, such as tables, lists and typed shapes.
kind: bug
lifecycle: done
links:
  - rel: references
    to: issue-028-contract-design-commits-before-the-go-ahead
    note: Shares a cause — the phase settles its output without review — and ships in the same fix.
  - rel: references
    to: contract-002-cli-superdev
    note: The --fix bullet grows the include-block materialization that carries the standard into the schemas (ADR-027).
---

# Bug: contract-design writes contracts as verbose prose

## Summary

`/contract-design` tends to produce contracts as long prose. A contract
must be clear and precise; prose buries the normative statements, and
the constructs that carry precision — tables, lists, typed shapes,
RFC 2119 sentences — go unused.

## Context

Observed on main at 05f8731, in any session running the SOKF-carried
skill set; the defect is prose, in
`.claude/skills/contract-design/SKILL.md` and possibly the contract
schemas it follows.

- Run `/contract-design` on a framed issue that touches a contract.
- Read the contract document it writes or updates.

## Behaviour

A contract is a binding surface, not a specification. The standard
applies to every contract kind alike — CLI, code interface, config,
file format, MCP and the rest — because it constrains how a surface is
written while each kind's schema names the surface's native form. As
criteria:

- When contract-design writes or updates a contract, every normative
  statement uses an RFC 2119 modal verb, one requirement per sentence.
- When a contract's surface is enumerable — commands, flags, keys,
  types, error cases, limits — contract-design expresses it in the
  kind's native structured form: a code block, table or list. Prose,
  doc comments included, describes and does not define.
- A contract binds only what callers rely on; behaviour a contract does
  not list is the code's to decide.
- A contract links the ADR behind each decision and does not restate
  the ADR's reasoning.
- When the fix ships, every active contract — the nine on file —
  conforms to criteria 1–4.

Instead, contract sections arrive as extended paragraphs; the
requirements are embedded in narrative and take effort to extract.

The root cause was confirmed in part at framing. The skill's prose sets
no style requirement for contract text, and where a schema does set one
— the interface schema's "Prose describes; it never defines", with
`content: code` — the prose migrates into doc comments inside the
code fences, which no check reads: contract-007 carries 40-line doc
comments retelling ADR reasoning. Shares a cause with
[I028][sokf:issue-028-contract-design-commits-before-the-go-ahead]:
the phase settles its output without the user's review, so verbosity
is never pushed back on. Where the standard is recorded — the skill's
prose, the contract schemas, or both — is settled in CONTRACT-DESIGN.

## Scope

The fix records the standard and sweeps the existing contracts.

- Fix: settled in CONTRACT-DESIGN — record the binding-surface
  standard where contract-design reads it, and sweep the nine active
  contracts to conform.
- Workaround: ask for a rewrite of the contract after the phase runs.

Regression risk: the contract schemas govern every existing contract on
file; a schema change may make settled contracts fail validation — the
sweep resolves that by rewriting the nine together. The sweep must not
change what any contract binds: it compresses form, never meaning, and
build codes against these documents.

## Resolution

Fixed by P019: the binding-surface standard (ADR-029) ships inside
every contract-kind schema through the include block (ADR-027), the
skill binds writers to it, and all nine active contracts conform —
the acceptance gap on 002 and 003 (I033) closed by slice 6. Verified
at acceptance by keyword census and reading.

## Comments

2026-09-01, framed. The user set two bounds on the recommended
standard: contracts must not become specifications carrying every
detail, and the standard must serve CLIs, code APIs and other APIs
alike. Both are folded into the criteria — criterion 3 is the
not-a-spec bound as a scope rule, and the standard is kind-agnostic by
construction. Back-catalog decision: sweep all nine active contracts
now, so the corpus stops teaching the old style by example.

2026-09-01, contract-design. The standard is carried into all 15
contract-kind schemas by a materialized include block (ADR-027) — the
user rejected a skill companion because a writer editing a contract
outside the skill reads only the schema, and rejected inheritance as
heavier than the need — so
[contract-002][sokf:contract-002-cli-superdev]'s `--fix` bullet grows
the materialization. The standard itself is ADR-029; the skill's
restructured interaction is ADR-028.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:issue-028-contract-design-commits-before-the-go-ahead]: /knowledge/issues/done/issue-028-contract-design-commits-before-the-go-ahead.md
