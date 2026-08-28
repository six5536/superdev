---
type: Template
id: template-contract-telemetry
title: Telemetry Contract Template
description: Knowledge concept skeleton — the metrics, the log shape and the traces operators build on.
status: stable
---

---
type: TelemetryContract
id: contract-<nnn>-telemetry-<slug>
title: Telemetry Contract
description: <one line: what this component emits>.
status: stable
---

# Metrics

| Name | Type | Labels | Meaning |
|------|------|--------|---------|
| `<name>` | <counter|gauge|histogram> | <labels, and which are bounded> | <what it counts> |

# Logs

<The format, the fields every line carries, what the levels mean and when each is used, and what is never logged.>

# Traces

<The spans emitted, how context is propagated in and out, and the sampling a consumer should expect. Drop this section where none are emitted.>

# Stability

<Which names, labels and fields are promised, how a rename is carried so dashboards keep working, and how a removal is announced.>
