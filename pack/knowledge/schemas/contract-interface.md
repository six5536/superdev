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
ADR-033, ADR-042, ADR-043, ADR-044):

- A contract's Definition MUST be one or more source includes of the
  regions that declare the interface, and MUST NOT carry an authored
  block; a caller reads the interface from the contract and reproduces
  it from the source the contract carries.
- A region MUST be bounded by `sokf:begin <name>` and `sokf:end <name>`
  in the source's own comment syntax. What is not marked is not
  promised.
- A doc comment inside an included region is contract text: a MUST
  there binds as a MUST in Behaviour does.
- Prose MUST describe and MUST NOT define. Behaviour MUST carry what no
  single element can say and what no include reaches — stability,
  consumers, behaviour across elements, exit codes, error semantics —
  each normative statement with an RFC 2119 modal verb, one requirement
  per sentence.
- Behaviour MUST cover what the schema's checklist names for the
  contract's kind, one `###` per item that applies.
- A contract MUST bind what it names and MUST NOT state how the
  interface is built inside.
- The Definition is bound by its include. The project MUST bind each
  Behaviour promise by a test of the behaviour it promises.
- A built-from source unreadable as a surface MUST be rendered by a
  generator that writes `sokf:generated-by <what>` in the rendering's
  leading lines, and the rendering MUST be proved current by a test.
- A Behaviour or Stability statement whose behaviour is unbuilt MAY
  carry `PENDING` in uppercase beside its modal verb, naming the issue
  or plan slice in parentheses, and MUST NOT once the feature settles; a
  definition element carries none.
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
