---
type: Issue
id: issue-077-sokf-file-tool-parity
title: SOKF-routed reads diverge from Pi's built-in read contract
description: A SOKF-routed read returns a rendered concept instead of exact source, and one mixed-purpose MCP operation serves both file reading and semantic retrieval, so a copied excerpt cannot be edited back and the read slot carries a second result dialect.
kind: feature
lifecycle: open
links:
  - rel: references
    to: idea-012-sokf-mutations-survive-validation-failures
    note: Originating mutation-survival idea; issue-082 and issue-083 settle its mutation and follow-up behaviour.
  - rel: references
    to: issue-081-sokf-symlink-membership
    note: Source resolution accepts a contained symlink target; membership selects which ingress owns it.
  - rel: references
    to: issue-082-sokf-mutation-parity
    note: Routed mutation reuses this resolver and the pinned paired harness.
---

# Feature: SOKF-routed reads match Pi's built-in read contract

## Summary

An agent reading canonical knowledge needs `read` to behave as it does on any
other file. A routed read instead returns a rendered concept, so a copied
excerpt no longer matches the source it came from and a later targeted edit
fails. One MCP operation serves both exact file reading and semantic retrieval,
which puts a second result dialect in Pi's `read` slot.

## Context

The [SOKF mutations survive validation failures][sokf:idea-012-sokf-mutations-survive-validation-failures]
idea originated this work; issue-082 and issue-083 settle its mutation and
follow-up behaviour.

`SOKF-EDIT-RELIABILITY-PLAN.md` records the observed gaps, and repository
inspection confirms each one. Virtual reads use `SokfService::read_path()` and
return rendered concepts rather than source bytes. The same MCP `sokf_read`
operation answers an overview address, a concept address, a section-qualified
address, and a physical path, so its result shape depends on which was asked.

Pi 0.85.1 exposes the executable reference through `createReadTool()`,
`ReadToolDetails`, inherited renderers, and `resolveReadPathAsync()`. Pi scopes
built-in tools and sessions to the active working directory, recognizes a linked
worktree's `.git` pointer file, and keeps that worktree distinct from the main
checkout. The current adapter's `findRepository()` accepts any `.git` entry
without distinguishing those cases.

The SOKF MCP contract is unreleased and already permits tool and schema changes
without a deprecation path, so preserving the mixed `sokf_read` behaviour is not
a requirement.

## Behaviour

A caller reading a physical path or an equivalent SOKF identity observes the same
Pi read contract.

- Routed reads return exact UTF-8 source and preserve built-in pagination,
  2,000-line and 50 KB truncation, continuation text, details, errors,
  cancellation, metadata, and rendering.
- The Pi `read` slot treats `sokf:<id>` as a file identity. It rejects the `sokf:`
  overview address and section-qualified addresses with concise guidance toward
  semantic retrieval.
- The MCP contract replaces the mixed-purpose `sokf_read` operation with a source
  resolver for file routing and a semantic `sokf_retrieve` operation for
  overviews, rendered concepts, and sections. The change requires no
  compatibility alias or deprecation path.
- Source resolution accepts existing and missing contained physical paths,
  repository-root and nested-working-directory spellings, and Pi's accepted path
  preparation, including one leading `@`. A direct physical argument must enter
  through `knowledge/`; a symlink may canonicalize to any target inside the
  repository, including a target outside `knowledge/`. Resolution refuses
  repository escapes and reports generated-region authority.
- The replacement operations expose exact closed request and structured-result
  schemas. Source resolution returns `ingressPath`, `canonicalPath`, `exists`,
  and line-bounded `generatedRegions`, and carries no semantic content.
- The entire SOKF adapter and its MCP server use the canonical active checkout
  root rather than Git's shared common directory or main checkout. A routed path
  entering another checkout fails before access.
- A copied routed excerpt, including frontmatter, matches the source it came
  from, so it works unchanged as an exact-replacement anchor.
- A paired characterization harness compares built-in and routed results, errors,
  details, and rendering inputs across success and edge cases, and pins
  `@earendil-works/pi-coding-agent` 0.85.1 as a test dependency.

## Scope

Routed read behaviour and the MCP operations it depends on.

- In: the Pi adapter's read routing and active-checkout discovery, the MCP tool
  list, source resolution, semantic retrieval, their exact closed schemas, the
  complete source-resolver matrix, paired read evidence, and the pinned Pi test
  dependency.
- In: the read, resolution, retrieval, and worktree promises on
  `contract-003-api-sokf` and `contract-012-api-sokf-pi-file-tools`, and the
  delegation and containment decisions in
  `adr-053-sokf-file-tools-delegate-to-pi`.
- Out: routed edit and write behaviour and the mutation outcome boundary, owned
  by [issue-082][sokf:issue-082-sokf-mutation-parity].
- Out: SOKF membership through contained symlinks, owned by
  [issue-081][sokf:issue-081-sokf-symlink-membership]. This issue resolves a
  contained symlink to its canonical target without deciding membership.
- Out: the validation follow-up state machine and file-tool prompt metadata,
  owned by [issue-083][sokf:issue-083-sokf-validation-follow-ups].
- Out: a semantic retrieval dialect in the Pi `read` slot, changes to semantic
  ranking or rendered retrieval content, unrelated MCP transport lifecycle
  changes, approximate forks of Pi's algorithms, the general validator symlink
  walk tracked by issue-031, and any weakening of SOKF safety.

## Comments

Originally scoped as one issue covering read, mutation, symlink membership, and
validation follow-ups. Its requirements review exceeded the isolated-role timeout
at 1200 s without submitting a result, so the scope was split into this issue and
[issue-081][sokf:issue-081-sokf-symlink-membership],
[issue-082][sokf:issue-082-sokf-mutation-parity], and
[issue-083][sokf:issue-083-sokf-validation-follow-ups]. Deferral markers on the
shared contracts name the issue that settles each promise.

<!-- sokf:links -->
[sokf:idea-012-sokf-mutations-survive-validation-failures]: /knowledge/ideas/idea-012-sokf-mutations-survive-validation-failures.md
[sokf:issue-081-sokf-symlink-membership]: /knowledge/issues/wontfix/issue-081-sokf-symlink-membership.md
[sokf:issue-082-sokf-mutation-parity]: /knowledge/issues/open/issue-082-sokf-mutation-parity.md
[sokf:issue-083-sokf-validation-follow-ups]: /knowledge/issues/open/issue-083-sokf-validation-follow-ups.md
