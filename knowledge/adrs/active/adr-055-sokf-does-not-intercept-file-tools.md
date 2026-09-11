---
type: Decision
id: adr-055-sokf-does-not-intercept-file-tools
title: SOKF does not intercept the file tools; knowledge is made consistent at turn end
description: Agents read and edit knowledge with the ordinary file tools on physical paths, SOKF serves only search, graph, and overview, and one repair-and-validate at turn end covers every writer.
lifecycle: active
links:
  - rel: supersedes
    to: adr-053-sokf-file-tools-delegate-to-pi
    note: Delegation kept the interception it was designed to make faithful; this decision removes the interception instead.
---

# ADR-055: SOKF does not intercept the file tools; knowledge is made consistent at turn end

- Date: 2026-09-11
- Deciders: superdev maintainers

## Context

[ADR-053][sokf:adr-053-sokf-file-tools-delegate-to-pi] decided that SOKF resolves
and protects knowledge targets while Pi owns file-tool semantics. That answered
how routed tools should behave, and never asked whether routing should exist.

Asking it changes the answer. The objectives are consistent interfaces,
consistent behaviour, and knowledge that is correct when the turn ends. Routing
serves none of them. It adds a second interface over the file tools, makes a
`knowledge/` path behave unlike every other path, and guards a fraction of the
writes: an agent writes knowledge with `bash`, a heredoc, `sed`, a patch, or
`git checkout` as readily as with `edit`.

A path carries more than an identity. `knowledge/issues/open/issue-082-…​.md`
states type, lifecycle, and topic; `issue-082-…` drops the lifecycle and needs an
MCP round trip to become openable. Identity stability mattered because repair
moved files mid-turn — a problem that disappears when repair runs once, at the
end.

Mutation policy protects `id`, `verified`, and `generated`. No concept in this
repository carries a `verified` or `generated` stamp, and validation already
reports an `id` that disagrees with its filename.

## Decision

We will stop intercepting the file tools. `read`, `edit`, and `write` are Pi's
own, acting on physical paths, and a knowledge file is an ordinary file.

SOKF MCP will serve only what a file tool cannot do: `sokf_search`,
`sokf_graph`, and `sokf_overview`. Source resolution, semantic retrieval, and
routed mutation are removed. Graph results will carry each concept's
repository-relative path, so a traversal reaches a file without a second lookup.

Consistency will be established once per turn. At `turn_end` the extension
repairs the knowledge and validates the result unconditionally, without first
deciding whether anything changed, so every writer is covered by one mechanism at
one moment. A conditional run would need a signal, and no signal the extension
can keep observes a write through `bash`; an unconditional one needs none to be
right, and costs a bounded second or so while writing nothing when the knowledge
is already valid. Findings are re-checked against the working tree
before they are sent; the first report for a pending sequence triggers one
correction turn and later reports are visible but non-triggering; state is
session memory keyed by repository root.

The `superdev sokf edit` and `superdev sokf write` commands, and the mutation
service they call, are unchanged and keep their inline repair. Their callers are
deterministic — the workflow transitions depend on them — and
`contract-002-cli-superdev` settles their behaviour already.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Stop intercepting; search, graph, and overview only; repair and validate at turn end | One interface, one consistency mechanism covering every writer, and a large deletion | Loses mutation-time policy, which guarded one write path and two unused stamps |
| Keep routing and bring mutation to parity, as issue-082 proposed | Preserves agent-safe policy on the file tools | Permanent adapter, resolver, and transport to guard a fraction of writes |
| Keep routed read only, dropping routed mutation | Smaller surface than full parity | Keeps resolver, adapter, and paired harness alive to serve an address carrying less than the path |
| Move repair out of mutation but keep routing | Fixes bytes changing under the caller | Leaves the interception whose cost this decision questions |
| Keep `sokf_retrieve` for rendered concepts and sections | No retrieval capability is lost | Duplicates `read` and `sokf_search` through a second dialect |
| Restore mutation policy at turn end | Recovers the guarantee interception gave | Validation already reports id-filename disagreement, and the stamps it protects are unused |

## Consequences

- Positive: one file-tool interface, so a knowledge path behaves as every other
  path and no agent learns a second dialect.
- Positive: consistency covers every writer, including `bash` and patch
  application, because the check observes the tree rather than a tool.
- Positive: bytes never move under a caller mid-turn, so a path an agent just
  used stays valid for the rest of the turn.
- Positive: the routing adapter, source resolver, mutation transport, paired
  harness, and `contract-012-api-sokf-pi-file-tools` are all removed.
- Negative: mutation-time policy no longer guards `id`, `verified`, or
  `generated` on the agent path; validation reports an `id` mismatch at turn end
  instead.
- Negative: most of the routed-read work delivered under
  [issue-077][sokf:issue-077-sokf-file-tool-parity] is reverted.
- Negative: an agent reaching knowledge semantically must follow a search or
  graph result to a path before reading it, rather than addressing a concept
  directly.
- Negative: every turn pays the repair-and-validate cost, including turns that
  touched no knowledge. Correctness is bought with a fixed cost rather than a
  signal that can be wrong.
- Follow-ups: [issue-095][sokf:issue-095-sokf-stops-intercepting-file-tools]
  implements this decision. Semantic ranking, index lifecycle, and MCP transport
  are unaffected and require separate issues.

<!-- sokf:links -->
[sokf:adr-053-sokf-file-tools-delegate-to-pi]: /knowledge/adrs/deprecated/adr-053-sokf-file-tools-delegate-to-pi.md
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/done/issue-077-sokf-file-tool-parity.md
[sokf:issue-095-sokf-stops-intercepting-file-tools]: /knowledge/issues/done/issue-095-sokf-stops-intercepting-file-tools.md
