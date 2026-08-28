---
type: Template
id: template-contract-cli
title: CLI Contract Template
description: Knowledge concept skeleton — one command-line surface, its behaviour, exit codes and stability promise.
status: stable
---

---
type: CliContract
id: contract-<nnn>-cli-<slug>
title: CLI Contract
description: <one line: which binary, and what it promises>.
status: stable
---

# Commands

```
<the usage block: one line per command and flag, as --help would print it>
```

# Behaviour

- **`<command>`** <what it reads, what it writes, what it refuses, and the behaviour a script would break on.>

# Exit codes

| Code | Meaning |
|------|---------|
| 0    | <success> |
| 1    | <the command found something to report> |
| 2    | <usage error> |

# Stability

<Which commands, flags and codes are promised, over what version range, and how a breaking change is signalled.>
