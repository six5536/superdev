---
type: Issue
id: issue-081-sokf-symlink-membership
title: A knowledge file reached through a contained symlink is not SOKF knowledge
description: SOKF loading, identity resolution, search, graph traversal, repair, refiling, and validation ignore an eligible Markdown file that `knowledge/` reaches only through a repository-contained file or directory symlink, so a deduplicated or aliased concept is invisible to every SOKF surface.
kind: feature
lifecycle: open
links:
  - rel: references
    to: issue-077-sokf-file-tool-parity
    note: Read routing resolves a contained symlink to its canonical target; this issue decides whether the file behind that target is knowledge.
  - rel: references
    to: issue-031-validate-follows-symlinks-out-of-the-repo
    note: The general validator symlink walk stays there; this issue changes only SOKF membership.
---

# Feature: a contained symlink below `knowledge/` carries SOKF membership

## Summary

An author who places a file symlink or directory symlink below `knowledge/`
expects the Markdown behind it to be a concept. SOKF ignores it, so the concept
has no identity, appears in no search or graph result, and receives no repair,
refiling, or validation. The author discovers the gap only when a link to that
concept fails to resolve.

## Context

SOKF derives membership from the physical `knowledge/` tree alone. Bundle
loading in `crates/lib/superdev-core/src/sokf/bundle.rs` walks real directory
entries, so a symlinked entry contributes nothing. Identity resolution, search,
graph traversal, repair, refiling, and validation all read that bundle and
inherit the same blind spot.

Two established rules constrain the fix. Repository containment already governs
every SOKF path: a target outside the repository is refused. Repair and refiling
already own where a concept lives, so a member reached through a symlink must not
lose those behaviours or acquire two identities.

The general validator walk is a separate concern. [Validate follows symlinks out
of the repo][sokf:issue-031-validate-follows-symlinks-out-of-the-repo] governs
the filesystem traversal that reads arbitrary governed files. This issue changes
SOKF membership only and must not broaden that walk.

## Behaviour

An eligible Markdown file that `knowledge/` reaches through a repository-contained
file or directory symlink is SOKF knowledge under its logical ingress.

- A file is eligible when its selected logical path carries no hidden directory
  component and its final name ends in `.md`. `index.md` remains a reserved
  index; every other eligible file is a concept candidate or a broken file.
- The logical `knowledge/` path is the member's SOKF path even when its canonical
  target lies elsewhere inside the repository. Bundle loading, identity
  resolution, source resolution, search, graph traversal, repair, refiling, and
  validation all use that logical path.
- SOKF processes each canonical file once. When several ingresses reach one
  canonical file, a direct ingress wins; otherwise the lexically first ingress
  wins.
- A directory symlink that forms a cycle terminates traversal at the repeated
  canonical directory identity rather than recursing.
- Repair writes the canonical file and preserves every ingress symlink.
- Refiling a member selected through a file symlink moves that logical directory
  entry and leaves the canonical target in place.
- A wrong-location member reachable only beneath a directory symlink has no
  independently movable alias. Refiling leaves the canonical target and the
  ancestor symlink unchanged and reports an actionable finding.
- A target outside the repository is refused. A repository file with no logical
  ingress from `knowledge/` is not SOKF knowledge.

## Scope

SOKF membership through repository-contained symlinks.

- In: bundle loading, identity resolution, source resolution, search, graph
  traversal, repair, refiling, and validation in `superdev-core`.
- In: the membership promises on `contract-003-api-sokf` and the membership
  decision recorded in `adr-053-sokf-file-tools-delegate-to-pi`.
- Out: the Pi file-tool adapter, which resolves a contained symlink to its
  canonical target without deciding membership.
- Out: the general filesystem walk owned by
  [issue-031][sokf:issue-031-validate-follows-symlinks-out-of-the-repo].
- Out: any relaxation of repository containment.

## Comments

Separated from [issue-077][sokf:issue-077-sokf-file-tool-parity] because its
requirements review exceeded the isolated-role timeout. Membership is
`superdev-core` work with no adapter surface, so it slices cleanly.

<!-- sokf:links -->
[sokf:issue-031-validate-follows-symlinks-out-of-the-repo]: /knowledge/issues/open/issue-031-validate-follows-symlinks-out-of-the-repo.md
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/open/issue-077-sokf-file-tool-parity.md
