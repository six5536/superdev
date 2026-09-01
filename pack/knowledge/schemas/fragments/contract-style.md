---
type: Fragment
id: contract-style
title: Contract Style
description: The definition standard every contract document follows, included into each contract-kind schema.
---

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
