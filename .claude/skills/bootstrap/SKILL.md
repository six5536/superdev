---
name: bootstrap
description: "Use after superdev init, when docs are stale or scattered, or when agents miss rules that live outside the knowledge bundle."
---

<skill name="bootstrap" purpose="Bootstrap the Knowledge Bundle" input="the sources or concepts to focus on, when given" user-input="$ARGUMENTS" output="the bundle, filled and PASSing, with a report: every source harvested (naming its concepts), reduced to summary-and-citation, or left with a reason; every skeleton filled or left TBD with the reason it must accrete">

<goal persona="technical writer">
You move the repo's stranded facts into the bundle at `knowledge/` and interview the owner for the rest.
</goal>

<constraints>
A harvest relocates one durable fact from a source into the bundle, leaving a one-line summary and a link behind in the source.
</constraints>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path=".agents/aokf/SPEC.md" when="before writing concepts" />
</bootstrap_actions>

<process_actions>
<step name="INVENTORY SOURCES" task="Inventory every prose document in the repo: README, CONTRIBUTING, `docs/`, in-repo wikis, and incumbent agent entry points (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md` and kin) that predate superdev. Skip generated files and the bundle itself. On a re-run, drop sources already summarised and cited" />
<step name="ASK ABOUT THE CODE SWEEP" task="Ask the owner: sweep the code too? If yes, add the comments that carry repo-wide conventions, decisions, and gotchas — never API-describing doc comments; those are the environment" />
<step name="HARVEST SOURCE BY SOURCE" task="Harvest source by source: split each into passages. Knowledge (a convention, decision, reason, or gotcha an agent cannot derive from the environment) lands in the bundle; environment cache (restating `package.json` scripts, `--help` output, file layout, API) stays in place" />
<step name="LAND SKELETON-FIRST" task="Land knowledge skeleton-first: replace the starter concepts' TBD prompts; create a new concept, frontmatter per the spec, only when nothing fits. Rewrite the source passage to a one-line summary plus a link; record the source in the concept's `sources`" />
<step name="TRIAGE MARGINAL PASSAGES" task="Passage neither knowledge nor environment cache (marketing prose, a stale claim, an ambiguous rule)? Ask the owner, or leave it with a reason in the report" />
<step name="RESOLVE INCUMBENT ENTRY POINTS" task="Incumbent entry point? Ask the owner: merge and reduce (durable content into the bundle, always-loaded rules into `AGENTS.md`, the incumbent down to its `@AGENTS.md` import), or leave as found" />
<step name="INTERVIEW THE OWNER" task="Interview the owner (`/grill-me`) on the skeletons still carrying TBD prompts that are answerable now: what the project is and its status, constraints and non-goals, first glossary terms, dependency policy, intended conventions. Land each answer in its skeleton" />
<step name="LEAVE ACCRETING TBDS" task="Leave the TBDs only development can fill (architecture as practised, testing strategy, error-handling conventions); they accrete through the project's own specs and decisions" />
<gate check="knowledge validates to PASS per the core knowledge block" on-fail="fix every error" />
</process_actions>


<rules>
<rule level="MUST NOT">import-only: a fact with two full homes drifts</rule>
<rule level="SHALL">ask decisions, act on mechanics: the owner decides whether to sweep the code, what to do with an incumbent entry point, and whether a marginal passage is knowledge</rule>
<rule level="SHALL">run clear-cut harvests without asking and land them as a reviewable diff</rule>
</rules>
</skill>
