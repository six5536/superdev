---
type: Schema
id: schema-release-notes
title: Release Notes Schema
description: Release notes — headline, highlights, breaking changes with migration steps, fixes and the upgrade command.
---

# Release Notes Schema

Structural rules for release notes, matched by name
(`**/*release-notes*.md`); the source names no filing directory, and filed in the knowledge as a concept. Distinct from
the changelog: the changelog is the complete record, these are what a user
is told about one version.

````yaml
description: >
  One version announced to its users: the headline, the highlights worth
  trying, what breaks and the exact migration step, the fixes, and how to
  upgrade.
line-limit: 800

frontmatter:
  type:
    const: ReleaseNotes

sections-ordered: true
sections:
  - heading-pattern: '^.+ v\d+\.\d+\.\d+.*$'
    level: 1
    required: true
    content: prose
    description: >
      The project name and version, then one or two sentences: the headline of
      this release — the thing most users will care about.
  - heading: "Highlights"
    level: 2
    required: true
    content: bullet-list
    description: >
      One bullet per feature: what it does for the user and how to try it, in
      one or two sentences.
  - heading: "Breaking changes"
    level: 2
    content: bullet-list
    description: >
      One bullet per break: what changed, before and after, and what users
      must do. Omit the section when there are none.
  - heading: "Fixes"
    level: 2
    required: true
    content: bullet-list
    description: >
      One bullet per fix, stated as the symptom fixed from the user's point of
      view, with the issue number.
  - heading: "Other changes"
    level: 2
    content: bullet-list
    description: >
      Smaller improvements and dependency bumps worth noting. Omit when empty.
  - heading: "Upgrade"
    level: 2
    required: true
    content: code
    description: >
      The upgrade command in a fenced block, and a link to the full changelog
      comparison for this version.

example: |
  # superdev v0.2.0

  Pack sources are now refused unless they use an authenticated transport,
  and sync stops rewriting the lockfile when nothing actually changed.

  ## Highlights

  - Transport allowlist — a manifest can no longer pull pack content over an
    unauthenticated channel. Run `superdev check --transports` to see whether
    any of your sources are affected.
  - Tag pins — a pack source can name a tag instead of a revision, and the
    resolved revision is still what lands in the lockfile.

  ## Breaking changes

  - Pack transports: `git://` and `http://` sources are refused at manifest
    parse. Repoint them to `https://`; the resolved revision is unchanged.
    Full steps in the 0.1-to-0.2 migration guide.

  ## Fixes

  - Sync no longer times out on pack payloads over 50 MB (#42).
  - The lockfile is written only when a pin changed, so a no-op sync leaves
    the working tree clean (#47).

  ## Other changes

  - `gix` bumped to 0.66.

  ## Upgrade

  ```sh
  cargo install superdev --version 0.2.0
  ```

  Full changelog: https://github.com/acme/superdev/compare/v0.1.0...v0.2.0
````
