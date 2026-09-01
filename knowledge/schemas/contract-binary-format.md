---
type: Schema
id: schema-contract-binary-format
title: Binary Format Contract Schema
description: One binary file others read or write — its magic number and version, the byte layout of every field, how a reader treats the unexpected, and the stability promise, a public contract.
---

# Binary Format Contract Schema

Structural rules for one public binary-format contract, filed at
`contract-{nnn}-binary-format-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. A format is a public contract when someone
outside this repository writes the file with their own tools or reads it
without this codebase — an archive, an index, a wire dump, a cache another
program mines.

A binary format is its own kind because its shape is a byte layout, not a
schema of keys: what binds is where each field sits, how wide it is, how it
is ordered, and what a reader does with the bytes it does not recognise. A
file whose shape is keys and values is a [text
format][sokf:schema-contract-text-format].

<!-- sokf:include contract-style -->
**Contract style — a contract defines its interface** (superdev
ADR-033, ADR-036):

- A contract MUST define every element a caller depends on in the
  structured form this schema declares, so a caller reproduces the
  interface from the contract alone.
- Prose MUST describe and MUST NOT define. Each normative statement
  outside the definition form MUST use an RFC 2119 modal verb, one
  requirement per sentence.
- A contract MUST bind what it names and MUST NOT state how the
  interface is built inside.
- The project MUST bind this contract to its implementation, by
  generating the surface from it or by a test where the implementation
  is hand-written; a committed generated artifact MUST be proved
  current.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.
<!-- /sokf:include -->

````yaml
description: >
  One binary format offered to others — where the files live and who writes
  them, the magic number and version a reader checks first, the byte layout
  of every field, how a reader treats what it does not recognise, and what is
  promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: BinaryFormatContract
  id:
    required: true
    pattern: '^contract-\d{3}-binary-format-[a-z0-9-]+$'
    description: >
      contract-{nnn}-binary-format-{slug}, the slug naming the format. The
      number is the next free one across every contract, public and internal
      together and every lifecycle folder — a duplicate is an error.
  title:
    required: true
  description:
    required: true
  lifecycle:
    enum: [active, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^Binary format contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: which files this binds and who reads or writes them —
      link the ADRs behind it.
  - heading: "Files"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Where the files live, who writes them, and what a reader may assume
      about a file it did not write.
  - heading: "Identification"
    level: 2
    required: true
    content: table
    columns: [Offset, Width, Value, Meaning]
    description: >
      The magic number and the version field, with the offset and width of
      each, so a reader identifies the format and its version before it reads
      anything else. A reader that cannot identify a file must be able to stop
      here.
  - heading: "Layout"
    level: 2
    required: true
    content: table
    columns: [Offset, Width, Type, Endianness, Field]
    description: >
      Every field the format carries, in file order: where it starts, how wide
      it is, how its bytes are read, and what it means. A variable-length
      field states what fixes its length. A writer produces a valid file from
      this table alone; prose around it describes and never defines.
  - heading: "Compatibility"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What a reader does with a version it does not know, a field it does not
      recognise, a length that overruns the file, and a truncated file. Say
      what is refused and what is tolerated.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Which fields and offsets are promised, over what version range, and how
      a layout change is signalled to readers that predate it.

example: |
  ---
  type: BinaryFormatContract
  id: contract-001-binary-format-widget-index
  title: Binary Format Contract
  description: The widget search index — a magic number, a version, and a fixed-width posting list.
  lifecycle: active
  ---

  # Binary format contract: widget index

  The on-disk search index `widget` builds and reads. A profiler or a repair
  tool reads it without this codebase, so the layout below binds.

  ## Files

  One file, `.widget/index.bin`, written whole by `widget index` and never
  appended to. A reader MUST treat the file as immutable once written, and
  MUST NOT assume a partially written file is readable.

  ## Identification

  | Offset | Width | Value | Meaning |
  |--------|-------|-------|---------|
  | 0 | 4 | `WIDX` | Magic number; a reader MUST refuse a file without it. |
  | 4 | 2 | `0x0002` | Format version, little-endian. |

  ## Layout

  | Offset | Width | Type | Endianness | Field |
  |--------|-------|------|------------|-------|
  | 0 | 4 | bytes | — | magic, `WIDX` |
  | 4 | 2 | u16 | little | version |
  | 6 | 2 | u16 | little | reserved, written as zero |
  | 8 | 8 | u64 | little | posting count |
  | 16 | 16 × count | record | little | postings: u64 widget id, u32 field, u32 score |

  ## Compatibility

  A reader MUST refuse a version above the one it knows, naming the version it
  found. It MUST ignore the reserved field rather than requiring zero, so a
  later release MAY claim it. A posting count that overruns the file MUST be a
  refusal, never a truncated read.

  ## Stability

  The magic number and the first sixteen bytes are stable from 1.0: within a
  major version their offsets and widths MUST NOT change. A record MAY gain
  fields only by a version bump, and both versions MUST stay readable for one
  release.
````

<!-- sokf:links -->
[sokf:schema-contract-text-format]: /knowledge/schemas/contract-text-format.md
