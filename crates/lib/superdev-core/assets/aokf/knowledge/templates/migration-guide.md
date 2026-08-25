---
type: Template
id: template-migration-guide
title: Migration Guide Template
description: Old-to-new steps with per-step verification, behavioural differences, rollback, and troubleshooting.
status: stable
---

# Migrating from <old> to <new>

<One or two sentences: what changed, why, and who needs to act. State clearly who does NOT need to act.>

## At a glance

| | Before | After |
|---|--------|-------|
| <Key aspect> | `<old usage>` | `<new usage>` |
| <Key aspect> | `<old usage>` | `<new usage>` |

## Prerequisites

- <Minimum versions, backups to take, feature flags to check>

## Steps

### 1. <Step name>

<What to do, with exact commands/code:>

```sh
<command>
```

Verify: <how to confirm this step worked before moving on.>

### 2. <Step name>

<Before/after code where the change is mechanical:>

```diff
- <old code>
+ <new code>
```

## Behavioral differences

<Changes that don't show up as compile errors — different defaults, timing, error types. These are the ones that bite.>

- <Difference and its consequence>

## Rollback

<How to get back to the old state if something goes wrong, and until which step rollback stays cheap.>

## Troubleshooting

- <Symptom/error message> — <cause and fix>
