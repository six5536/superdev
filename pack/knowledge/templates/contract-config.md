---
type: Template
id: template-contract-config
title: Configuration Contract Template
description: Knowledge concept skeleton — the settings a deployer supplies, where they come from, which source wins.
status: stable
---

---
type: ConfigContract
id: contract-<nnn>-config-<slug>
title: Configuration Contract
description: <one line: what must be supplied to run this>.
status: stable
---

# Settings

| Name | Type | Default | Meaning |
|------|------|---------|---------|
| `<NAME>` | <type> | <default, or "none — required"> | <what it changes> |

# Sources and precedence

- <Strongest source first — flag, environment, file, built-in default.>
- <What happens when two disagree, and whether a value is re-read while running.>

# File

```<toml|yaml|json>
<the configuration file's shape, every key. Drop this section on software configured entirely by environment.>
```

# Secrets

<Which settings carry credentials, how they are supplied, and what is never logged or echoed. Drop this section where none is sensitive.>

# Stability

<Which names and defaults are promised, how a renamed setting is carried through deprecation, and what a deployer must do at a major version.>
