---
name: integrate
description: "Superdev process: verify the slice against its cases and the contracts, then merge it and bring the records up to date."
---

<skill name="integrate" purpose="Verify and Merge the Slice" input="the slice, when not handed off" user-input="$ARGUMENTS" output="the slice verified against its cases and merged, the changelog and the canonical knowledge brought up to date, and the slice ticked in the feature's plan">

<goal persona="integration manager">
First you try to make the slice fail; only what survives is merged. Verify the slice given in the input above against its cases and the contracts, then integrate it and keep the project's records current.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/code-review.md" when="if the last slice" />
<tool_call name="read_file" path="knowledge/schemas/investigation.md" when="if a failure needs investigation" />
<tool_call name="sokf_read" id="feature-plan-{nnn}-{slug}" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{kind}-{slug}" when="always" />
<tool_call name="sokf_graph" id="issue-{nnn}-{kind}-{slug}" when="always" />
<tool_call name="sokf_read" id="{each contract the framed issue links}" when="always" />
<tool_call name="sokf_read" id="definition-of-done" when="always" />
<tool_call name="sokf_read" id="development-procedure" when="always" />
<tool_call name="sokf_read" id="schema-changelog" when="if the change is user-visible" />
<tool_call name="sokf_read" id="schema-migration-guide" when="if a contract change breaks users" />
</bootstrap_actions>

<process_actions>
<step name="UPDATE ONTO THE MERGE TARGET" task="Update the slice onto the merge target named by the `development-procedure` concept; every check below runs on that state" />
<gate check="The update onto the merge target is conflict-free" on-fail="/build with the conflict as input" />
<step name="RUN CHECKS" task="Run the full build, tests, typecheck and lint" />
<step name="RUN THE SLICE'S CASES" task="Run the slice's cases from the plan, including manual ones, and confirm each covers the criteria it names" />
<step name="CHECK THE DONE-CHECK" task="Check the diff against the slice's done-check" />
<step name="CHECK CONTRACTS" task="Check the diff's interfaces against the contracts the framed issue links" />
<step name="REVIEW THE DIFF" when="if the last slice" task="Review the whole feature diff against the merge target for correctness and for simplifications (`/code-review`); wait for a background review with a blocking TaskOutput call and do not end the turn while it runs — the completion notification cannot wake a stopped subagent; findings return to build, they are not applied here" />
<step name="CHECK RENDERED UI" task="UI: check the rendered result (`/run`)" />
<step name="WRITE FINDINGS" when="if the last slice" task="Write the review's findings per `schema-code-review`" />
<step name="WRITE INVESTIGATION" task="Write an investigation per `schema-investigation` for a failure that needs one" />
<gate check="Every case in the slice has an implemented test, or the plan marks it manual" on-fail="/build" />
<gate check="No acceptance criterion is ambiguous or wrong" on-fail="/frame — the criterion lives in the issue" />
<gate check="No case is ambiguous or wrong" on-fail="/feature-plan — the case lives in the plan" />
<gate check="The diff does not diverge from the contracts" on-fail="/build; a divergence a contract should adopt returns to /contract-design" />
<gate check="Every check passed and the done-check is met" on-fail="/build with the failure as input" />
<step name="MERGE" task="Merge the slice per the convention the `development-procedure` concept states: target branch, commit style, PR or direct, required checks" />
<step name="UPDATE CHANGELOG" task="User-visible change? Add a line to the changelog's Unreleased section" />
<step name="UPDATE THE KNOWLEDGE" task="New convention, changed interface, or new term? Update the canonical knowledge so later slices follow it: the glossary for terms; a new concept starts from its schema (see the knowledge-concepts section of `knowledge/schemas/index.md`)" />
<step name="WRITE MIGRATION GUIDE" task="Contract change breaks users? Write the migration guide" />
<step name="MARK THE SLICE DONE" task="Mark the slice done in the feature's plan concept. Last slice? Set the plan's `lifecycle` to `done`; `superdev validate --fix` refiles it" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<step name="COMMIT THE RECORDS" task="Commit the changelog, knowledge and plan edits this integrate made, per the `development-procedure` concept's commit convention — after the merge, so a failed check commits nothing" />
<skill_call name="/build" when="if a next slice remains" input="the next slice" />
<skill_call name="/feature-plan" when="if the slice list needs re-cutting" />
</process_actions>


<rules>
<rule level="MUST NOT">change the code while verifying; findings, simplifications included, return to build</rule>
<rule level="SHALL">report failures with their output</rule>
<rule level="SHALL">record at merge time; later slices depend on it</rule>
</rules>
</skill>
