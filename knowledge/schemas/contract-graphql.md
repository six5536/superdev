---
type: Schema
id: schema-contract-graphql
title: GraphQL Contract Schema
description: One GraphQL API — its SDL, endpoint, error and limit behaviour, and the stability promise, a public contract.
---

# GraphQL Contract Schema

Structural rules for one public GraphQL contract, filed at
`contract-{nnn}-graphql-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. One endpoint and one schema, so the sections
differ from a REST surface: there are no per-route status codes to tabulate,
and deprecation stands in for versioning.

Pick by protocol. A resource-shaped HTTP surface is a
[rest contract][sokf:schema-contract-rest]; a compiled service IDL is an
[rpc contract][sokf:schema-contract-rpc].

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
  One GraphQL API offered to callers — the schema in SDL, where it is served
  and how a caller authenticates, what a partial failure looks like, the limits
  a query must stay within, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: GraphqlContract
  id:
    required: true
    pattern: '^contract-\d{3}-graphql-[a-z0-9-]+$'
    description: >
      contract-{nnn}-graphql-{slug}, the slug naming which graph. The
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
  - heading-pattern: '^GraphQL contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Schema"
    level: 2
    required: true
    content: code
    description: >
      The surface in SDL — types, queries, mutations, subscriptions, and the
      `@deprecated` directives currently in force. One fenced `graphql` block.
      Every field a caller may select is defined here.
  - heading: "Endpoint and authentication"
    level: 2
    required: true
    content: prose
    description: >
      The URL, the methods accepted, whether persisted queries or introspection
      are available in production, what a caller presents to authenticate, and
      the response to a missing or rejected credential.
  - heading: "Errors"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      How failures reach the caller — the `errors` array beside partial `data`,
      the extension fields carrying a machine-readable code, and which
      conditions are transport-level failures instead.
  - heading: "Limits"
    level: 2
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Query depth, complexity budget, pagination caps and rate limits, and what
      a caller sees on exceeding one. Omit where the graph imposes none.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What may be added without notice, how a field is deprecated and how long
      it then survives, and the rare change that forces a second endpoint.

example: |
  ---
  type: GraphqlContract
  id: contract-001-graphql-widget
  title: GraphQL Contract
  description: The widget graph — one endpoint, deprecation in place of versioning.
  lifecycle: active
  ---

  # GraphQL contract: widget graph

  The widget graph: one endpoint, deprecation in place of versioning.

  ## Schema

  ```graphql
  type Widget {
    id: ID!
    name: String!
    owner: String @deprecated(reason: "use ownerAccount; removed after 2026-06")
    ownerAccount: Account
  }

  type Account {
    id: ID!
    displayName: String!
  }

  type Query {
    widget(id: ID!): Widget
    widgets(first: Int = 20, after: String): [Widget!]!
  }

  type Mutation {
    createWidget(name: String!): Widget!
  }
  ```

  ## Endpoint and authentication

  `POST /graphql`, JSON body. `GET` is accepted for persisted queries only.
  Introspection is on in every environment, because the schema is public.
  A bearer token in the `Authorization` header identifies the caller; an
  anonymous request may read `widgets` and may not mutate.

  ## Errors

  A resolver failure returns HTTP 200 with `data` partially populated and one
  entry per failure in `errors`, each carrying `extensions.code` — one of
  `BAD_USER_INPUT`, `UNAUTHENTICATED`, `FORBIDDEN`, `INTERNAL`. Only a
  malformed request body or a rejected token fails at the transport, as 400 or
  401. A caller MUST therefore read `errors` even on a 200.

  ## Limits

  Query depth MUST be capped at 10 and complexity at 1000 points, one point
  per field and 20 per list. `widgets` MUST return at most 100 per page.
  Exceeding any of these MUST be `BAD_USER_INPUT` with the budget in the
  extension, before execution starts.

  ## Stability

  Types and fields MAY be added without notice, and a caller MUST tolerate
  fields it did not ask for appearing in the schema. A field going away MUST be
  marked `@deprecated` with a date, stays for at least six months, and is
  announced in the release notes. Nothing else removes a field, and there is no
  `/graphql/v2`.
````

<!-- sokf:links -->
[sokf:schema-contract-rest]: /knowledge/schemas/contract-rest.md
[sokf:schema-contract-rpc]: /knowledge/schemas/contract-rpc.md
