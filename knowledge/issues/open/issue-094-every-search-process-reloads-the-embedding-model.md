---
type: Issue
id: issue-094-every-search-process-reloads-the-embedding-model
title: Every search process reloads the embedding model
description: A search or overview costs about 2.2 seconds of compute before it answers, because each process loads the 125 MB local embedding model afresh, and the script test suite pays that cost repeatedly.
kind: chore
lifecycle: open
links:
  - rel: references
    to: issue-089-an-exact-identifier-does-not-win-its-own-search
    note: Both concern the retrieval path; that issue is about ranking, this one about cost.
---

# Chore: every search process reloads the embedding model

## Summary

A `sokf search` costs about 2.2 seconds before it answers, nearly all of it
compute spent loading the local embedding model. The cost repeats per process,
so the tests that exercise the real binary pay it many times over and the
script suite takes about 20 seconds.

## Context

Measured on one development machine against the debug binary, with the model
already cached in `~/.cache/superdev` and the index already built:

| operation | time |
|---|---|
| bare `pi` startup, no extension | 0.20 s |
| warm `cargo build -p superdev` | 0.39 s |
| `sokf read` | 0.23 s |
| `sokf graph` | 0.23 s |
| `sokf search`, three consecutive runs | 2.11 s, 2.34 s, 2.33 s |
| `sokf overview` | 2.41 s |

The split falls on the retrieval boundary the CLI contract already draws.
`read` and `graph` parse current knowledge without opening the index or loading
an embedding model, as `P_sokf-direct-retrieval-skips-embeddings` requires, and
cost about 0.23 seconds. `search` and `overview` open the 7.3 MB index and load
`minishlab/potion-retrieval-32M`, 125 MB on disk, and cost about 2.2 seconds.

The cost is compute rather than disk: 2.07 seconds of the 2.11 is user time.
It is also per process and never amortized, as the three consecutive runs show.

The script suite takes about 20 seconds, and its four slowest tests are the
ones that spawn the real binary or a real Pi child:

- 12.1 s for the SOKF Pi adapter smoke, which issues three `sokf_search` calls
  through an MCP child; that fixture alone measures 5.5 s when run by itself.
- 4.4 s for the pinned workflow service test, which builds with
  `--message-format=json` and then drives real Git.
- 1.0 s each for the two tests that spawn real Pi children.

## Behaviour

A search that does not need vectors does not pay for them, and a suite that
runs many searches pays the model load once rather than per process.

Three candidate approaches, none yet chosen:

- Build the binary under `--release` for the tests that spawn it. The load is
  compute-bound, so this is likely the largest single win and the smallest
  change.
- Let `search` reach its lexical path without constructing the embedder, so a
  query that needs no vectors skips the load entirely.
- Reuse one MCP child across a fixture's searches, so repeated searches in one
  test share a single load.

Done means the measured cost of a search, and the wall time of the script
suite, are both recorded again after the change, so the improvement is a number
rather than an impression.

## Scope

The cost of the retrieval path and the test suites that pay it.

- In: when the embedding model is loaded, and whether a lexical query needs it.
- In: the build profile and child reuse used by the script tests.
- Out: retrieval quality and ranking, which
  [the identifier ranking issue][sokf:issue-089-an-exact-identifier-does-not-win-its-own-search]
  covers.
- Out: the choice of embedding model, and the index format.

## Comments

Filed after the numbers above were measured while investigating an unrelated
suite run. The measurements come from one machine and one debug build; a
release build and other hardware would move them.

<!-- sokf:links -->
[sokf:issue-089-an-exact-identifier-does-not-win-its-own-search]: /knowledge/issues/open/issue-089-an-exact-identifier-does-not-win-its-own-search.md
