---
type: Issue
id: issue-022-a-schemas-worked-example-is-checked-by-nothing
title: A schema's worked example is the thing agents copy, and it is the one part of the schema nothing checks
description: Every schema carries an `example:` block showing a conforming document, and no check reads it — five of the twenty-three example ids on file broke their own schema's id pattern, left behind by a migration that changed the pattern and not the example.
kind: feature
lifecycle: done
links:
  - rel: relates-to
    to: issue-018-the-schema-layer-checks-sections-and-nothing-else
  - rel: references
    to: contract-010-interface-document-schemas
    note: The vocabulary gains the example key's obligation and the link-form rule, per ADR-024 and ADR-025.
  - rel: references
    to: contract-002-cli-superdev
    note: validate's schema half grows the example check.
---

# Feature: a schema's worked example is checked by nothing

## Summary

Every schema carries an `example:` block showing one conforming document.
It sits inside a fenced YAML scalar, so no check reads it — not the schema
layer, which checks documents against schemas, and not the SOKF layer,
which reads links and footnotes from a body and sees a fence as opaque
text. An agent writing a new document copies the example.

## Context

Measured on this repository today: of the 23 example documents declared
across the schemas, **five carried an `id` its own schema's `id` pattern
refuses**.

| Schema | Example id | Its own pattern |
|--------|-----------|-----------------|
| `schema-adhoc-plan` | `adhoc-plan-002-scheme-match-cleanup` | `^plan-\d{3}-adhoc-[a-z0-9-]+$` |
| `schema-bug-report` | `issue-042-pack-sync-etimedout` | `^issue-\d{3}-bug-[a-z0-9-]+$` |
| `schema-chore` | `issue-042-drop-the-legacy-cache-directory` | `^issue-\d{3}-chore-[a-z0-9-]+$` |
| `schema-feature-plan` | `feature-plan-001-pack-source-allowlist` | `^plan-\d{3}-feature-[a-z0-9-]+$` |
| `schema-feature-request` | `issue-042-validate-reports-machine-readable-json` | `^issue-\d{3}-feature-request-[a-z0-9-]+$` |

All five were left by the commit that gave issues and plans one filename
convention: it changed each pattern and missed the example beside it, and
nothing said so for the four commits since. They were corrected by hand
alongside P010, which is the point — a hand correction is the only thing
that finds them, and only when someone happens to look.

The example is not decoration. It is the one part of a schema an agent
reads and copies verbatim, so a wrong example is a wrong document
generator. It also carries the authority of the checked half of the file:
a reader who sees the section rules enforced assumes the example beside
them was enforced too.

The same blindness covers more than the id. An example's sections, its
ordering, its content kinds and — since SOKF 0.4 — its body links are all
unread, so an example may teach a path link where the format now requires
`[text][sokf:<id>]`.

## Behaviour

`validate` reads each schema's `example:` block as a document and checks
it against the schema that declares it, reporting a failure the way it
reports any other schema finding — naming the schema file and what the
example broke. The example has a known governing schema — the one it sits
in — so this needs no dispatch: the document check that already runs over
a real document runs over the example with the schema handed to it.

The check covers the schema layer — the frontmatter contract, the
sections, their order and content kinds — and the form of the example's
body links, never their destination. A link to a concept takes the
`[text][sokf:<id>]` reference form; a path link into the knowledge is
refused. No id or target is resolved, so an example cites fictional
concepts freely, and a link pointing outside the knowledge — a URL, a
repository path — keeps its ordinary markdown form. An example that does
not parse as a document at all is itself a schema finding.

- When a schema's example, read as a document, breaks the declaring
  schema's frontmatter contract — a value failing its `const`, `pattern`
  or `enum`, or an absent required key — validate reports an error
  naming the schema file and what the example broke.
- When a schema's example breaks the declaring schema's section rules —
  a required section absent, sections misordered, a prohibited section
  present, or a section's body lacking its declared content kind —
  validate reports an error naming the schema file and what the example
  broke.
- When a schema's example links to a concept by a path into the
  knowledge, validate reports an error naming the schema file; the
  `[text][sokf:<id>]` form is the accepted form for a concept link.
