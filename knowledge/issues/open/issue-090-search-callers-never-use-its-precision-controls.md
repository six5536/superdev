---
type: Issue
id: issue-090-search-callers-never-use-its-precision-controls
title: Search callers never use its precision controls
description: Isolated roles write keyword-soup queries and used no type, tag, or lifecycle filter in fourteen observed searches, because nothing tells them the search is semantic or that the filters exist.
kind: feature
lifecycle: open
sources:
  - id: experiment
    resource: /evals/sokf/search-results.json
    title: Luna search experiment requests, evidence choices, metrics and hashes
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

A query that triggers lenient-parser recovery remains searchable and reports
the parser's diagnostic. Do not guess whether prose is malformed with a new
syntax-debris heuristic.

Whether the roles' prompts should also direct search use is a separate
judgement. The review prompts list the tools available and say nothing about
retrieval strategy, so a role that reaches for `read` first is following its
instructions. This issue does not decide whether that guidance belongs in the
tool, the prompt, or both.

## Scope

What a caller is told about `sokf_search`, and whether those instructions and
its results help a caller reach the relevant evidence.

- In: the tool description, the request field documentation, and the server
  instruction string.
- In: whether a query carrying syntax debris should be reported.
- In: a small scripted LLM evaluation of caller guidance, direct-match
  priority and match labels. Ranking changes overlap
  [the ranking issue][sokf:issue-089-an-exact-identifier-does-not-win-its-own-search];
  test them separately from guidance rather than assuming either is sufficient.
- Out: a broad ranking rewrite or parameter sweep without evidence.
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

### Development notes

The user requested direct development without SCOPE, BUILD or ACCEPT. Run a
small LLM evaluation during development and record results here. Success means
better retrieval of relevant evidence, not more frequent filter use. Definitions
of a direct match and the usefulness of match labels are experimental questions.

Use `openai-codex/gpt-5.6-luna`, the least expensive of Luna, Terra and Sol in
the installed provider catalogue. Keep a frozen corpus and baseline binary so
issue updates and concurrent work do not change the comparison. Start with six
tasks covering identifier, title, path, semantic intent, scoped live work and
an absent target. Hide expected answers from the model, record usage, and keep
both network calls and output bounded. This is a development sample, not a
statistical acceptance claim.

The existing evaluation scores tool-call order, not relevance. A separate
search experiment will let the LLM compose requests and select evidence from
real search results; deterministic checks will score those choices against
predeclared relevant concepts. Answer quality beyond evidence selection will
need manual inspection and will not be claimed from a matching identifier alone.

The first Luna baseline made 4/6 correct evidence choices, with a mean
reciprocal rank of 0.60 across the five present targets. The identifier and
path targets were absent from its eight-section results. It inferred the
identifier from another document instead of selecting returned evidence. All
six generated requests were phrases, not keyword lists; one used a lifecycle
filter. The historical observations do not justify a universal claim that
callers always write keyword lists or never use filters.

The first guidance prototype made 5/6 correct choices, also at 0.60 MRR. It
retrieved the identifier and path but lost the decision: the example written
for this experiment used `ADR`, while the corpus actually uses `Decision`.
That was an error in the proposed guidance, not in the filter implementation.
Correct the example before judging filters. Model-reported estimated cost for
the two six-task runs was $0.0051 in total.

The scorer was corrected without rerunning the model: an unambiguous numbered
shorthand is credited only when the corresponding full concept is present in
the returned results. Thus `issue-096` correctly selects its returned issue;
`contract-002` inferred from a review does not count as retrieved evidence.
Raw requests and replies are retained in the experiment reports.

The bounded comparison is now recorded in the executable experiment
artefact.[^experiment] Each row uses the same frozen corpus; replay rows reuse
requests verbatim rather than asking the model to compose new ones. MRR is
measured over concept groups, not individual sections.

| Variant | Correct choices | False choices | MRR, present targets | LLM calls |
| --- | ---: | ---: | ---: | ---: |
| Original guidance and ranking | 4/6 | 1 | 0.60 | 12 |
| First guidance prototype, wrong type example | 5/6 | 0 | 0.60 | 12 |
| Corrected guidance, original ranking | 6/6 | 0 | 0.70 | 12 |
| Corrected queries, direct priority and labels | 6/6 | 0 | 0.90 | 6 |
| Same queries and ranking, labels withheld | 6/6 | 0 | 0.90 | 6 |
| Original queries, direct priority and labels | 6/6 | 0 | 1.00 | 6 |

