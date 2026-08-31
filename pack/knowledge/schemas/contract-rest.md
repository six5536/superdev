---
type: Schema
id: schema-contract-rest
title: REST Contract Schema
description: One HTTP API — its endpoints in TypeSpec, the authentication, the error responses and the stability promise, a public contract.
---

# REST Contract Schema

Structural rules for one public HTTP contract, filed at
`contract-{nnn}-rest-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. The endpoints are defined in TypeSpec, which is
the language-neutral form an HTTP surface has no native one for; prose around
it describes and never defines.

````yaml
description: >
  One HTTP API offered to callers — the endpoints in TypeSpec, how a caller
  authenticates, the error responses, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    const: RestContract
  id:
    pattern: '^contract-\d{3}-rest-[a-z0-9-]+$'
    description: >
      contract-{nnn}-rest-{slug}, the slug naming which HTTP API. The
      number is the next free one across every contract, public and
      private together and every lifecycle folder — a duplicate is
      an error.
  lifecycle:
    enum: [active, deprecated]

sections-ordered: true
sections:
  - heading: "Endpoints"
    level: 1
    required: true
    content: code
    description: >
      The surface in TypeSpec — routes, request and response models, status
      codes. One fenced `typespec` block. Every field a caller may read or send
      appears here, not in the prose.
  - heading: "Authentication"
    level: 1
    required: true
    content: prose
    description: >
      What a caller presents, where it goes, how it is obtained and how it
      expires. The response to a missing or rejected credential.
  - heading: "Errors"
    level: 1
    required: true
    content: table
    columns: [Status, Condition]
    description: >
      The statuses a caller must handle and what provokes each. The error body
      shape belongs in the TypeSpec above.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      How the API is versioned, what may be added within a version, what
      forces a new one, and the deprecation window callers get.

example: |
  ---
  type: RestContract
  id: contract-001-rest-widget
  title: HTTP API Contract
  description: The widget HTTP API — read and create widgets, versioned in the path.
  lifecycle: active
  ---

  # Endpoints

  ```typespec
  @service(#{ title: "Widgets" })
  namespace Widgets;

  model Widget {
    id: string;
    name: string;
    createdAt: utcDateTime;
  }

  model Error {
    code: string;
    message: string;
  }

  @route("/v1/widgets")
  interface WidgetsApi {
    @get list(@query limit?: int32): Widget[] | Error;
    @post create(@body widget: Widget): Widget | Error;
  }
  ```

  # Authentication

  A bearer token in the `Authorization` header, issued by the tenant's identity
  provider and valid for one hour. A missing or expired token is `401`; a token
  valid but scoped to another tenant is `404`, never `403`, so the API never
  confirms that an unreachable widget exists.

  # Errors

  | Status | Condition |
  |--------|-----------|
  | 400    | the body fails the model above |
  | 401    | no token, or an expired one |
  | 404    | no such widget, or one outside the caller's tenant |
  | 429    | the caller is over its rate limit; `Retry-After` is set |

  # Stability

  The version sits in the path. Within `/v1` fields are added to responses and
  optional fields to requests, never removed or retyped; a caller must ignore
  fields it does not know. Anything else opens `/v2`, and `/v1` then runs for
  six further months with a `Deprecation` header on every response.
````
