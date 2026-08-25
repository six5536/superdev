---
type: Template
id: template-release-notes
title: Release Notes Template
description: Headline, highlights, breaking changes with migration steps, fixes, and the upgrade command.
status: stable
---

# <project-name> v<X.Y.Z>

<One or two sentences: the headline of this release — the thing most users will care about.>

## Highlights

- <Feature name> — <what it does for the user and how to try it, one or two sentences>
- <Feature name> — <...>

## Breaking changes

<For each, what breaks and the exact migration step. Delete section if none.>

- <What changed>: <before → after, and what users must do>

## Fixes

- <Symptom fixed, from the user's point of view> (#issue)
- <...>

## Other changes

- <Smaller improvements, dependency bumps worth noting>

## Upgrade

```sh
<upgrade command>
```

Full changelog: <compare-url>/v<prev>...v<X.Y.Z>
