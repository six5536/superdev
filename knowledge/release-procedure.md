---
type: ReleaseProcedure
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
[CONTRIBUTING → Releasing](/CONTRIBUTING.md).[^contributing] There are two
release paths and each is one command
([ADR-008](decisions/adr-008-one-command-per-release.md)). The invariants:

1. **Changelog first.** A `## [X.Y.Z]` section must exist in `CHANGELOG.md`
   before cutting a release. `npm run release` and the release workflow both
   refuse a version with no section; the section becomes the GitHub release
   notes. A content release carries its own `## [assets-vA.B.C]` section,
   because it has no binary section to be described by.
2. **`npm run release X.Y.Z`** sets the version everywhere in lockstep (18
   locations, including both lockfiles and this repo's own skills pin in
   `.superdev/`), verifies, commits, and tags — it never pushes. It cuts the
   content release riding on that commit too: `pack/pack.toml`'s version and
   `DEFAULT_PACK.rev` move together and the commit is tagged `assets-vA.B.C`
   as well as `vX.Y.Z`, so a binary can never ship pinned at content it did
   not embed. A second argument names the pack version; left off, it is the
   version `pack.toml` declares while that is unreleased and the next patch
   once it is out.
3. **Review, then `git push --follow-tags`.** Pushing the tag triggers the
   publish, which cannot be undone (crates.io never; npm after 72 hours).
4. The workflow verifies, runs the full check gate, cross-builds, dry-runs
   every publish, publishes (platform packages → launcher → cargo), and
   creates the GitHub Release — see
   [software-components](software-components.md) for the job breakdown.
   Prerelease tags (`vX.Y.Z-rc.N`) publish under npm's `next` dist-tag and
   never become `latest`.
5. **`npm run release:pack [A.B.C]`** cuts a content release alone — the
   skills, templates and scaffolds change with no new binary. The `v*` trigger
   does not match an `assets-v` tag, so no workflow runs and nothing reaches a
   registry; pushing it is what lets `update` find the release. Both commands
   settle the version before writing anything, and refuse one that is
   malformed, behind what `pack.toml` declares, or a candidate for a version
   already out.

A binary release candidate cuts a **candidate** content tag,
`assets-vA.B.C-rc.N`. `update` moves a pin only between three-number
releases, so candidate content never reaches a released binary's users; the
candidate binary's own pin still names exactly what it embedded, and a repo
left pinned on one comes forward as soon as a release covers it.

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
