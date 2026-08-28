---
type: Schema
id: schema-contract-authz
title: Authorisation Contract Schema
description: What a caller may do — the principals, the role and scope vocabulary, the permissions and the boundaries every surface enforces, in knowledge/contracts/public/.
---

# Authorisation Contract Schema

Structural rules for one public authorisation contract, filed at
`knowledge/contracts/public/contract-{nnn}-authz-{slug}.md`. The role and scope vocabulary, what each may do,
and the boundary every surface enforces.

This is the one contract no surface can own. Each API contract's
Authentication section says what a caller presents and how identity is
established — who you are. This document says what that identity may then do,
which is the same answer whether the caller arrived over
[rest](contract-rest.md), [rpc](contract-rpc.md),
[graphql](contract-graphql.md), [mcp](contract-mcp.md) or the
[cli](contract-cli.md), and a rebuild cannot infer it from any one of them.
Row-level rules the store itself enforces are stated here and implemented in
the [data contract](contract-data.md); the vulnerability policy and the
guarantees the design makes are the security-requirements concept, not a
contract.

````yaml
description: >
  What a caller may do — who can act, the roles and scopes they hold, the
  permission each action needs, the tenancy or ownership boundary every
  surface enforces, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    const: AuthzContract
  id:
    pattern: '^contract-\d{3}-authz-[a-z0-9-]+$'
    description: >
      contract-{nnn}-authz-{slug}, the slug naming which authorisation model. The
      number is the next free one across knowledge/contracts/, public and
      private together.
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Principals"
    level: 1
    required: true
    content: prose
    description: >
      Who or what can act — an end user, a service account, an unauthenticated
      caller — and where each one's identity comes from. Name the surface
      contract that establishes it rather than restating the authentication.
  - heading: "Roles and scopes"
    level: 1
    required: true
    content: table
    columns: [Name, Kind, Meaning]
    description: >
      The whole vocabulary, one row each. A name that appears in a token, a
      policy or a database column and not in this table is a name a rebuild
      will get wrong.
  - heading: "Permissions"
    level: 1
    required: true
    content: table
    columns: [Action, Resource, Requires]
    description: >
      What each action needs, in terms of the vocabulary above. One row per
      action a caller can attempt, including the ones every caller may take.
  - heading: "Boundaries"
    level: 1
    required: true
    content: prose
    description: >
      The tenancy or ownership rule that cuts across every permission, where it
      is enforced, and what a refused caller is told — a refusal that confirms
      the resource exists is a disclosure, and the choice belongs here rather
      than in each surface.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      Which names are promised, what happens to a token carrying a scope that
      no longer exists, and how a permission is tightened without locking out
      callers mid-release.

example: |
  ---
  type: AuthzContract
  id: contract-001-authz-widget
  title: Authorisation Contract
  description: Who may do what to a widget — three roles, one tenancy boundary.
  status: stable
  ---

  # Principals

  Three. An **end user**, identified by the bearer token the HTTP API contract
  describes, always belonging to exactly one tenant. A **service account**,
  identified by the same token type but carrying no tenant, used by reporting
  and the migration job. An **anonymous caller**, holding no token, which
  reaches only the health endpoints.

  # Roles and scopes

  | Name | Kind | Meaning |
  |------|------|---------|
  | `viewer` | role | may read widgets in its own tenant |
  | `editor` | role | `viewer`, plus create, rename and delete |
  | `owner` | role | `editor`, plus manage members and tokens |
  | `widgets:read` | scope | narrows a token to reads, whatever the role |
  | `widgets:write` | scope | permits writes, if the role allows them |

  A role is what a person holds; a scope is what a token was issued for. Both
  must permit an action for it to succeed, so a token scoped `widgets:read`
  cannot write even when its holder is an `owner`.

  # Permissions

  | Action | Resource | Requires |
  |--------|----------|----------|
  | list, get | widget | `viewer` and `widgets:read` |
  | create, rename | widget | `editor` and `widgets:write` |
  | delete | widget | `editor` and `widgets:write` |
  | invite, revoke | member | `owner` and `widgets:write` |
  | read all tenants | widget | the reporting service account |
  | health check | none | any caller, including anonymous |

  # Boundaries

  Every widget belongs to one tenant, and a principal reaches only its own —
  the rule holds above every row in the table. It is enforced twice: the API
  narrows every query by the caller's tenant, and the store carries a
  row-level policy on `tenant_id` so a query that forgets returns nothing
  rather than everything.

  A caller reaching another tenant's widget is told `404`, never `403`, so the
  refusal does not confirm the widget exists. A caller inside its own tenant
  attempting an action above its role is told `403`, because there the
  existence is not a secret and the message can name the role required.

  # Stability

  Role and scope names are stable within a major version. A token carrying a
  scope that no longer exists is rejected outright rather than downgraded,
  because a silently narrowed token fails later and further away. A permission
  is tightened in two steps: one release logs what the new rule would refuse,
  the next enforces it.
````
