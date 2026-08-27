---
type: Schema
id: schema-changelog
title: Changelog Schema
description: CHANGELOG.md — Keep-a-Changelog, with Unreleased plus per-release change groups and compare links.
---

# Changelog Schema

Structural rules for `CHANGELOG.md` at the repository root. It is the one
schema here that declares no order: the change groups repeat *inside* each
release heading, which itself repeats, and `sections` is a flat list keyed
by level — it can say a level-3 group repeats, but not that it repeats once
per release. The Keep a Changelog ordering — Unreleased first, then releases
newest-first — holds by convention until the vocabulary can nest repetition.

````yaml
target-files: "CHANGELOG.md"
description: >
  The Keep a Changelog record: an Unreleased section, one section per released
  version with its date, change groups under each, and the compare links.
line-limit: 800

sections:
  - heading: "Changelog"
    level: 1
    required: true
    content: prose
    description: >
      The fixed preamble: that this file documents all notable changes, that
      the format follows Keep a Changelog, and that the project follows
      Semantic Versioning — each linked.
  - heading: "[Unreleased]"
    level: 2
    required: true
    description: >
      Changes landed but not yet released. Always present, even when empty:
      its absence is what makes a release cut ambiguous.
  - heading-pattern: '^\[\d+\.\d+\.\d+\] - \d{4}-\d{2}-\d{2}$'
    level: 2
    required: true
    repeatable: true
    description: >
      One per released version: the version in brackets and its release date,
      newest first. The bracketed version is the label of the compare link at
      the foot of the file.
  - heading-pattern: '^(Added|Changed|Deprecated|Removed|Fixed|Security)$'
    level: 3
    required: true
    repeatable: true
    content: bullet-list
    description: >
      A change group under a version, from the Keep a Changelog vocabulary and
      no other. One bullet per change, phrased from the user's perspective —
      the symptom fixed, not the internal mechanics — with the issue number
      where there is one. A Security entry carries enough detail to assess
      exposure without being a how-to. Groups with nothing in them are
      omitted, not left empty.

example: |
  # Changelog

  All notable changes to this project are documented in this file.

  The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
  and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

  ## [Unreleased]

  ### Added
  - `superdev check` now validates documents against their schemas.

  ## [0.2.0] - 2026-08-26

  ### Added
  - Pack sources may be pinned to a tag as well as a revision.

  ### Changed
  - `superdev sync` writes the lockfile only when a pin actually changed.

  ### Fixed
  - Pack sync no longer times out on payloads over 50 MB (#42).

  ### Security
  - Pack sources are refused unless the transport is https, ssh or file.
    Earlier versions would fetch over any transport git accepted.

  [Unreleased]: https://github.com/acme/superdev/compare/v0.2.0...HEAD
  [0.2.0]: https://github.com/acme/superdev/compare/v0.1.0...v0.2.0
````
