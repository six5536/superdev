---
type: Template
id: template-contract-rpc
title: RPC Contract Template
description: Knowledge concept skeleton — one RPC service in its IDL, its transport, errors and wire compatibility.
status: stable
---

---
type: RpcContract
id: contract-<nnn>-rpc-<slug>
title: RPC Contract
description: <one line: which service, and what it promises>.
status: stable
---

# Services

```<protobuf|thrift|json>
<the service and message definitions. Field numbers and reserved tags are part of the contract.>
```

# Transport

<Where a client connects and over what, the deadline and message-size limits it must respect, and which methods stream in which direction.>

# Authentication

<What the client presents and on which metadata key, how it expires, and the status returned for a missing or rejected credential.>

# Errors

| Code | Condition |
|------|-----------|
| `<CODE>` | <what provokes it, and whether it is safe to retry> |

# Stability

<The wire compatibility rules — field numbers never reused, what may be added — and the deprecation window callers get.>
