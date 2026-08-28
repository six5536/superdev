---
type: Template
id: template-contract-file-format
title: File Format Contract Template
description: Knowledge concept skeleton — one file others read or write, its shape, compatibility rules and stability promise.
status: stable
---

---
type: FileFormatContract
id: contract-<nnn>-file-format-<slug>
title: File Format Contract
description: <one line: which file, and what it promises>.
status: stable
---

# Files

<The paths or glob the format covers, who writes each file — the tool, the user, or both — and which of them is authoritative when they disagree.>

# Shape

```<toml|json|yaml|…>
<the format in its own schema language, or an example carrying every key.>
```

# Compatibility

<What a reader does with an unknown key, a missing optional, a version field it does not recognise, and a file a newer release wrote.>

# Stability

<Which keys are promised, how the format is versioned, and how a caller is migrated when it changes.>
