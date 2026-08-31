---
type: Decision
id: adr-007-git-fetch-by-spawn
title: Git Pack Sources Are Fetched by Spawning Git
description: superdev resolves a git pack source with a shallow, blobless, sparse clone through the user's own git binary, so any forge, private repo and ssh URL works with the user's credentials and no token is stored.
lifecycle: active
links:
  - rel: implements
    to: contract-007-interface-pack-resolution
  - rel: relates-to
    to: architectural-rules
---

# ADR-007: Git pack sources are fetched by spawning git

- Date: 2026-08-25
- Deciders: project owner

## Context

[The pack-resolution contract][sokf:contract-007-interface-pack-resolution] allows a pack source to be
a git URL, and the trust model rests on the user's own credentials — superdev
stores no token and adds no auth surface. superdev has never invoked git: its
only git awareness is `root.join(".git").exists()`, a filesystem test. `ureq`
is already a dependency, so an HTTPS tarball was the alternative.

The tarball only reaches forges whose archive URL superdev can construct.
That covers `github:owner/repo` and little else — not ssh, not self-hosted,
not a private repo without a token the design has promised not to hold. The
contract already offers `git+ssh://git@git.acme.internal/dev/packs.git` as a
supported example.

## Decision

We will resolve a git pack source by spawning the user's `git`: a shallow,
blobless, sparse clone that takes the pack directory and not the history.
Assuming git is present is safe — `init` refuses a directory that is not a
git repository, so every superdev user already has one.

```
git clone --depth 1 --filter=blob:none --sparse <url> --branch <rev> <dir>
git -C <dir> sparse-checkout set pack
```

This also settles how a pack is released: pushing a tag is the whole of it.
No archive job, no artifact, no registry.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Spawn `git` | Any forge, ssh, self-hosted and private repos, on the user's own credentials; no new crate; matches the existing rule that unpinned tools are spawned directly; releasing a pack is just a tag | Depends on a binary superdev does not pin, and on its version and config |
| HTTPS tarball via `ureq` | No subprocess; fully in-process tests; no reliance on the user's git | Per-forge URL construction, so github-and-gitlab-shorthand only — no ssh, no self-hosted, no private repo without a stored token |
| Tarball for shorthand, git for the rest | Fast common path, full generality underneath | Two fetch paths whose results, errors and digests must stay identical |
| A pure-Rust git library | No subprocess, works everywhere | A large dependency tree against a deliberately tight [dependency-policy][sokf:dependency-policy], to replace a binary every user has |

## Consequences

- Positive: releasing content is `git tag assets-vX.Y.Z && git push --follow-tags`.
  The pack-archive CI job the plan carried is not needed.
- Positive: no rule conflict —
  [architectural-rules][sokf:architectural-rules] already says tools superdev
  does not pin are spawned directly, as `claude` is.
- Negative: git becomes a runtime requirement for resolving a git source, and
  its absence must fail with a message that says so.
- Negative: `--branch` accepts a tag or branch but not a commit sha. Pinning
  a sha needs `git init` + `git fetch origin <sha>` + checkout, which most
  forges allow but none guarantee. Build should implement the sha path
  explicitly rather than discover it.
- Follow-ups: `technology-stack` records git as a runtime requirement at
  integrate; the release procedure gains the content-only tag flow.
- The resolver takes the `CommandRunner` the rest of the codebase spawns
  through, so no test reaches a real network and the fetch is scripted like
  every other command. It is the one side-effect outside the engine, and
  [architectural-rules][sokf:architectural-rules] states the exception.

<!-- sokf:links -->
[sokf:architectural-rules]: /knowledge/architectural-rules.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:dependency-policy]: /knowledge/dependency-policy.md
