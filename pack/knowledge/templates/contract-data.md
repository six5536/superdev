---
type: Template
id: template-contract-data
title: Data Contract Template
description: Knowledge concept skeleton — the persisted store, its schema, the constraints it holds and how it migrates.
status: stable
---

---
type: DataContract
id: contract-<nnn>-data-<slug>
title: Data Contract
description: <one line: which store, and who reads it>.
status: stable
---

# Store

<The engine and version, where it lives, which component owns writes, and who else may read it.>

# Schema

```<sql|…>
<the definitions in the store's own language. Prose describes; this block defines.>
```

# Constraints

- <Keys and uniqueness.>
- <Referential rules, and what is nullable.>
- <Retention and deletion — soft or hard, and after how long.>

# Migration

<How the schema changes under a running system — expand-then-contract or downtime, whether old and new code read the same rows during a rollout, and how a migration is rolled back.>

# Stability

<Which tables and columns are promised to readers outside the owning component, and how a breaking change reaches them.>
