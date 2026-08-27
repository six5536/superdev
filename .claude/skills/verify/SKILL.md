---
name: verify
description: "Superdev process: check that this slice works as intended, against the spec and interface contract."
---

<skill name="verify" purpose="Verify a Slice" input="the slice to verify, when not handed off" user-input="$ARGUMENTS" output="a code review of the slice, with an investigation where a failure needs one">

<goal persona="QA engineer">
You try to make the slice fail. Check that the slice given in the input above works as intended, against the spec and the interface contract.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/code-review.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/investigation.md" when="if a failure needs investigation" />
<tool_call name="sokf_read" id="feature-plan-{nnn}-{slug}" when="always" />
<tool_call name="sokf_read" id="spec-{nnn}-{feature-slug}" when="always" />
<tool_call name="sokf_read" id="contract-{feature-slug}" when="always" />
<tool_call name="sokf_read" id="definition-of-done" when="always" />
<tool_call name="sokf_read" id="development-procedure" when="always" />
</bootstrap_actions>

<process_actions>
<step name="UPDATE ONTO THE MERGE TARGET" task="Update the slice onto the merge target named by the `development-procedure` concept; every check below runs on that state" />
<gate check="The update onto the merge target is conflict-free" on-fail="/build with the conflict as input" />
<step name="RUN CHECKS" task="Run tests, typecheck, and lint" />
<step name="RUN TEST-PLAN CASES" task="Run the slice's assigned test-plan cases, including manual checks" />
<step name="CHECK THE DONE-CHECK" task="Check the diff against the slice's done-check" />
<step name="CHECK INTERFACES" task="Check the diff's interfaces against the interface contract" />
<step name="REVIEW THE DIFF" task="Review the diff for correctness and for simplifications (`/code-review`); simplifications return to build as findings, they are not applied here" />
<step name="CHECK RENDERED UI" task="UI: check the rendered result (`/run`)" />
<step name="WRITE FINDINGS" task="Write findings per `schema-code-review`; use `schema-investigation` for a failure that needs investigation" />
<gate check="Every assigned test-plan case has an implemented test, or the test plan marks it manual" on-fail="/build" />
<gate check="No test-plan case or criterion is ambiguous or wrong" on-fail="/spec" />
<gate check="The diff does not diverge from the interface contract" on-fail="/build; a divergence the contract should adopt returns to /interface-design" />
<gate check="Every check passed and the done-check is met" on-fail="/build with the failure as input" />
<skill_call name="/integrate" when="always" />
</process_actions>


<rules>
<rule level="MUST NOT">change anything here</rule>
<rule level="SHALL">return findings, including simplifications, to build</rule>
<rule level="SHALL">report failures with their output</rule>
</rules>
</skill>
