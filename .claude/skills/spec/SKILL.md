---
name: spec
description: "Superdev process: use to describe the feature from the outside, as the user sees it once the framing is clear."
---

<skill name="spec" purpose="Create or Update a Feature Spec" input="the feature to specify, when not handed off from frame" user-input="$ARGUMENTS" output="the spec, per `schema-spec`, and its test plan">

<goal persona="requirements analyst">
You describe the feature from outside, as a user or caller sees it. Create or update the feature spec as specified in the input above, following `schema-spec`.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/spec.md" when="always" />
<tool_call name="aokf_overview" when="always" />
<tool_call name="aokf_read" id="glossary" when="always" />
<tool_call name="aokf_search" query="{prior specs and conventions this spec must not contradict}" when="always" />
</bootstrap_actions>

<process_actions>
<step name="DRAFT THE SPEC" task="Create a draft concept at `knowledge/specs/spec-{nnn}-{feature-slug}.md` per `schema-spec`; list it in the specs index" />
<step name="DESCRIBE BEHAVIOUR" task="Describe the observable behaviour" />
<step name="WRITE ACCEPTANCE CRITERIA" task="Write acceptance criteria as pass/fail checks (given/when/then)" />
<step name="COVER FAILURE" task="State the expected behaviour for bad input and failure, not just the happy path" />
<step name="BOUND SCOPE" task="State what is out of scope" />
<step name="LIST UI STATES" task="For UI work, enumerate the states per `schema-spec`" />
<step name="APPEND THE TEST PLAN" task="Add the test-plan sections per `schema-spec`" />
<step name="INTERVIEW THE USER" task="`/grill-me`: resolve every criterion or behaviour readable two ways until one reading remains" />
<step name="DOUBLE-CHECK" task="`/double-check` the spec and test plan; fix what it finds" />
<gate check="Verify and accept can check every criterion pass/fail without interpretation" on-fail="rework the criterion" />
<gate check="The spec contradicts no prior spec or convention" on-fail="report the conflict; never override it" />
<gate check="knowledge validates to PASS per the core knowledge block" on-fail="fix every error" />
<skill_call name="/interface-design" when="always" />
</process_actions>


<rules>
<rule level="SHALL">describe behaviour in the project's terms as defined in the glossary</rule>
<rule level="SHALL">say what the feature does</rule>
<rule level="MUST NOT">describe implementation</rule>
<rule level="SHALL">treat the spec as a working document: its criteria become the tests, its decisions become ADRs at interface design, and it is tagged `done` at accept</rule>
<rule level="SHALL NOT">maintain the spec as documentation</rule>
<rule level="SHALL">write the behaviour description as the draft of the user documentation; accept uses it</rule>
</rules>
</skill>
