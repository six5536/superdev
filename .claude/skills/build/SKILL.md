---
name: build
description: "Superdev process: use to implement exactly one slice against the framed issue and the contracts, once both are clear."
---

<skill name="build" purpose="Implement Exactly One Slice" input="the slice, when not handed off" user-input="$ARGUMENTS" output="a small, committed diff with passing tests">

<goal persona="lead software engineer">
You build exactly one slice and nothing beyond it. Implement the slice identified in the input above, against the framed issue's criteria and the contracts it links.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="sokf_read" id="feature-plan-{nnn}-{slug}" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{kind}-{slug}" when="always" />
<tool_call name="sokf_read" id="{each contract the framed issue links}" when="always" />
<tool_call name="sokf_read" id="coding-standards" when="before writing code and tests" />
<tool_call name="sokf_read" id="testing-strategy" when="before writing code and tests" />
<tool_call name="sokf_read" id="visual-system" when="if UI slice" />
<tool_call name="sokf_read" id="development-procedure" when="before committing" />
<tool_call name="codegraph_explore" query="{the code under change and its callers}" when="before editing" />
</bootstrap_actions>

<process_actions>
<step name="IMPLEMENT WITH TDD" task="Implement the slice using TDD, against the cases its plan entry carries. Write tests with the code only where TDD is impractical, e.g. exploratory UI work" />
<gate check="The implementation needs no contract change" on-fail="/contract-design — never diverge from a contract" />
<gate check="The slice is small enough to build in one pass" on-fail="/feature-plan" />
<step name="RUN THE TESTS" task="Run the tests you wrote and the affected existing tests; fix failures before handing off" />
<gate check="The diff contains nothing outside the slice" on-fail="remove what is outside it" />
<step name="COMMIT THE SLICE" task="Commit the slice; write the commit message per the `development-procedure` concept's convention" />
<skill_call name="/integrate" when="always" />
</process_actions>


<rules>
<rule level="SHALL">keep the change small</rule>
<rule level="SHALL">deliver code and tests as one deliverable</rule>
<rule level="MUST NOT">deliver code alone</rule>
<rule level="SHALL">treat the contract as binding per the core's `core_principles` block</rule>
</rules>
</skill>
