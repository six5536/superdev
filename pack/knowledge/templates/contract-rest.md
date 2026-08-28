---
type: Template
id: template-contract-rest
title: REST Contract Template
description: Knowledge concept skeleton — one HTTP API in TypeSpec, its authentication, errors and stability promise.
status: stable
---

---
type: RestContract
id: contract-<nnn>-rest-<slug>
title: HTTP API Contract
description: <one line: which API, and what it promises>.
status: stable
---

# Endpoints

```typespec
<the surface in TypeSpec — routes, request and response models, status codes.
Every field a caller may read or send is defined here, not in the prose.>
```

# Authentication

<What a caller presents, where it goes, how it is obtained and how it expires. The response to a missing or rejected credential.>

# Errors

| Status | Condition |
|--------|-----------|
| <400>  | <what provokes it> |

# Stability

<How the API is versioned, what may be added within a version, what forces a new one, and the deprecation window callers get.>
