---
name: feature-plan
description: "Superdev process: cut the feature into buildable slices, once the interface is clear."
---

<skill name="feature-plan" purpose="Cut the Feature into Buildable Slices" input="the feature or spec id, when not handed off" user-input="$ARGUMENTS" output="the feature plan, per `schema-feature-plan`">

<goal persona="project planner">
You decompose, you don't build. Produce the feature's plan as specified in the input above, following `schema-feature-plan`.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/feature-plan.md" when="always" />
<tool_call name="sokf_read" id="spec-{nnn}-{feature-slug}" when="always" />
<tool_call name="sokf_read" id="contract-{feature-slug}" when="always" />
<tool_call name="sokf_read" id="feature-plan-{nnn}-{slug}" when="if re-entering" />
<tool_call name="sokf_read" id="{gap-issue-id}" when="if re-entering" />
<tool_call name="sokf_search" query="{the feature or spec}" when="if the spec id is not given" />
<tool_call name="codegraph_explore" when="before setting the sequence" />
</bootstrap_actions>

<process_actions>
<step name="CUT SLICES" task="Cut the spec — and any gap issues — into slices small enough to build and verify in one pass" />
<step name="ORDER SLICES" task="Order the slices per `schema-feature-plan`: dependency first, then risk" />
<step name="GIVE DONE-CHECKS" task="Give each slice its own done-check" />
<step name="ASSIGN CASES" task="Assign each of the spec's test-plan cases to a slice per `schema-feature-plan`" />
<step name="FILE THE PLAN" task="File the slice list as the feature's plan: a draft concept in `knowledge/plans/` per `schema-feature-plan`, listed in that directory's index. Re-entering? Extend the existing plan" />
<step name="DOUBLE-CHECK" task="`/double-check` the plan; fix what it finds" />
<gate check="No slice is too big to build and verify in one pass" on-fail="cut it again" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<skill_call name="/build" when="always" input="the first slice" />
</process_actions>


<rules>
<rule level="MUST">decompose only</rule>
<rule level="MUST NOT">write code or design</rule>
<rule level="MUST">keep the plan current when this phase ends; build, verify, and integrate read it</rule>
</rules>
</skill>
