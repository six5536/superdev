---
type: Issue
id: issue-090-search-callers-never-use-its-precision-controls
title: Search callers never use its precision controls
description: Isolated roles write keyword-soup queries and used no type, tag, or lifecycle filter in fourteen observed searches, because nothing tells them the search is semantic or that the filters exist.
kind: feature
lifecycle: open
links:
  - rel: references
    to: issue-089-an-exact-identifier-does-not-win-its-own-search
    note: The same searches, for how the tool ranks rather than how its callers use it.
---

# Feature: search callers use its precision controls

## Summary

`sokf_search` accepts `types`, `tags`, and `lifecycle` filters. Fourteen
searches across three requirements reviews used none of them. The same searches
were written as keyword lists rather than phrases, which is the worst input for
the semantic half of a hybrid search. Both follow from what the tool tells its
caller, which is almost nothing.

## Context

Observed across three requirements reviews of `plan-077-sokf-file-tool-parity`.
Every search supplied a query, six adjusted `limit`, and none set a filter:

```text
"documentation map triggers surfaces readme contributor-guide canonical-knowledge changelog"
"CLI sokf read command promises knowledge-relative path offset limit contract-002"
"sokf_read tool rendered virtual read overview address"
"Pi extension read edit write knowledge root repository discovery .superdev/config.toml marker"
"repository root detection .superdev/config.toml marker non-git repository init requires git\","
```

The last carries a stray `",` from a broken string, which the lenient parser
absorbed silently.

Every one is a bag of keywords. The second names `contract-002` and would have
returned it first under `types: ["Contract"]`; instead it ranked sixth, which
[the ranking issue][sokf:issue-089-an-exact-identifier-does-not-win-its-own-search]
covers.

What the caller is told explains both habits. The `query` field is documented
as "What to look for, in the caller's own words". Nothing says the search is
hybrid lexical and semantic, and nothing says a phrase retrieves better than a
keyword list. The filters are documented one line each on their own fields,
where a model composing a call sees them only if it reads the whole schema.
The server's instruction string says "use sokf_search and sokf_read for project
knowledge", which names the tool without describing how to use it well.

The isolated roles show the consequence. Search accounted for roughly one call
in ten across those reviews, against 33 to 37 reads each. Most of those reads
were direct paths reached by following links rather than by searching.

## Behaviour

A caller composing a search knows that phrasing matters and that filters exist.

The tool describes its own retrieval. A caller told the search is hybrid, and
that a natural phrase serves the semantic half better than a keyword list, can
write for the mechanism instead of against it.

The filters are visible where a caller decides. Naming them in the tool
description, with the case each one serves, reaches a model composing a call in
a way that per-field documentation does not.

A malformed query does not pass silently. The lenient parser exists so that an
unbalanced quote still searches for the words, which is correct, but a query
carrying obvious syntax debris is worth reporting rather than absorbing.

Whether the roles' prompts should also direct search use is a separate
judgement. The review prompts list the tools available and say nothing about
retrieval strategy, so a role that reaches for `read` first is following its
instructions. This issue does not decide whether that guidance belongs in the
tool, the prompt, or both.

## Scope

What a caller is told about `sokf_search` before composing a query.

- In: the tool description, the request field documentation, and the server
  instruction string.
- In: whether a query carrying syntax debris should be reported.
- Out: how results are ranked once found, which
  [the ranking issue][sokf:issue-089-an-exact-identifier-does-not-win-its-own-search]
  covers.
- Out: the filters' behaviour. They work; they are simply never used.
- Out: the role prompts, whose retrieval guidance is a separate decision about
  where such guidance belongs.

## Comments

Filed from a live session after analysing the search calls in three
requirements reviews. The queries, the filter usage, and the read-to-search
ratio were counted from the role artifacts rather than estimated.

The evidence has one limitation worth recording. Fourteen searches from three
runs of one plan show what these roles did, not what callers do generally, and
no experiment was run to confirm that phrased queries retrieve better than
keyword lists in this corpus. That expectation follows from how the semantic
half works rather than from a measurement taken here.

<!-- sokf:links -->
[sokf:issue-089-an-exact-identifier-does-not-win-its-own-search]: /knowledge/issues/open/issue-089-an-exact-identifier-does-not-win-its-own-search.md
