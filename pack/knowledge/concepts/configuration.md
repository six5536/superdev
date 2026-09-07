---
type: Configuration
id: configuration
title: Configuration & Environments
description: Project configuration is committed while runtime cache and credentials remain machine-local.
status: draft
---

Project configuration belongs in the repository; transient workflow state and credentials do not.

# Project configuration

Hand-edited project policy. Record each supported setting, its default, and its owning tool here when the project adopts it.

```toml
[workflow]
human_acceptance_required = true
max_stalled_block_attempts = 3
max_final_correction_cycles = 3
```

# Outside the repo

User credentials and provider configuration remain machine-local. Runtime workflow cache is transient and gitignored; a fresh machine reconstructs durable progress from canonical issue and plan records.
