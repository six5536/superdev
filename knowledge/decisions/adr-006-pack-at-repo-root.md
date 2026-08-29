---
type: Decision
id: adr-006-pack-at-repo-root
title: The Stock Pack Lives at the Repo Root
description: superdev's shipped content moves to a browsable /pack directory at the repository root, reached from superdev-core by a symlink so it still packages into the crates.io tarball without becoming a crate of its own.
status: stable
links:
  - rel: implements
    to: spec-014-content-packs
  - rel: relates-to
    to: directory-structure
---

# ADR-006: The stock pack lives at the repo root

- Status: accepted
- Date: 2026-08-25
- Deciders: project owner

## Context

[Externally sourced content packs][sokf:spec-014-content-packs]
turns superdev's shipped content into a pack. It is also the part of this
repository a visitor most wants to read, and it sits three levels down at
`crates/lib/superdev-core/assets/`, where nothing about the path suggests it
holds anything but build inputs.
[ADR-003][sokf:adr-003-items-by-layout] already reorganises that tree into pack
layout, so relocating it at the same time costs almost nothing.

The constraint is packaging. All 254 files reach the published crate today
only because they sit under `superdev-core`'s package root: `cargo package`
collects the package directory, and `include`/`exclude` globs cannot escape
it. Moved to the repo root and left unreferenced, the content would vanish
from the tarball while every in-repo build kept working — the invisible
failure the `no_template_asset_is_named_cargo_toml` test exists to catch.

Whether a symlink crosses that boundary was an open question, so it was
measured rather than assumed. With the content at `/pack/` and
`crates/lib/superdev-core/assets` a symlink to it, `cargo package`
dereferences the link: the tarball carries `assets/**` as real files in a
real directory, and the verification build compiles `include_str!` through
it. Git records the symlink as mode 120000 and does not follow it, but cargo
does not consult git for this.

## Decision

We will move the content to `/pack/` at the repository root and reach it from
`crates/lib/superdev-core/assets` by a relative symlink. `superdev-pack` is
not created: the content stays part of `superdev-core`'s package and no new
crate is published.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Root directory, symlinked into the crate | Browsable at the root and rendered as a real directory by the GitHub web UI; no new crate on crates.io; no addition to the release lockstep; packaging verified by measurement | A Windows checkout without symlink support turns the link into a text file, and the build fails there |
| Root directory as its own crate | No symlink anywhere; packages by construction | A fourth crate published to crates.io, permanently, for a directory that is content — and a fourth link in the ordered publish chain |
| Root directory, copied in at release | No symlink, no new crate | The in-repo build and the published build differ, which is the invisible failure mode |
| Symlink the other way — files stay in the crate, `/pack` points at them | Zero packaging risk; a broken link on Windows breaks nothing that builds | The GitHub web UI renders `/pack` as a symlink pointer, not a directory listing, so a browsing visitor is no better off — which is the whole point |
| Rename in place, sign-post from the README | No risk at all | A reader still has to descend three levels |

## Consequences

- Positive: the content is the first thing a visitor sees, and it is a real
  standalone pack — the reference a third-party author copies, with nothing in
  it that only exists to serve superdev's build.
- Positive: nothing is added to crates.io or to the version lockstep
  ([release-procedure][sokf:release-procedure] counts 18 locations today).
- Negative: this is the repo's first tracked symlink, and Git for Windows
  needs `core.symlinks=true` plus Developer Mode or elevation to materialise
  one. Without it the link checks out as a text file and
  `include_str!` fails — loudly, at compile time, but it fails.
- Follow-ups:
  - The `windows-latest` job in `checks.yml` runs
    `cargo nextest run --workspace` from the checkout, so it must set
    `core.symlinks=true` before `actions/checkout`. The runners are
    administrators, so creation is permitted; this needs proving in CI on the
    slice that moves the files, not assumed.
  - CONTRIBUTING gains the Windows checkout requirement, so a contributor
    meets it as setup rather than as a mystery build error.
  - `.gitattributes` carries `crates/lib/superdev-core/assets/** -text` to
    keep owned content LF through a Windows checkout; the pattern must follow
    the real files to `pack/**`.
  - [directory-structure][sokf:directory-structure] is updated at integrate.
  - The `no_template_asset_is_named_cargo_toml` test retargets to
    `pack/projects/`.

<!-- sokf:links -->
[sokf:adr-003-items-by-layout]: /knowledge/decisions/adr-003-items-by-layout.md
[sokf:directory-structure]: /knowledge/directory-structure.md
[sokf:release-procedure]: /knowledge/release-procedure.md
[sokf:spec-014-content-packs]: /knowledge/specs/spec-014-content-packs.md