- Validate accepts, without resolution, an example whose ids and `sokf:`
  labels name nothing in the knowledge, and an example link whose target
  is outside the knowledge — a URL or a repository path — in its
  ordinary markdown form.
- When a schema's example does not parse as a document — no frontmatter
  block, or frontmatter that is not YAML — validate reports an error
  naming the schema file.
- Validate reports PASS on this repository once the feature's
  reconciliation lands: every schema's example satisfies its own schema,
  or the schema was deliberately corrected.

## Scope

The check reaches every schema's example and reads its form alone.

- In: checking each schema's `example:` block against its own schema —
  frontmatter, sections and body — and reporting failures as schema
  findings.
- In: the check reaching every schema, including those a managed
  repository ships once
  [the schemas ship][sokf:issue-020-the-schemas-do-not-ship].
- Out: the frontmatter and content-kind checks themselves —
  [I018][sokf:issue-018-the-schema-layer-checks-sections-and-nothing-else]
  delivered them; this issue feeds the example to those checks.
- Out: resolving an example's links or ids against the knowledge. The
  check reads form alone; an example's content is fictional by design.
- Out: fenced examples in documents that are not schemas. A plan or a spec
  may show whatever illustrates its argument.

Alternatives considered:

- **Extract examples into real files under a fixture tree.** They would
  then be checked by the existing machinery with no new code, but a schema
  whose example lives elsewhere is a schema a reader cannot read on its
  own, and the example is where it is precisely so it is read.
- **Check only the `id`.** Cheapest, and it covers all five faults found.
  Rejected: the id was wrong because nothing looked, and the sections and
  links are unlooked-at for the same reason. A check scoped to the fault
  that happened to surface leaves the next one to surface the same way.
- **Leave it and correct examples when noticed.** That is the state this
  issue describes, measured at five faults across four commits.

## Resolution

Delivered by [plan-017][sokf:plan-017-feature-example-conformance] in an
unattended run. Acceptance on 2026-08-31 walked all six criteria end to
end against the feature branch head with the full suite passing
(640 tests): every fault class — a frontmatter value or required key, a
section or content kind, a path link into the knowledge, an example that
does not parse — reports an error on the schema file prefixed
`example:`; a conforming example with fictional `sokf:` labels and
external links passes without resolution; and `superdev validate`
reports PASS on this repository. The behaviour is documented in
[contract-002][sokf:contract-002-cli-superdev] and
[contract-010][sokf:contract-010-interface-document-schemas], the
feature-wide review's five findings were fixed before the merge
(code-review-002), and a security review of the feature diff found no
vulnerabilities.

## Comments

2026-08-31 — A hand review of all 53 schemas found 26 examples breaking
their own frontmatter constraints: 18 carried a `type` the schema's
`const` refuses, and 8 omitted frontmatter the schema constrains.
Plan-014 fixed every instance by hand. The count strengthens the case
for the checker: the five id faults first recorded here were the
surfaced fraction of a fault class five times that size.

2026-08-31 — Framing settled the check's reach with the user: the
schema-layer document check plus link form. A concept link in an example
must take the `[text][sokf:<id>]` form and a path link into the
knowledge is refused, but no target is ever resolved — an example's
content is fictional by design, and a link may point outside the
knowledge, where it keeps its ordinary markdown form.

2026-08-31 — Contract-design landed the trace: the
[document-schemas interface contract][sokf:contract-010-interface-document-schemas]
gains the example key's obligation and the link-form rule, per ADR-024
and ADR-025, and the
[CLI contract][sokf:contract-002-cli-superdev]'s validate bullet grows
the example check.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-018-the-schema-layer-checks-sections-and-nothing-else]: /knowledge/issues/done/issue-018-the-schema-layer-checks-sections-and-nothing-else.md
[sokf:issue-020-the-schemas-do-not-ship]: /knowledge/issues/done/issue-020-the-schemas-do-not-ship.md
[sokf:plan-017-feature-example-conformance]: /knowledge/plans/done/plan-017-feature-example-conformance.md
