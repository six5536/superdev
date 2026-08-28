---
type: Template
id: template-contract-interface
title: Interface Contract Template
description: The interfaces build codes against — data model and API, module boundaries, key flows, and cross-cutting concerns — each expressed in its native language, or TypeSpec.
status: stable
---

# Interface contract: <feature name>

<One paragraph: the interfaces this feature adds or changes, and the decisions behind them — link the ADRs. A working document: build codes against it, and it is discarded once the code is canonical. Its decisions remain in the ADRs, its definitions in the code and the public contracts.>

## Data model & API

<Each contract in its native language — the language the code will enforce: SQL DDL for the schema, the host language's types, traits or interfaces for module APIs, the framework's route definitions for endpoints. A contract with no native form — a language-neutral HTTP API — is written in TypeSpec. Prose describes; it never defines.>

<One fenced code block per contract, tagged with its language (`sql`, `rust`, `ts`, `typespec`, …)

```<language>
<schema / signatures / TypeSpec>
```

## Module boundaries

<Which module owns what, and the direction of dependency. One line per boundary.>

## Key flows

<Describe the 1–3 most important scenarios across the interfaces, end to end.>

## Cross-cutting concerns

- Security: <authn/authz, data exposure, input validation>
- Performance: <expected load, hot paths, limits>
- Migration/rollout: <how we get from the current state to this, and how we roll back>
- Observability: <what is logged or measured to know it works>
