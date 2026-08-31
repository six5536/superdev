---
name: maintain
description: "Use when the user asks to maintain, audit, tidy, or check the canonical knowledge, KB, or the workflow's records, and regularly between times."
---

<skill name="maintain" purpose="Maintain the Knowledge" input="concepts or checks to focus on, when given" user-input="$ARGUMENTS" output="a report: fixes grouped by check; findings that need a human (lapsed verifications, code-vs-doc conflicts, judgement calls); what was intentionally left alone">

<goal persona="knowledge's auditor">
You check the canonical knowledge at `knowledge/` — the workflow's records included — and repair what you find.
</goal>

<constraints>
The MUST NOT rules below are from SPEC §4, §5, §7 — never break these.
</constraints>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path=".agents/sokf/SPEC.md" when="before editing" />
<tool_call name="read_file" path=".agents/professionalism.md" when="before editing" />
</bootstrap_actions>

<process_actions>
<loop until="the validator passes">
<step name="RUN THE VALIDATOR" task="Run the validator per the core validation block. Fix every error; treat warnings as work items. Broken links usually mean a rename the canonical knowledge missed — fix the reference, not the target" />
</loop>
<step name="SCRIPT THE UNCOVERED CHECKS">Script the checks the validator does not cover (a throwaway script in the scratchpad is fine); don't eyeball them:

- `knowledge/index.md` lists every concept, and each entry's text matches the concept's `description` (the index lowercases the first word; ignore that difference).
- Every file `AGENTS.md` references (its `@`-imports and links) exists.
- A `verified.at` older than the file's last content change (`git log -1 --format=%cI -- <file>`) is lapsed. Report it; do not touch the field.</step>
  <step name="CHECK ACCURACY AGAINST THE CODE" task="Check accuracy against the code, per the core's `core_principles` block. For each concept whose `resource` or repo-path `sources` changed after it (compare the `git log -1 --format=%cI` dates), read the changed source and correct the claims that no longer hold. For concepts without repo sources, spot-check the two or three most load-bearing claims" />
  <gate check="Where a doc and the code disagree, the code is right and the doc can be fixed" on-fail="say so and stop for direction" />
  <step name="CHECK THE WORKFLOW RECORDS">Check the workflow's records for lapsed record-keeping. Fix the record where the evidence is clear; report it where it is not:
- A feature plan with every slice ticked but `lifecycle` still `open`; an issue settled in prose but still `open`, or a document whose folder disagrees with its `lifecycle` (`superdev validate` names these).
- Gap issues still `open` against a `done` plan, or issues no plan or slice ever picked up.
- Backlog entries taken up but never moved out.
- The changelog's Unreleased section missing merged user-visible changes.</step>
  <step name="CHECK STRUCTURE">Check structure:
- No knowledge duplicated between concepts, or between the canonical knowledge and README/CONTRIBUTING: the concept summarises and cites via `sources`; detail lives in one home, cross-referenced.
- Misplaced content moves; a concept covering two unrelated things splits; near-empty concepts merge into a neighbour (keep the surviving file's `id`, re-point inbound `links` and index lines).
- Where prose in one concept leans on another's content, ensure a typed `links` entry with the right `rel` plus the mirroring body link. Prefer `id` targets; declare each edge once, from the more natural side.
- Each `description` is an accurate one-liner; update drifted ones and re-sync the `index.md` entry.</step>
  <step name="APPLY THE WORDING RULES" task="Apply the wording rules to every body you touched and skim the rest; tighten without losing warnings, caveats, or stated assumptions. Surgical changes only" />
  <gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
  </process_actions>


<rules>
<rule level="MUST NOT">add, edit, reorder, or delete a `verified` entry, even when rewriting the rest of the file; lapsed verification is reported, not edited</rule>
<rule level="MUST NOT">change an existing `id`</rule>
<rule level="MAY">assign an `id` to a concept that lacks one</rule>
<rule level="MUST NOT">write `generated` in a concept, or `producer`, `generated`, or `counts` in the manifest — those are export-time stamps</rule>
</rules>
</skill>
