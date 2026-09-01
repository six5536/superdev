---
type: Schema
id: schema-contract-telemetry
title: Telemetry Contract Schema
description: The signal operators build on — the metrics, the log shape, the traces, and the stability promise, a public contract.
---

# Telemetry Contract Schema

Structural rules for one public telemetry contract, filed at
`contract-{nnn}-telemetry-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. Metric names, label sets and log fields are a
promised surface: a dashboard or an alert binds to them, and renaming one
breaks a consumer who never called an API.

Not to be confused with the [event contract][sokf:schema-contract-events]. Events are
published as a product feature and consumed as data; this is operational
signal, consumed by whoever runs the software. The error taxonomy itself —
which failures exist and what each means — belongs to the error-handling
concept, and is referenced here rather than restated.

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
  One telemetry surface — the metrics with their types and labels, the shape
  every log line carries, the traces emitted, and what is promised stable to
  the dashboards and alerts built on them.
line-limit: 400

frontmatter:
  type:
    required: true
    const: TelemetryContract
  id:
    required: true
    pattern: '^contract-\d{3}-telemetry-[a-z0-9-]+$'
    description: >
      contract-{nnn}-telemetry-{slug}, the slug naming which emitting component. The
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
  - heading-pattern: '^Telemetry contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Metrics"
    level: 2
    required: true
    content: table
    columns: [Name, Type, Labels, Meaning]
    description: >
      Every metric an operator may build on. Name the labels exactly and say
      which are bounded — an unbounded label is a cardinality incident, and the
      table is where that is caught.
  - heading: "Logs"
    level: 2
    required: true
    content: prose
    description: >
      The format, the fields every line carries, what the levels mean and when
      each is used, and what the software promises never to log. A field a
      query depends on is as much a contract as a metric name.
  - heading: "Traces"
    level: 2
    content: prose
    description: >
      The spans emitted, how context is propagated in and out, and the sampling
      a consumer should expect. Omit where the software emits none.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Which names, labels and fields are promised, how a rename is carried
      through so dashboards keep working, and how a removal is announced.

example: |
  ---
  type: TelemetryContract
  id: contract-001-telemetry-widget-api
  title: Telemetry Contract
  description: What the widget API emits — three metrics, structured logs, OTLP traces.
  lifecycle: active
  ---

  # Telemetry contract: widget API

  What the widget API emits: three metrics, structured logs, OTLP
  traces.

  ## Metrics

  | Name | Type | Labels | Meaning |
  |------|------|--------|---------|
  | `widget_requests_total` | counter | `route`, `method`, `status` | requests served; `route` is the template, never the filled path |
  | `widget_request_seconds` | histogram | `route`, `method` | request duration; buckets 5ms to 10s |
  | `widget_store_pool_in_use` | gauge | none | database connections checked out |

  Every label is bounded: `route` takes one of the templates in the API
  contract, `method` an HTTP verb, `status` a three-digit code. No label
  carries a widget id, a tenant id or anything else unbounded.

  ## Logs

  One JSON object per line on stdout. Every line carries `ts` (RFC 3339),
  `level`, `msg`, `service`, `version` and, inside a request, `trace_id` and
  `route`. `error` and `warn` mean an operator should look; `info` marks
  lifecycle events only — start, ready, shutdown — so a healthy service is
  quiet. Request bodies, tokens and anything from the `Authorization` header
  are never logged at any level.

  ## Traces

  OTLP over gRPC to the collector named by configuration. One server span per
  request, named for the route template, with a child span per database query.
  Incoming `traceparent` is honoured and propagated outward. Sampling is
  head-based at 1% by default, and every trace carrying an error is kept.

  ## Stability

  The metric names, their label sets and the log fields above MUST stay stable
  within a major version. A renamed metric MUST be emitted under both names for
  one minor release, so a dashboard can be moved without a gap. A removed
  metric or log field MUST be announced in the release notes one release
  ahead.
````

<!-- sokf:links -->
[sokf:schema-contract-events]: /knowledge/schemas/contract-events.md
