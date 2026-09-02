---
name: feature-plan
description: "Superdev process: cut the feature into buildable slices, once the contracts are clear."
---

<skill name="feature-plan" purpose="Cut the Feature into Buildable Slices" input="the feature-request or issue id, when not handed off" user-input="$ARGUMENTS" output="the feature plan, per `schema-feature-plan`">

<goal persona="project planner">
You decompose, you don't build. Produce the feature's plan as specified in the input above, following `schema-feature-plan`.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/feature-plan.md" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{kind}-{slug}" when="always" />
<tool_call name="sokf_read" id="{each contract the framed issue links}" when="always" />
<tool_call name="sokf_read" id="feature-plan-{nnn}-{slug}" when="if re-entering" />
<tool_call name="sokf_read" id="{gap-issue-id}" when="if re-entering" />
<tool_call name="sokf_search" query="{the feature}" when="if the issue id is not given" />
<tool_call name="codegraph_explore" when="before setting the sequence" />
</bootstrap_actions>

<process_actions>
<gate check="The framed issue's lifecycle is framed" on-fail="/frame — an unframed issue is framed before it is planned" />
<step name="CUT SLICES" task="Cut the feature — its acceptance criteria and any gap issues — into slices small enough to build and verify in one pass" />
<step name="ORDER SLICES" task="State each slice's `Depends-on`, then order the slices per `schema-feature-plan`: topologically — every slice after its dependencies — then a slice closing a contract-implementation gap before the slices that do not, and riskiest early among what is left. A forward reference is legal; adding a slice never renumbers the ones already written" />
<step name="GIVE DONE-CHECKS" task="Give each slice its own done-check" />
<step name="WRITE CASES" task="Write each slice's cases inline — `schema-feature-plan`'s slice rule says how a case names the criteria it covers, where an integration or e2e case sits, and that every criterion is covered" />
<step name="FILE THE PLAN" task="File the slice list as the feature's plan: an open concept (`plan-{nnn}-feature-{slug}`, `lifecycle: open`) per `schema-feature-plan`, listed in the plans index. Re-entering? Extend the existing plan" />
<step name="DOUBLE-CHECK" task="`/double-check` the plan; fix what it finds" />
<gate check="No slice is too big to build and verify in one pass" on-fail="cut it again" />
<gate check="The `Depends-on` graph has no cycle" on-fail="re-cut the slices until it has none" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<skill_call name="/build" when="always" input="the first slice" />
</process_actions>


<rules>
<rule level="MUST">decompose only</rule>
<rule level="MUST NOT">write code or design</rule>
<rule level="MUST">keep the plan current when this phase ends; build and integrate read it</rule>
</rules>
</skill>
