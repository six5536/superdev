---
type: Template
id: template-contract-library
title: Library Contract Template
description: Knowledge concept skeleton — one published library, its exported API, errors and stability promise.
status: stable
---

---
type: LibraryContract
id: contract-<nnn>-library-<slug>
title: Library Contract
description: <one line: which package, and what it promises>.
status: stable
---

# Package

<The published name, the registry it goes to, the runtimes or toolchain versions supported, and any feature flags that change the surface.>

# Public API

```<language>
<the exported surface as a caller sees it — types, functions, traits or interfaces. Anything absent here is private.>
```

# Errors

<The error type callers match on, what each variant means, and what the library panics or throws on rather than returning.>

# Stability

<The versioning scheme, what counts as a breaking change to this surface, and how long a deprecated item stays before removal.>
