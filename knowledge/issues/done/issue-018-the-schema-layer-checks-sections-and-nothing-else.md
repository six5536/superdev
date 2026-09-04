---
type: Issue
id: issue-018-the-schema-layer-checks-sections-and-nothing-else
title: A schema declares content kinds and a frontmatter contract, and the validator reads neither
description: P008 made schemas govern documents, but only their sections — the content kind under each heading and the frontmatter constraints beside it are declared on every schema and read by nothing, which is the fault P008 set out to cure, one level down.
kind: feature
lifecycle: done
links:
  - rel: references
    to: contract-002-cli-superdev
    note: validate's schema half grows the content-kind and frontmatter checks.
  - rel: references
    to: contract-010-interface-document-schemas
    note: New — the declaration vocabulary the schemas write and the validator reads.
---

# Feature: the schema layer checks sections and nothing else

## Summary

P008 made `superdev validate` check documents against the schema their `type`
names, and it checks the sections: present, ordered, not prohibited, table
columns, line limit. Two other families of constraint sit in every schema and
are read by nothing — the `content` kind declared per section, and the
`frontmatter` contract declared beside it.

## Context

This is the fault P008 was written to cure, one level down. `target-files` was
declared required on forty schemas and read by nothing, so no document was
ever checked; the fix left two smaller versions of the same thing in place.

Measured on this repository today:

- **Content kinds.** 33 sections declare a `content` kind their body does not
  match, across ten schemas — nearer 10 under the lead-in rule framing
  settled (Comments), and either count is a field read by nothing.
- **Frontmatter contracts.** `schema-adhoc-plan` declares
  `id: pattern '^plan-\d{3}-adhoc-[a-z0-9-]+$'` and the `title` and
  `description` constraints beside it, and no check reads any of them — an
  id that matched nothing would pass unexamined. The `lifecycle` key is the
  one exception: plan-011 taught the validator to read it, which shows the
  gap is closable one key at a time.

A reader of a schema cannot tell which half of it binds. That is worse than a
schema that checks nothing, because the checked half lends the unchecked half
its authority.

## Behaviour

`validate` reports, as errors in the same shape as the section findings —
the document, the rule, and the schema that declares it:

- a section whose body is not the kind its schema declares — prose,
  bullet-list, numbered-list, table or code. The kind binds the section's
  substance, so a list section may open with a lead-in sentence before its
  list;
- a present frontmatter value that breaks the `const`, `pattern` or `enum`
  beside it. A key declared with only a description is guidance and is not
  checked;
- an absent frontmatter key its schema marks required. The schemas gain a
  way to mark one, and each of the 53 declares which of its keys are —
  today nothing distinguishes "must carry a title" from "a title means
  this".

A schema that itself declares a content kind outside that vocabulary, a
constraint the validator cannot read, or an unparseable `pattern` is
reported as a schema finding, so a mis-declared rule surfaces on the schema
rather than silently binding nothing.

The criteria a reader checks one by one:

- When a section's body does not match the content kind its schema
  declares — prose, bullet-list, numbered-list, table or code — the
  validator reports an error naming the document, the section and the
  schema.
- The validator accepts a lead-in sentence before the list in a bullet-list
  or numbered-list section.
- When a present frontmatter value breaks the `const`, `pattern` or `enum`
  its schema declares for that key, the validator reports an error naming
  the document, the key and the schema.
- When a frontmatter key a schema marks required is absent, the validator
  reports an error naming the document, the key and the schema.
- If a schema declares a content kind outside the vocabulary or a `pattern`
  that does not compile, the validator reports the schema file itself.
- The validator reports PASS on this repository once the feature's
  reconciliation lands: every live document satisfies its schema's content
  kinds and frontmatter contract, or the schema was deliberately corrected.

## Scope

The boundary as drawn at framing:

- In: the `content` kinds; the `frontmatter` `const`, `pattern` and `enum`
  checks on present values; the vocabulary for marking a key required and
  the pass over the 53 schemas that declares it; and the reconciliation of
  every live finding the new checks surface.
- Out: checking each schema's worked example against its own schema —
  that is [issue-022][sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing],
  which lands on top of this machinery.
- Out: index shape and index-entry checks — I011 and I010.
- Out: whether `schema-templates-index` should exist at all. It governs
  `knowledge/templates/index.md`, and indexes carry no frontmatter and are
  deliberately excluded from the candidate list, so it is the one schema left
  that can never fire.

Alternatives considered:

- Delete the unread declarations from the grammar instead. Cheaper, and it
  throws away the guidance the descriptions carry for an agent writing a new
  document — which is most of what a schema is for.
- Check frontmatter only, and leave content kinds. The frontmatter half is
  unambiguous and would land in an afternoon; the content half needed the
  lead-in question settled first. A defensible order, not a different
  outcome.
- Land the checks as warnings and promote them once the tree is clean.
  Rejected: warnings here go unactioned (I012 measured 39), and ADR-017
  made knowledge validation pass-or-fail.
- Check values only and leave key presence unstated. Rejected by the user:
  a schema that cannot require a key cannot express its own contract, and
  SOKF requires only `type`.

## Resolution

Delivered by [plan-016][sokf:plan-016-feature-schema-layer-enforcement].
Acceptance on 2026-08-31 walked all six criteria end to end against the
feature branch head, with the full test suite passing (625 tests): a
content-kind, pattern, enum and required-key fault each report an error
naming the document, the rule and the schema; a lead-in sentence before a
list passes; a mis-declared schema — an unknown content kind, an
uncompilable pattern — is reported on the schema file and binds nothing;
and `superdev validate` reports PASS on this repository. The behaviour is
documented in
[contract-002][sokf:contract-002-cli-superdev] and
[contract-010][sokf:contract-010-interface-document-schemas]. A security
review of the feature diff found no vulnerabilities.

## Comments

2026-08-31 — Framing settled four decisions with the user: a list section
admits a lead-in sentence (the kind binds the section's substance); the
new findings are errors, per ADR-017's pass-or-fail stance; the
frontmatter check covers both present values and required-key presence,
with the schemas gaining the vocabulary to mark a key required; and
issue-022's example checking stays a separate feature.

2026-08-31 — Contract-design landed the trace: the
[CLI contract][sokf:contract-002-cli-superdev]'s validate bullet grows
the new checks, and the new
[document-schemas interface contract][sokf:contract-010-interface-document-schemas]
fixes the declaration vocabulary, per ADR-022 and ADR-023.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing]: /knowledge/issues/done/issue-022-a-schemas-worked-example-is-checked-by-nothing.md
[sokf:plan-016-feature-schema-layer-enforcement]: /knowledge/plans/done/plan-016-feature-schema-layer-enforcement.md
