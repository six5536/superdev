---
type: Schema
id: schema-contract-interface
title: Interface Contract Schema
description: The interfaces build codes against — data model, module boundaries, key flows — an internal contract, durable and keyed to the interface.
---

# Interface Contract Schema

Structural rules for interface contracts, filed as
`contract-{nnn}-interface-{interface-slug}`, an internal contract placed in
its lifecycle folder by `superdev validate --fix`. It is durable and keyed
to the interface it describes — a module boundary, never a feature — and
CONTRACT-DESIGN updates it as features change that interface. Build codes
against it; the decisions behind it are recorded as ADRs.

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
  The interfaces build codes against — data model and API, module
  boundaries, key flows, and cross-cutting concerns — each expressed in
  its native language, or TypeSpec.
line-limit: 800

frontmatter:
  type:
    required: true
    const: InterfaceContract
  id:
    required: true
    pattern: '^contract-\d{3}-interface-[a-z0-9-]+$'
    description: >
      contract-{nnn}-interface-{slug}, the slug naming the interface. The
      number is the next free one across every contract, public and
      internal together and every lifecycle folder — a duplicate is
      an error.
  title:
    required: true
  description:
    required: true
  lifecycle:
    enum: [active, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^Interface contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the interfaces this contract binds, and the
      decisions behind them — link the ADRs.
  - heading: "Data model & API"
    level: 2
    required: true
    content: code
    description: >
      Each contract in its native language — the language the code will
      enforce: SQL DDL for the schema, the host language's types, traits
      or interfaces for module APIs, the framework's route definitions
      for endpoints. A contract with no native form — a language-neutral
      HTTP API — is written in TypeSpec. Prose describes; it never
      defines. One fenced code block per contract, tagged with its
      language (sql, rust, ts, typespec, …). Every exported item the
      boundary binds appears with its full signature, so the next module
      is written against this block alone; what the block does not name,
      the contract does not bind. The project binds these blocks to the
      code — by generating one from the other, or by a test that fails
      when a declared item is absent (ADR-036).
  - heading: "Module boundaries"
    level: 2
    required: true
    content: bullet-list
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Which module owns what, and the direction of dependency. One line
      per boundary.
  - heading: "Key flows"
    level: 2
    required: true
    content: bullet-list
    description: >
      The 1-3 most important scenarios across the interfaces, end to
      end.
  - heading: "Cross-cutting concerns"
    level: 2
    required: true
    content: bullet-list
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Bullets for Security (authn/authz, data exposure, input
      validation), Performance (expected load, hot paths, limits),
      Migration/rollout (how we get from the current state to this, and
      how we roll back), and Observability (what is logged or measured
      to know it works).

example: |
  ---
  type: InterfaceContract
  id: contract-001-interface-pack-source-allowlist
  title: Pack source allowlist interface
  description: The transport check pack source parsing enforces.
  lifecycle: active
  ---

  # Interface contract: pack source allowlist

  Adds a transport check to pack source parsing; decisions in ADR-012.

  ## Data model & API

  ```rust
  pub enum Transport { Https, Ssh, File }

  pub fn parse_source(raw: &str) -> Result<Source, SourceError>;
  ```

  ## Module boundaries

  - pack::resolve owns transport validation; the CLI depends on pack, and
    pack MUST NOT depend on the CLI.

  ## Key flows

  - sync: manifest → parse_source → fetch over an allowed transport.

  ## Cross-cutting concerns

  - Security: the parser MUST refuse a transport outside the allowlist.
  - Performance: validation is per-source and constant-time.
  - Migration/rollout: existing https/ssh/file manifests unchanged.
  - Observability: the refusal error names the offending source.
````
