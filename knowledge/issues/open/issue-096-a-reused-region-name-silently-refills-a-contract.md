---
type: Issue
id: issue-096-a-reused-region-name-silently-refills-a-contract
title: A reused region name silently refills a contract's definition with the wrong source
description: An include names a path and a region, so moving a region's content to another file while leaving the name in place makes `validate --fix` refill the contract from whatever now carries that name, and the run passes.
kind: bug
lifecycle: open
links:
  - rel: references
    to: adr-041-an-include-block-materializes-a-source-region
    note: The mechanism this issue reports — an include names a path and a region, and nothing binds the region to the contract that includes it.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The rule the defect undermines — a materialised definition is meant to be unable to drift.
---

# Bug: a reused region name silently refills a contract's definition with the wrong source

## Summary

A contract's Definition include names a file and a region inside it. When a
region's content moves to another file but the name stays behind on different
content, `validate --fix` refills the contract from whatever now carries that
name. The contract then documents an interface it was never about, and the run
passes.

## Context

Found on 2026-09-11 while removing SOKF's routed file tools under issue-095, at
`12dde7e` on `work/095-sokf-stops-intercepting-file-tools`.

`contract-012-api-sokf-pi-file-tools` included
`/.pi/extensions/sokf.ts#tools`, which held the routed `read`, `edit`, and
`write` registrations. The removal moved those registrations to
`/archive/pi/sokf-file-tools.ts` and left the three surviving tools —
`sokf_search`, `sokf_graph`, `sokf_overview` — inside a region still named
`tools` in the original file.

`validate --fix` then refilled the routed-file-tools contract with the three
tools that have nothing to do with routing, and which `contract-013` already
governs. Validation reported no finding:

```text
documents: 244 checked against 38 schemas
concepts: 280
✓ no findings

PASS (0 error(s), 0 warning(s))
```

It was caught only because a claim made from memory — that the tree did not
validate — contradicted the run, and chasing the contradiction turned up the
refill. Nothing in the tooling would have raised it.

The near miss is the shape of the hazard. Had the region been deleted rather
than reused, `render` in `crates/lib/superdev-core/src/validate/source.rs`
would have failed loudly with `the file carries no region `tools``, and the
mistake would have been fixed the same minute. Reuse is the one case that
passes.

## Behaviour

A contract that includes a region carries the interface that contract is about,
or the run says otherwise.

The mechanism has no way to notice the substitution today. `region_lines` finds
every `sokf:begin tools` in the named file, concatenates the lines, and returns
them. It cannot ask whether this `tools` is the `tools` the contract meant,
because nothing records what it meant: under
[ADR-041][sokf:adr-041-an-include-block-materializes-a-source-region] an include
carries a path and a name and no other identity. A region is not declared to
belong to a contract, and a contract's claim on a region is not written anywhere
the validator reads. That gap undermines what
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]
promises: a materialised definition is meant to be unable to drift.

The defect is therefore one of ownership rather than one of matching. Candidate
directions, none yet chosen and none costed:

- A region names the contract it serves, in the marker or beside it, and the
  validator reports an include whose region names a different contract. This
  makes the claim explicit and checkable, at the price of a second place to keep
  in step.
- A region name is unique across the repository, so moving content cannot leave
  its name behind on something else. Cheap to check, but it forbids the
  `pack-resolution` and `document-schemas` patterns already in use, where one
  name deliberately spans several files.
- The contract records a digest of the region it last materialised, and a
  changed digest reports what moved rather than refilling in silence. This
  catches every substitution, including a same-file rewrite, but it would report
  on every ordinary source edit too, which is the noise the mechanism exists to
  avoid.

Whichever is chosen, the case above must fail: a contract whose region is
refilled from source that no longer implements its interface is a finding, not a
pass.

## Scope

- In: the binding between a contract's Definition include and the region it
  materialises, and what `validate` reports when that binding is broken by a
  move.
- Out: multi-file regions of one name, which are a deliberate pattern; a region
  that is deleted outright, which already fails; and the content of any contract
  currently on file.

## Comments

Filed 2026-09-11 from the issue-095 removal. The contract that exposed it was
corrected in that work — `contract-012` is deprecated and its include retargeted
at the archive — so this issue records the mechanism rather than the instance.

<!-- sokf:links -->
[sokf:adr-041-an-include-block-materializes-a-source-region]: /knowledge/adrs/active/adr-041-an-include-block-materializes-a-source-region.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
