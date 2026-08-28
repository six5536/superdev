---
type: Schema
id: schema-contract-events
title: Event Contract Schema
description: One published message or event stream — its transport, payloads, delivery guarantees and stability promise, in knowledge/contracts/public/.
---

# Event Contract Schema

Structural rules for one public message contract, filed at
`knowledge/contracts/public/contract-{nnn}-events-{slug}.md`. Covers anything a consumer subscribes to rather
than calls — a queue, a topic, a webhook, a change feed.

````yaml
description: >
  One stream of messages published for others to consume — where it is
  published, the payloads in their own schema language, what the consumer may
  assume about delivery, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    const: EventContract
  id:
    pattern: '^contract-\d{3}-events-[a-z0-9-]+$'
    description: >
      contract-{nnn}-events-{slug}, the slug naming which message stream. The
      number is the next free one across knowledge/contracts/, public and
      private together.
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Transport"
    level: 1
    required: true
    content: prose
    description: >
      Where the messages appear — broker, topic, queue or webhook endpoint —
      how a consumer subscribes, and how it authenticates.
  - heading: "Messages"
    level: 1
    required: true
    content: code
    description: >
      The payloads in their own schema language — JSON Schema, Protobuf, Avro
      or TypeSpec. One fenced block, tagged. Every field a consumer may read is
      defined here, not described in the prose.
  - heading: "Ordering and delivery"
    level: 1
    required: true
    content: bullet-list
    description: >
      What the consumer may assume: ordering guarantees, at-least-once against
      exactly-once, the retry and dead-letter behaviour, and which field makes
      a message idempotent.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      How payloads are versioned, what may be added without a new version, and
      how a consumer is told a message type is going away.

example: |
  ---
  type: EventContract
  id: contract-001-events-widget-lifecycle
  title: Event Contract
  description: The widget lifecycle stream on Kafka — created and deleted events.
  status: draft
  ---

  # Transport

  Kafka, topic `widgets.lifecycle`, six partitions keyed by widget id.
  Consumers authenticate with SASL/SCRAM against their own principal; a
  principal may read the topic and never write it.

  # Messages

  ```json
  {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "WidgetEvent",
    "type": "object",
    "required": ["eventId", "type", "widgetId", "occurredAt"],
    "properties": {
      "eventId": { "type": "string", "format": "uuid" },
      "type": { "enum": ["widget.created", "widget.deleted"] },
      "widgetId": { "type": "string" },
      "occurredAt": { "type": "string", "format": "date-time" }
    }
  }
  ```

  # Ordering and delivery

  - Ordered per widget id, because the partition key is the widget id. There is
    no ordering across widgets.
  - At-least-once. A consumer must deduplicate on `eventId`.
  - A handler that fails is retried five times with backoff, then the message
    goes to `widgets.lifecycle.dlq` and the stream moves on.

  # Stability

  Fields are added to a payload, never removed or retyped, so a consumer must
  ignore fields it does not know. A new `type` value may appear at any time and
  a consumer must skip the ones it does not handle. Removing a `type` is
  announced one release ahead and the value keeps being published, unused, for
  a further release.
````
