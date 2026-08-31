---
type: Decision
id: adr-022-a-frontmatter-key-is-required-by-a-per-key-flag
title: A Frontmatter Key Is Required by a Per-Key Flag
description: A schema marks a required frontmatter key with a `required` flag beside that key's constraints, mirroring the section rules' own vocabulary, so requiredness reads where the key is declared.
lifecycle: active
---

# ADR-022: A Frontmatter Key Is Required by a Per-Key Flag

- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

I018 makes the validator read the frontmatter contract a schema
declares. Value constraints — `const`, `pattern`, `enum` — check a key
that is present, but nothing in the vocabulary says a key must be
present: `status` carries an enum and is optional everywhere, while a
filed document without its `id` should not pass. Requiredness cannot be
inferred from the constraints, so the schemas need a way to state it,
and the shape ships in the pack to every managed repository.

## Decision

We will mark a required frontmatter key with `required: true` inside
that key's own constraint block:

```yaml
frontmatter:
  id:
    required: true
    pattern: '^issue-\d{3}-feature-request-[a-z0-9-]+$'
  status:
    enum: [draft, stable]
```

An absent key marked required is an error naming the document, the key
and the schema. A key without the flag is optional; its constraints
bind only when it is present.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| `required: true` per key | Reads beside the constraints it strengthens; the same word the section rules already use | Requiredness is spread over the keys rather than listed once |
| A `frontmatter-required:` list | The required set in one place | Names every key a second time, and the list can drift from the keys |
| Infer from having constraints | No new vocabulary | Wrong: `status` has an enum and is optional; inference cannot distinguish the cases |

## Consequences

- Positive: one requiredness vocabulary across both halves of a schema —
  sections and frontmatter say `required: true` alike.
- Negative: the 53 schemas must each be visited once to declare their
  required keys; until a schema does, its keys stay optional.
- Follow-ups: the declaration pass over the shipped schemas, in the
  feature's reconciliation slice.
