---
type: Schema
id: schema-contract-data
title: Data Contract Schema
description: The persisted store — its schema, the constraints it holds, how it migrates, and the stability promise, a public contract.
---

# Data Contract Schema

Structural rules for one data contract, filed at
`contract-{nnn}-data-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. The store the software owns and what anyone
reading or migrating it may rely on.

This is the storage shape, not the wire shape. An API's models are defined by
its own contract — [rest][sokf:schema-contract-rest], [rpc][sokf:schema-contract-rpc],
[graphql][sokf:schema-contract-graphql] — and the two differ on purpose: a field the API
never exposes still lives here, and a field the API composes from three columns
does not. A file the software exchanges rather than queries is a
[file-format contract][sokf:schema-contract-text-format].

<!-- sokf:include contract-style -->
**Contract style — a contract is a binding surface, not a
specification** (superdev ADR-029):

- Each normative statement MUST use an RFC 2119 modal verb, one
  requirement per sentence.
- An enumerable surface — commands, flags, keys, types, error cases,
  limits — MUST be defined in the kind's native structured form: a code
  block, table or list. Prose, doc comments included, describes and
  MUST NOT define.
- A contract MUST bind only what callers rely on; behaviour a contract
  does not list is the code's to decide.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.
<!-- /sokf:include -->

````yaml
description: >
  One persisted store — the engine and who may reach it, the schema in its own
  definition language, the constraints callers rely on, how it changes under a
  running system, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: DataContract
  id:
    required: true
    pattern: '^contract-\d{3}-data-[a-z0-9-]+$'
    description: >
      contract-{nnn}-data-{slug}, the slug naming which store. The
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
  - heading-pattern: '^Data contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Store"
    level: 2
    required: true
    content: prose
    description: >
      The engine and version, where it lives, which component owns writes, and
      who else may read it. A store no one outside the owning component reads
      is still a contract with the next release of that component.
  - heading: "Schema"
    level: 2
    required: true
    content: code
    description: >
      The definitions in the store's own language — SQL DDL, the collection
      documents, the key layout. One fenced block, tagged. Every table,
      column, type, index and constraint a reader may meet appears, so a
      migration is written from this block alone; prose describes and never
      defines. The project binds this block to the store — by generating one
      from the other, or by a test (ADR-036).
  - heading: "Constraints"
    level: 2
    required: true
    content: bullet-list
    item-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What always holds: primary and unique keys, referential rules, what is
      nullable and what is not, retention and deletion, and any ordering or
      uniqueness a reader may depend on.
  - heading: "Migration"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      How the schema changes under a running system — expand-then-contract or
      downtime, whether old and new code read the same rows during a rollout,
      and how a migration is rolled back.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Which tables, columns and semantics are promised to readers outside the
      owning component, and how a breaking change reaches them.

example: |
  ---
  type: DataContract
  id: contract-001-data-widget
  title: Data Contract
  description: The widget store — Postgres, owned by the API, read by reporting.
  lifecycle: active
  ---

  # Data contract: widget store

  The widget store: Postgres, owned by the API service and read by
  reporting.

  ## Store

  Postgres 16, one database per environment. The API service owns every write.
  Reporting connects with a read-only role and may read `widget` but never
  `widget_audit`, which carries user identifiers.

  ## Schema

  ```sql
  CREATE TABLE widget (
      id          uuid PRIMARY KEY,
      name        text NOT NULL,
      tenant_id   uuid NOT NULL,
      created_at  timestamptz NOT NULL DEFAULT now(),
      deleted_at  timestamptz
  );

  CREATE UNIQUE INDEX widget_name_per_tenant
      ON widget (tenant_id, lower(name)) WHERE deleted_at IS NULL;
  ```

  ## Constraints

  - `id` MUST be a v7 uuid, so insertion order and creation order agree.
  - A name MUST be unique per tenant, case-insensitively, among live rows
    only.
  - Deletion MUST be soft: `deleted_at` is set and the row stays, and only a
    retention job issues `DELETE`.
  - The retention job MUST remove soft-deleted rows ninety days after
    `deleted_at`, the point after which a restore is impossible.

  ## Migration

  Expand then contract, always: a migration MUST be split so old and new code
  read the same rows throughout a rolling deploy. A release adds a nullable
  column and backfills; the next reads it; a third makes it `NOT NULL` and
  drops what it replaced. A migration that cannot be split this way takes a
  maintenance window, and its rollback is a restore from the pre-migration
  snapshot.

  ## Stability

  `widget` is read by reporting, so within a major version its columns MAY be
  added and MUST NOT be removed or retyped. `widget_audit` is private to the
  API and MAY change in any release. A column reporting depends on MUST be
  deprecated in the release notes one release before it goes.
````

<!-- sokf:links -->
[sokf:schema-contract-graphql]: /knowledge/schemas/contract-graphql.md
[sokf:schema-contract-rest]: /knowledge/schemas/contract-rest.md
[sokf:schema-contract-rpc]: /knowledge/schemas/contract-rpc.md
[sokf:schema-contract-text-format]: /knowledge/schemas/contract-text-format.md
