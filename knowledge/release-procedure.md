---
type: Procedure
id: release-procedure
title: Release Procedure
description: The changelog gate, the release command, the irreversible push, and the tag-driven pipeline.
status: stable
sources:
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Contributing guide (releasing, authoritative walkthrough)
---

Releases are tag-driven; the authoritative walkthrough is
[CONTRIBUTING → Releasing](/CONTRIBUTING.md).[^contributing] The invariants:

1. **Changelog first.** A `## [X.Y.Z]` section must exist in `CHANGELOG.md`
   before cutting a release. `npm run release` and the release workflow both
   refuse a version with no section; the section becomes the GitHub release
   notes.
2. **`npm run release X.Y.Z`** sets the version everywhere in lockstep (18
   locations, including both lockfiles and this repo's own skills pin in
   `.superdev/`), verifies, commits, and tags — it never pushes.
3. **Review, then `git push --follow-tags`.** Pushing the tag triggers the
   publish, which cannot be undone (crates.io never; npm after 72 hours).
4. The workflow verifies, runs the full check gate, cross-builds, dry-runs
   every publish, publishes (platform packages → launcher → cargo), and
   creates the GitHub Release — see
   [software-components](software-components.md) for the job breakdown.
   Prerelease tags (`vX.Y.Z-rc.N`) publish under npm's `next` dist-tag and
   never become `latest`.

Credentials: npm uses trusted publishing (OIDC) — no token; crates.io uses
`CARGO_REGISTRY_TOKEN`.

**Before the first release**: every npm package (the launcher and the five
platform packages) needs a `0.0.0` placeholder published by hand and a
trusted publisher attached on npmjs.com — a trusted publisher can only be
attached to a package that exists. Those placeholders **must not be
unpublished** (removing a package's only version can take its
trusted-publisher configuration with it). The same applies to any platform
package added later; the steps are in
[CONTRIBUTING → Adding a platform package](/CONTRIBUTING.md).

[^contributing]: Contributing guide (releasing, authoritative walkthrough)
