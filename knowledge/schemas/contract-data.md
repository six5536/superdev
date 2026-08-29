---
type: Schema
id: schema-contract-data
title: Data Contract Schema
description: The persisted store — its schema, the constraints it holds, how it migrates, and the stability promise, in knowledge/contracts/public/.
---

# Data Contract Schema

Structural rules for one data contract, filed at
`knowledge/contracts/public/contract-{nnn}-data-{slug}.md`. The store the software owns and what anyone
reading or migrating it may rely on.

This is the storage shape, not the wire shape. An API's models are defined by
its own contract — [rest][sokf:schema-contract-rest], [rpc][sokf:schema-contract-rpc],
[graphql][sokf:schema-contract-graphql] — and the two differ on purpose: a field the API
never exposes still lives here, and a field the API composes from three columns
does not. A file the software exchanges rather than queries is a
[file-format contract][sokf:schema-contract-file-format].

````yaml
description: >
  One persisted store — the engine and who may reach it, the schema in its own
  definition language, the constraints callers rely on, how it changes under a
  running system, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    const: DataContract
  id:
    pattern: '^contract-\d{3}-data-[a-z0-9-]+$'
    description: >
      contract-{nnn}-data-{slug}, the slug naming which store. The
      number is the next free one across knowledge/contracts/, public and
      private together.
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Store"
    level: 1
    required: true
    content: prose
    description: >
      The engine and version, where it lives, which component owns writes, and
      who else may read it. A store no one outside the owning component reads
      is still a contract with the next release of that component.
  - heading: "Schema"
    level: 1
    required: true
    content: code
    description: >
      The definitions in the store's own language — SQL DDL, the collection
      documents, the key layout. One fenced block, tagged. Prose describes;
      this block defines.
  - heading: "Constraints"
    level: 1
    required: true
    content: bullet-list
    description: >
      What always holds: primary and unique keys, referential rules, what is
      nullable and what is not, retention and deletion, and any ordering or
      uniqueness a reader may depend on.
  - heading: "Migration"
    level: 1
    required: true
    content: prose
    description: >
      How the schema changes under a running system — expand-then-contract or
      downtime, whether old and new code read the same rows during a rollout,
      and how a migration is rolled back.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      Which tables, columns and semantics are promised to readers outside the
      owning component, and how a breaking change reaches them.

example: |
  ---
  type: DataContract
  id: contract-001-data-widget
  title: Data Contract
  description: The widget store — Postgres, owned by the API, read by reporting.
  status: stable
  ---

  # Store

  Postgres 16, one database per environment. The API service owns every write.
  Reporting connects with a read-only role and may read `widget` but never
  `widget_audit`, which carries user identifiers.

  # Schema

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

  # Constraints

  - `id` is a v7 uuid, so insertion order and creation order agree.
  - A name is unique per tenant, case-insensitively, among live rows only.
  - Deletion is soft: `deleted_at` is set and the row stays. Nothing outside a
    retention job issues `DELETE`.
  - The retention job removes soft-deleted rows ninety days after
    `deleted_at`, which is the point after which a restore is impossible.

  # Migration

  Expand then contract, always. A release adds a nullable column and
  backfills; the next reads it; a third makes it `NOT NULL` and drops what it
  replaced. Old and new code therefore read the same rows throughout a rolling
  deploy. A migration that cannot be split this way takes a maintenance window,
  and its rollback is a restore from the pre-migration snapshot.

  # Stability

  `widget` is read by reporting, so its columns are added, never removed or
  retyped, within a major version. `widget_audit` is private to the API and may
  change in any release. A column reporting depends on is deprecated in the
  release notes one release before it goes.
````

<!-- sokf:links -->
[sokf:schema-contract-file-format]: /knowledge/schemas/contract-file-format.md
[sokf:schema-contract-graphql]: /knowledge/schemas/contract-graphql.md
[sokf:schema-contract-rest]: /knowledge/schemas/contract-rest.md
[sokf:schema-contract-rpc]: /knowledge/schemas/contract-rpc.md
