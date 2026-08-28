---
type: Schema
id: schema-contract-interface
title: Interface Contract Schema
description: The interfaces build codes against — data model, module boundaries, key flows — filed in knowledge/contracts/private/.
---

# Interface Contract Schema

Structural rules for interface contracts filed at
`knowledge/contracts/private/contract-{nnn}-interface-{feature-slug}.md`. Build codes against it;
the decisions behind it are recorded as ADRs.

````yaml
description: >
  The interfaces build codes against — data model and API, module
  boundaries, key flows, and cross-cutting concerns — each expressed in
  its native language, or TypeSpec.
line-limit: 800

frontmatter:
  type:
    const: InterfaceContract
  id:
    pattern: '^contract-\d{3}-interface-[a-z0-9-]+$'
    description: >
      contract-{nnn}-interface-{slug}, the slug naming which feature. The
      number is the next free one across knowledge/contracts/, public and
      private together.
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^Interface contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the interfaces this feature adds or changes, and the
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
      language (sql, rust, ts, typespec, …).
  - heading: "Module boundaries"
    level: 2
    required: true
    content: bullet-list
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
    description: >
      Bullets for Security (authn/authz, data exposure, input
      validation), Performance (expected load, hot paths, limits),
      Migration/rollout (how we get from the current state to this, and
      how we roll back), and Observability (what is logged or measured
      to know it works).

example: |
  # Interface contract: pack source allowlist

  Adds a transport check to pack source parsing; decisions in ADR-012.

  ## Data model & API

  ```rust
  pub enum Transport { Https, Ssh, File }

  pub fn parse_source(raw: &str) -> Result<Source, SourceError>;
  ```

  ## Module boundaries

  - pack::resolve owns transport validation; the CLI depends on pack,
    never the reverse.

  ## Key flows

  - sync: manifest → parse_source → fetch over an allowed transport.

  ## Cross-cutting concerns

  - Security: refuse transports outside the allowlist at parse time.
  - Performance: validation is per-source and constant-time.
  - Migration/rollout: existing https/ssh/file manifests unchanged.
  - Observability: the refusal error names the offending source.
````
