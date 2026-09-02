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
  One HTTP API offered to callers — the endpoints in TypeSpec, how a caller
  authenticates, the error responses, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: RestContract
  id:
    required: true
    pattern: '^contract-\d{3}-rest-[a-z0-9-]+$'
    description: >
      contract-{nnn}-rest-{slug}, the slug naming which HTTP API. The
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
  - heading-pattern: '^REST contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Endpoints"
    level: 2
    required: true
    content: code
    description: >
      The surface in TypeSpec — routes, request and response models, status
      codes. One fenced `typespec` block. Every field a caller may read or send
      appears here, not in the prose.
  - heading: "Authentication"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What a caller presents, where it goes, how it is obtained and how it
      expires. The response to a missing or rejected credential.
  - heading: "Errors"
    level: 2
    required: true
    content: table
    columns: [Status, Condition]
    description: >
      The statuses a caller must handle and what provokes each. The error body
      shape belongs in the TypeSpec above.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
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

  # REST contract: widget API

  The widget HTTP API: read and create widgets, versioned in the path.

  ## Endpoints

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

  ## Authentication

  A bearer token in the `Authorization` header, issued by the tenant's
  identity provider and valid for one hour. A missing or expired token MUST be
  `401`; a token valid but scoped to another tenant MUST be `404`, never
  `403`, so the API never confirms that an unreachable widget exists.

  ## Errors

  | Status | Condition |
  |--------|-----------|
  | 400    | the body fails the model above |
  | 401    | no token, or an expired one |
  | 404    | no such widget, or one outside the caller's tenant |
  | 429    | the caller is over its rate limit; `Retry-After` is set |

  ## Stability

  The version sits in the path. Within `/v1` fields MAY be added to responses
  and optional fields to requests, and MUST NOT be removed or retyped; a
  caller MUST ignore fields it does not know. Anything else opens `/v2`, and
  `/v1` then runs for six further months with a `Deprecation` header on every
  response.
````