Total: 54 Luna requests, estimated $0.0121. The whole-query and evidence-choice
prompts are tool-free; calls and result context are bounded. This does not measure adaptive agent retries,
subsequent file reads, latency or final answer correctness. Expected concepts
are hidden from the model. The path case deliberately probes retrieval of a
known address; ordinary agents should read a known path without searching.

Decisions from this development sample:

- Implement exact ID, unambiguous numbered shorthand and path priority before
  hybrid relevance. It repairs the observed identity failures even without new
  guidance. Every explicit filter still applies, including to settled concepts.
- Do not prioritise ordinary single-word IDs inside prose: `testing` is a
  direct ID only as the whole query. Do not guess ambiguous numbered IDs.
- Keep concise, accurate descriptions of the existing controls, without
  promising that short phrases or filters always improve retrieval. Corrected
  guidance loses one rank on the semantic task versus the original phrasing;
  both still select the right evidence. Filter use is not the success measure.
- Keep small per-section identifier/path/lexical/semantic/combined labels to
  explain retrieval. The label-only comparison shows no accuracy improvement;
  no confidence or probability is implied by a label.
- Do not boost literal titles or arbitrary body phrases: the title case already
  ranks first. Do not tune RRF or historical-work penalties on this sample.
- Surface up to three distinct diagnostics from the actual lenient parser while
  preserving recovered results. Cover malformed quotes with executable tests,
  not further paid model trials.

The implementation also separates index storage, retrieval, address matching
and tests into named files; no index schema or stored-vector format changes.

Verification: formatting, workspace clippy with warnings denied, the doctest,
warning-free rustdoc, 45 script tests, launcher tests, documentation checks,
blueprint drift and knowledge validation pass. The final binary reproduces all
12 rendered results from the two paid direct-priority replay arms byte-for-byte;
parser diagnostics were then covered without further model calls.

The full Rust suite reports 879/880 passing. The remaining failure is
`each_harness_receives_its_own_sokf_authoring_skill`: it expects literal
Pi-specific prose removed from the shortened authoring skill. Both skill copies
and that test are unchanged from HEAD for this work. This is a pre-existing
failure, not a waived gate; the user's canonical guidance was left intact.
Coverage, version and release gates were not rerun. This issue remains open;
no workflow acceptance is claimed.

Double-check found and corrected a path-matching error: splitting paths into
identifier tokens could promote `notes.md` when the requested path was
`guides/rust+api notes.md`. Whole-query and quoted paths now remain intact,
including spaces in the checkout root, filename punctuation and `./` prefixes.
Native path components also recognise Windows separators; that platform-specific
regression test is not executed on the Linux checkout. A failing regression test
preceded the correction. Single-word IDs still need to be the whole query, and
ambiguous shorthand and explicit filters retain their earlier behaviour.

The evaluation runner now checks knowledge, configuration and binary hashes
through each task and verifies the retrieval model across cases and replays.
Malformed model JSON no longer loses already-reported token and cost usage.
Offline tests cover drift, accounting and stopping before search after a
configuration change. Missing configuration/model metadata was added to old
reports from the unchanged frozen inputs, explicitly labelled as retrospective;
no requests, choices or measured outcomes were rewritten. All 12 rendered
results from the two direct-priority replay arms remain byte-identical, so no
new paid model calls were made during the review.

The evidence supports a small development improvement, not a universal optimum.
The test set does not cover tag-scoped tasks or multi-document questions, and
the recorded metric measures evidence selection rather than final answer quality.

After double-check, formatting, clippy, the doctest, warning-free rustdoc,
49 script tests, launcher tests, documentation checks and blueprint drift pass.
The full Rust run passes 880/881 tests; only the same unchanged authoring-skill
wording test fails. Knowledge validation reports zero findings. The review did
not change the user's canonical instructions or waive that pre-existing gate.

[^experiment]: Six-task development experiment using openai-codex/gpt-5.6-luna.

<!-- sokf:links -->
[sokf:issue-089-an-exact-identifier-does-not-win-its-own-search]: /knowledge/issues/open/issue-089-an-exact-identifier-does-not-win-its-own-search.md
