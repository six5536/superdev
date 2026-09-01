---
type: Schema
id: schema-contract-rpc
title: RPC Contract Schema
description: One RPC service — its IDL, transport, authentication, error codes and stability promise, a public contract.
---

# RPC Contract Schema

Structural rules for one public RPC contract, filed at
`contract-{nnn}-rpc-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. gRPC, Thrift, Cap'n Proto or JSON-RPC: the
service is defined in the IDL its own toolchain compiles, and the wire
compatibility rules are the IDL's, not HTTP's.

Pick by protocol. A resource-shaped HTTP surface is a
[rest contract][sokf:schema-contract-rest]; a single-endpoint schema with client-chosen
selections is a [graphql contract][sokf:schema-contract-graphql]; a stream a consumer
subscribes to rather than calls is an [event contract][sokf:schema-contract-events].

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
  One RPC service offered to callers — the IDL that defines it, how a client
  connects, how it authenticates, the status codes it must handle, and what is
  promised stable on the wire.
line-limit: 400

frontmatter:
  type:
    required: true
    const: RpcContract
  id:
    required: true
    pattern: '^contract-\d{3}-rpc-[a-z0-9-]+$'
    description: >
      contract-{nnn}-rpc-{slug}, the slug naming which RPC service. The
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
  - heading-pattern: '^RPC contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Services"
    level: 2
    required: true
    content: code
    description: >
      The service and message definitions in their IDL — protobuf, Thrift, a
      JSON-RPC schema. One fenced block, tagged. Every service, method,
      message and field appears, with its field numbers and reserved tags, so
      a client is generated from this block alone. The project binds it to the
      server — by generating one from the other, or by a test (ADR-036).
  - heading: "Transport"
    level: 2
    required: true
    content: prose
    description: >
      Where a client connects, over what — TLS, ALPN, plaintext for local use —
      the deadline and message-size limits it must respect, and which methods
      stream in which direction.
  - heading: "Authentication"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What the client presents and on which metadata key, how it is obtained
      and how it expires, and the status returned for a missing or rejected
      credential.
  - heading: "Errors"
    level: 2
    required: true
    content: table
    columns: [Code, Condition]
    description: >
      The status codes a caller must handle and what provokes each, including
      which are safe to retry. Error detail payloads are defined in the IDL
      above.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      The wire compatibility rules — field numbers never reused, what may be
      added to a message, what forces a new service version — and the
      deprecation window callers get.

example: |
  ---
  type: RpcContract
  id: contract-001-rpc-widget
  title: RPC Contract
  description: The widget gRPC service — get and watch, over mTLS.
  lifecycle: active
  ---

  # RPC contract: widget

  The widget gRPC service: get and watch, over mTLS.

  ## Services

  ```protobuf
  syntax = "proto3";
  package widgets.v1;

  message Widget {
    string id = 1;
    string name = 2;
    reserved 3;              // was owner_email, removed in 1.4
  }

  message GetRequest { string id = 1; }
  message WatchRequest { string tenant_id = 1; }

  service Widgets {
    rpc Get(GetRequest) returns (Widget);
    rpc Watch(WatchRequest) returns (stream Widget);
  }
  ```

  ## Transport

  gRPC over HTTP/2 on port 8443, TLS required, ALPN `h2`. The default deadline
  is five seconds and the server rejects a call arriving without one. Messages
  are capped at 4 MiB. `Watch` streams from server to client and stays open
  until the client cancels or the deadline passes.

  ## Authentication

  A bearer token on the `authorization` metadata key, issued by the tenant's
  identity provider and valid for one hour. A missing or expired token MUST be
  `UNAUTHENTICATED`; a token scoped to another tenant MUST be `NOT_FOUND`, so
  the service never confirms that an unreachable widget exists.

  ## Errors

  | Code | Condition |
  |------|-----------|
  | `INVALID_ARGUMENT` | the request fails the message definition above |
  | `NOT_FOUND` | no such widget, or one outside the caller's tenant |
  | `UNAUTHENTICATED` | no token, or an expired one |
  | `RESOURCE_EXHAUSTED` | the caller is over its rate limit; safe to retry with backoff |
  | `UNAVAILABLE` | the server is restarting; safe to retry |

  ## Stability

  Field numbers MUST NOT be reused: a removed field is `reserved`, as `3` is
  above. Fields MAY be added to a message and MUST NOT be retyped, and a
  client MUST ignore fields it does not know. A change that breaks either rule
  opens `widgets.v2`, and `widgets.v1` then serves for six further months.
````

<!-- sokf:links -->
[sokf:schema-contract-events]: /knowledge/schemas/contract-events.md
[sokf:schema-contract-graphql]: /knowledge/schemas/contract-graphql.md
[sokf:schema-contract-rest]: /knowledge/schemas/contract-rest.md
