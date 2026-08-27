---
type: Schema
id: schema-release-procedure
title: Release Procedure Schema
description: How a release is cut, the gates on it, and the steps that cannot be undone, in knowledge/release-procedure.md.
---

# Release Procedure Schema

Structural rules for `knowledge/release-procedure.md`, the bundle's
Procedure concept for cutting a release. The document has no headings — a
lead line, the ordered steps, and a note on credentials — so it declares a
preamble and no sections.

````yaml
target-files: "knowledge/release-procedure.md"
description: >
  How a release is triggered, the gates that must pass first, the step that
  publishes irreversibly, and where publish rights come from.
line-limit: 800

frontmatter:
  type:
    const: Procedure
  id:
    const: release-procedure
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: numbered-list
  description: >
    How releases are triggered, in one line, linking the authoritative
    walkthrough if one lives elsewhere. Then the ordered steps: the gate — what
    must exist or pass before a release can be cut; the cut — the command or
    action and everything it changes; the point of no return — which step
    publishes irreversibly, called out as such; and what the pipeline does from
    there, and how a prerelease differs. Close with credentials: where publish
    rights come from, and which secrets exist.

example: |
  ---
  type: Procedure
  id: release-procedure
  title: Release Procedure
  description: How a release is cut, the gates, and the irreversible steps.
  status: stable
  ---

  Releases are cut from `main` by pushing a version tag; nothing else
  triggers a publish.

  1. Gate: `just check` passes on `main`, the changelog has an entry under
     Unreleased, and no issue is tagged `release-blocker`.
  2. The cut: `just release X.Y.Z` bumps the manifest versions, rewrites the
     changelog heading, commits, and pushes the tag.
  3. Point of no return: the tag push starts the publish job. A published
     version cannot be replaced or withdrawn — only superseded by a new
     patch, so a bad release costs a release rather than a revert.
  4. From there CI builds, publishes, and opens the release notes as a draft.
     A prerelease uses a `-rc.N` suffix, skips the notes draft, and is not
     tagged latest.

  Credentials: the publish token is an organisation secret available only to
  the tagged-release workflow; no other job can read it.
````
