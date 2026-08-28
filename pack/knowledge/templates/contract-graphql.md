---
type: Template
id: template-contract-graphql
title: GraphQL Contract Template
description: Knowledge concept skeleton — one graph in SDL, its endpoint, errors, limits and deprecation policy.
status: stable
---

---
type: GraphqlContract
id: contract-<nnn>-graphql-<slug>
title: GraphQL Contract
description: <one line: which graph, and what it promises>.
status: stable
---

# Schema

```graphql
<the surface in SDL — types, queries, mutations, subscriptions, and the @deprecated directives in force.>
```

# Endpoint and authentication

<The URL, the methods accepted, whether persisted queries and introspection are available in production, and what a caller presents to authenticate.>

# Errors

<How failures reach the caller — the errors array beside partial data, the extension field carrying a machine-readable code, and which conditions fail at the transport instead.>

# Limits

<Query depth, complexity budget, pagination caps and rate limits, and what a caller sees on exceeding one. Drop this section where the graph imposes none.>

# Stability

<What may be added without notice, how a field is deprecated and how long it survives, and the rare change that forces a second endpoint.>
