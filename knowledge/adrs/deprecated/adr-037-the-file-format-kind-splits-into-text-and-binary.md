---
type: Decision
id: adr-037-the-file-format-kind-splits-into-text-and-binary
title: The file-format kind splits into text and binary
description: The file-format contract kind becomes two — a text format, whose shape is a schema or a worked example carrying every key, and a binary format, whose shape is a byte layout — and the three contracts on file are renamed to the kind they belong to.
lifecycle: deprecated
---

# ADR-037: The file-format kind splits into text and binary

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

ADR-033 requires each kind to demand the whole of its surface, and one
file-format kind cannot: the shape of a text format is a schema or a
worked example carrying every key, while the shape of a binary format
is a byte layout — offsets, widths, endianness, a magic number and a
version field. A schema demanding one is wrong for the other, and
"file format" names the pair rather than either. superdev ships these
schemas to repositories that hold both kinds of file.

## Decision

The file-format kind becomes two. A text format contract declares its
shape as a schema in the file's own language, or a worked example
carrying every key, and states what a reader does with an unknown key.
A binary format contract declares its layout — each field's offset,
width, type and endianness — the magic number that identifies it, and
how a version is read before anything else is. Each is a public
contract with its own stability promise. The three contracts on file
are text formats and take the new kind token in their ids, which
`validate --fix` refiles and whose links it rewrites by id.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Two kinds, contracts renamed | Each schema demands the shape its files actually have; the id token stays truthful | Renames three contracts and every link into them |
| Two kinds, existing ids kept | No link churn | The id token names a kind that no longer exists, on the three contracts a reader meets first |
| One kind with an optional layout section | No rename at all | The kind demands neither shape properly, which is the fault this decision exists to fix |
| Defer the split | Keeps this feature about buildability | The buildability work rewrites all three contracts anyway, so the split costs least now |

## Consequences

- Positive: a binary format has a contract kind that fits it, and a
  text format's schema stops carrying rules meant for bytes.
- Negative: three contracts change id, so any reference outside the
  knowledge — a commit message, a pull request — reads the old name.
- Follow-ups: `schema-contract-text-format` and
  `schema-contract-binary-format` replace `schema-contract-file-format`,
  and contracts 005, 006 and 008 are refiled.
