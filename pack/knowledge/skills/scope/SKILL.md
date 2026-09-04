---
name: scope
description: "Superdev process: use on a filed issue or a one-off piece of work — cut the branch, make the contract changes and write the plan /build works."
---

<skill name="scope" purpose="Scope the Work into a Plan" input="the issue's id, or the one-off work to scope" user-input="$ARGUMENTS" output="the work's branch, the contract changes it makes, and the plan per `schema-plan`, committed on that branch">

<goal persona="technical lead">
You decide how the work is cut and which interfaces it moves, and you build none of it. Scope the issue or the one-off work given in the input above into one plan, per `schema-plan`.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/plan.md" when="always" />
<tool_call name="sokf_overview" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{slug}" when="if an issue is given" />
<tool_call name="sokf_search" query="{the work, in the user's words}" when="if no issue is given" />
<tool_call name="sokf_read" id="{each contract the work touches}" when="always" />
<tool_call name="sokf_read" id="glossary" when="always" />
<tool_call name="sokf_read" id="development-procedure" when="before branching and before committing" />
<tool_call name="sokf_read" id="project-overview" when="always" />
<tool_call name="sokf_read" id="constraints-non-goals" when="always" />
<tool_call name="codegraph_explore" query="{the code under change and its callers}" when="before cutting the blocks" />
</bootstrap_actions>

<process_actions>
<gate check="Project knowledge is initialised, not TBD" on-fail="/bootstrap" />
<gate check="The work is not already decided against or out of scope" on-fail="tell the user why, and scope nothing" />
<gate check="The work is shaped enough to state a goal" on-fail="/brainstorm, then scope its shortlist" />
<step name="BRANCH" task="Create the work's branch off the default branch and switch to it — the `development-procedure` concept's convention wins; where it names none, use `feature/<nnn>-<slug>`, `<nnn>` the issue's number, for work an issue asks for, and `adhoc/<nnn>-<slug>`, `<nnn>` the plan's number, for one-off work with no issue, and record the convention in that concept" />
<step name="INTERVIEW THE USER" task="`/grill-me` where the design is open: resolve the open decisions, the gaps and every reading of the intent that competes with another, until one reading remains. The interview is an interaction, never a self-check" />
<step name="ESTABLISH EXTERNAL FACTS" task="Does the design rest on a third-party API or another external fact? Establish it with `/research`; the findings land in the canonical knowledge" />
<step name="MOCK THE UI" task="UI: a mockup (`/design`; `/frontend-design` for the visual direction) or a throwaway prototype (`/prototype`). Discard the prototype and build against what it settled" />
<step name="DECIDE THE CONTRACT CHANGES" task="Decide what the work changes that is expensive to change once other code depends on it — data schema, API contracts, module boundaries, auth surface, UI — and name each contract it touches with the promises and criteria it adds, changes or withdraws, as the plan's Contract changes section carries them. Work that touches no contract carries the single bullet 'none'" />
<skill_call name="/contract-design" when="if the contract changes name a contract" input="the contract changes" />
<step name="WRITE THE PLAN" task="Write the plan per `schema-plan` — Goal, Contract changes, Work blocks, Deferred decisions — as `plan-{nnn}-{slug}`, `lifecycle: open`, listed in the plans index; `superdev validate --fix` places the file. Cut the Work blocks small enough to build and commit in one pass, order them by the schema's rule, give each its Depends-on, its done-check and its cases, and cite by key the contract criteria a case covers. Re-entering? Extend the plan on file" />
<step name="RECORD THE DECISIONS" task="Record the decisions the plan rests on: promote an idea taken up (`/file`); file a rejected idea as a `wontfix` issue with the reasoning; add a term the project will keep to the glossary" />
<loop until="the check finds nothing left to fix" max="3">
<step name="DOUBLE-CHECK" task="`/double-check` the plan; fix what it finds" />
</loop>
<gate check="No block is too big to build and commit in one pass" on-fail="cut it again" />
<gate check="The `Depends-on` graph has no cycle" on-fail="re-cut the blocks until it has none" />
<gate check="The plan contradicts no convention and no contract" on-fail="report the conflict; never override it" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<step name="COMMIT THE PLAN" task="Commit the plan and the contract, source-declaration and decision-record edits `/contract-design` made, on the work's branch, per the `development-procedure` concept's commit convention" />
<skill_call name="/build" when="always" input="the plan" />
</process_actions>

<rules>
<rule level="MUST">decompose the work and decide its interfaces; write no product code</rule>
<rule level="SHALL">ask the user every question only the user can answer; where the user is not there, record each one under the plan's Deferred decisions, naming the block it blocks or "blocks nothing"</rule>
<rule level="SHALL">describe the work in the project's terms as the `glossary` defines them</rule>
<rule level="SHALL">keep the plan current when this phase ends; build works its blocks and accept reads it</rule>
</rules>
</skill>
