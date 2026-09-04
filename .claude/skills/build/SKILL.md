---
name: build
description: "Superdev process: use to work a plan's blocks in order — tests, then code — and to verify, record and merge the whole change after the last block."
---

<skill name="build" purpose="Work the Plan's Blocks and Land the Change" input="the plan, when not handed off" user-input="$ARGUMENTS" output="every block committed with its tests, the whole change verified once, the changelog and the canonical knowledge current, the change merged on its branch, and the plan `lifecycle: done`">

<goal persona="lead software engineer">
You build the plan given in the input above, one work block at a time and nothing beyond it, against the plan's cases and the contracts its Contract changes name.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="sokf_read" id="plan-{nnn}-{slug}" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{slug}" when="if the plan delivers an issue" />
<tool_call name="sokf_read" id="{each contract the plan's Contract changes name}" when="always" />
<tool_call name="sokf_read" id="coding-standards" when="before writing code and tests" />
<tool_call name="sokf_read" id="testing-strategy" when="before writing code and tests" />
<tool_call name="sokf_read" id="visual-system" when="if the block is UI work" />
<tool_call name="sokf_read" id="development-procedure" when="before committing and before merging" />
<tool_call name="sokf_read" id="definition-of-done" when="before verifying the whole change" />
<tool_call name="sokf_read" id="schema-changelog" when="if the change is user-visible" />
<tool_call name="sokf_read" id="schema-migration-guide" when="if a contract change breaks users" />
<tool_call name="read_file" path="knowledge/schemas/investigation.md" when="if a failure needs investigation" />
<tool_call name="codegraph_explore" query="{the code under change and its callers}" when="before editing" />
</bootstrap_actions>

<process_actions>
<gate check="The working tree is on the work's branch, with the plan committed on it" on-fail="/scope — the branch and the plan are cut there" />
<loop until="every block's Done is ticked">
<step name="PICK THE BLOCK" task="Take the first unticked block whose `Depends-on` blocks are all ticked" />
<step name="IMPLEMENT WITH TDD" task="Write the block's cases as tests, watch them fail, then write the code that passes them. Write tests with the code only where TDD is impractical, e.g. exploratory UI work" />
<gate check="The block needs no contract change" on-fail="/scope — never diverge from a contract" />
<gate check="The block is small enough to build and commit in one pass" on-fail="/scope — the blocks are cut there" />
<step name="RUN THE BLOCK'S TESTS" task="Run the tests this block wrote and the tests its change touches; fix every failure before committing, and the full suite waits for the last block" />
<step name="CHECK THE DONE-CHECK" task="Check the diff against the block's Done-check; a check it fails is fixed here, in this block" />
<gate check="The diff contains nothing outside the block" on-fail="remove what is outside it" />
<step name="TICK AND COMMIT THE BLOCK" task="Tick the block's Done in the plan — `- [x] Done — ticked by build at its commit.` — and commit the code, the tests and the plan together, per the `development-procedure` concept's commit convention" />
</loop>
<step name="UPDATE ONTO THE MERGE TARGET" task="Update the branch onto the merge target named by the `development-procedure` concept; every check below runs on that state" />
<gate check="The update onto the merge target is conflict-free" on-fail="resolve the conflict on the branch and update again" />
<step name="VERIFY THE WHOLE CHANGE" task="Run the full build, the whole test suite, the typecheck, the lint and `superdev validate` — once, after the last block, on the updated state" />
<step name="CHECK CONTRACTS" task="Check the diff's interfaces against every contract the plan's Contract changes name" />
<step name="JUDGE THE CONTRACTS" task="Plan touched a contract? Read each one as its consumer would and report, per contract, what you checked and where it falls short: where an included region omits part of the promised surface; where an optional Behaviour section that `schema-contract`'s checklist names for the kind is absent with no reason given; where a reader could not learn the interface from the document. The report is a judgement, not a validator finding, and blocks nothing. No contract touched? Say so and report nothing further" />
<step name="CHECK RENDERED UI" task="UI: check the rendered result (`/run`)" />
<step name="WRITE INVESTIGATION" task="Write an investigation per `schema-investigation` for a failure that needs one" />
<gate check="Every case in the plan has an implemented test, or the plan marks it manual" on-fail="write the missing test" />
<gate check="The diff does not diverge from the contracts" on-fail="fix the code; a divergence a contract should adopt returns to /scope" />
<gate check="Every check passed and every block's done-check is met" on-fail="fix it on the branch and verify again" />
<step name="UPDATE CHANGELOG" task="User-visible change? Add a line to the changelog's Unreleased section" />
<step name="UPDATE THE KNOWLEDGE" task="New convention, changed interface, or new term? Update the canonical knowledge: the glossary for terms; a new concept starts from its schema (see the knowledge-concepts section of `knowledge/schemas/index.md`)" />
<step name="WRITE MIGRATION GUIDE" task="Contract change breaks users? Write the migration guide" />
<step name="MERGE" task="Merge per the convention the `development-procedure` concept states: target branch, commit style, PR or direct, required checks" />
<step name="CLOSE THE PLAN" task="Set the plan's `lifecycle` to `done`, every block ticked; `superdev validate --fix` refiles it" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<step name="COMMIT THE RECORDS" task="Commit the changelog, knowledge and plan edits this build made, per the `development-procedure` concept's commit convention — after the merge, so a failed check commits nothing" />
</process_actions>


<rules>
<rule level="SHALL">keep each block's change small</rule>
<rule level="SHALL">deliver code and tests as one deliverable</rule>
<rule level="MUST NOT">deliver code alone</rule>
<rule level="SHALL">run the full verification once, after the last block; a block runs its own tests and the tests it touches (ADR-050)</rule>
<rule level="SHALL">report failures with their output</rule>
<rule level="SHALL">treat the contract as binding per the core's `core_principles` block</rule>
</rules>
</skill>
