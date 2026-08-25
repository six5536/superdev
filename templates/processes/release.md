# Process: Cutting a release

Releases are outward-facing and hard to reverse — confirm with the user before publishing/tagging/pushing unless explicitly told to proceed.

## 1. Confirm the release is defined

- Which commits are in it (`git log <last-tag>..HEAD`), what version it becomes (semver: breaking → major, feature → minor, fix → patch — pre-1.0 per project convention), and any project release checklist that overrides this generic one.

## 2. Pre-flight on a clean tree

- Working tree clean, on the release branch, synced with the remote.
- Full test suite, lint, typecheck, and a production build — green on exactly the tree being released.
- Verify the built artifact itself (install/run the package output, not just the source).

## 3. Prepare the release commit

- Bump the version everywhere it lives (manifest, lockfile, any hardcoded version checks).
- Update the changelog: move Unreleased into the new version section with today's date (see `templates/changelog.md`).
- Draft release notes for humans (see `templates/release-notes.md`) — highlights and breaking changes, not the commit list.

## 4. Tag and publish — with confirmation

- Show the user the version, changelog entry, and publish target; get an explicit go unless already durably authorized.
- Commit (`chore(release): vX.Y.Z`), tag, push commit and tag, then publish in the project's defined way (registry publish, GitHub release, deploy).
- Publishing is public and often irreversible (registries rarely allow unpublish) — re-check the package name/scope and target before the final command.

## 5. Verify the release landed

- Fetch/install the published artifact from the real registry and smoke-test it.
- Check the tagged CI run is green; confirm the release page/notes render correctly.

## 6. Post-release

- Start the next Unreleased changelog section; bump to a dev/prerelease version if the project uses one.
- Report: version, links (tag, release, registry), and verification performed. If anything was skipped, say so plainly.
