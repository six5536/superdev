---
type: Template
id: template-contract-events
title: Event Contract Template
description: Knowledge concept skeleton — one published message stream, its payloads, delivery guarantees and stability promise.
status: stable
---

---
type: EventContract
id: contract-<nnn>-events-<slug>
title: Event Contract
description: <one line: which stream, and what it promises>.
status: stable
---

# Transport

<Where the messages appear — broker, topic, queue or webhook endpoint — how a consumer subscribes, and how it authenticates.>

# Messages

```<json|protobuf|avro|typespec>
<the payloads in their own schema language. Every field a consumer may read is defined here.>
```

# Ordering and delivery

- <Ordering guarantees, and what they are keyed on.>
- <At-least-once or exactly-once, and which field makes a message idempotent.>
- <Retry and dead-letter behaviour.>

# Stability

<How payloads are versioned, what may be added without a new version, and how a consumer is told a message type is going away.>
