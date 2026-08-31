---
name: adhoc-plan
description: "Superdev process: plan one-off work that does not go through the feature workflow."
---

<skill name="adhoc-plan" purpose="Plan One-off Work Outside the Feature Workflow" input="the work to plan" user-input="$ARGUMENTS" output="an ad-hoc plan filed as a concept; the work follows its steps, and the plan's `lifecycle` is set to `done` when it lands">

<goal persona="project planner">
You plan one piece of work outside the feature workflow — a refactor, a migration, a chore. Plan the work given in the input above, following `schema-adhoc-plan`.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/adhoc-plan.md" when="always" />
<tool_call name="sokf_overview" when="always" />
<tool_call name="sokf_search" query="{the conventions and constraints the plan must respect}" when="always" />
<tool_call name="codegraph_explore" query="{the affected code and its callers}" when="before setting the steps" />
</bootstrap_actions>

<process_actions>
<gate check="The work needs no feature framing and changes no contract that is expensive to change" on-fail="/frame — it is a feature" />
<step name="DRAFT THE PLAN" task="Draft the plan per `schema-adhoc-plan`" />
<step name="ORDER THE STEPS" task="Order the steps so the codebase stays working after each one where possible" />
<step name="INTERVIEW THE USER" task="`/grill-me`: resolve the open questions and the risks that need their judgement" />
<step name="BRANCH" task="When the planned work touches code, create `adhoc/<slug>` off the default branch and switch to it; a documentation-only plan stays where it is" />
<step name="FILE THE PLAN" task="File the plan as an open concept (`plan-{nnn}-adhoc-{slug}`, `lifecycle: open`) per `schema-adhoc-plan`, listed in the plans index; `superdev validate --fix` places the file" />
<step name="DOUBLE-CHECK" task="`/double-check` the plan; fix what it finds" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
</process_actions>


<rules>
<rule level="MUST">plan only: no code</rule>
</rules>
</skill>
