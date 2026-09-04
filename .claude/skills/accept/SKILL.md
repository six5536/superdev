---
name: accept
description: "Superdev process: acceptance on the merged code — run at the user's request, once the work has stopped changing."
---

<skill name="accept" purpose="Accept the Merged Change" input="the issue or plan id" user-input="$ARGUMENTS" output="the review's findings written up, an issue filed for every gap, and the issue `lifecycle: done` on a clean pass">

<goal persona="acceptance tester acting for the end user">
You judge the whole change on merged code, as the user will experience it. Acceptance is the user's step and the last one: it finds what a block's own tests cannot — blocks that do not work together, and regressions elsewhere in the app.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/issue.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/code-review.md" when="always" />
<tool_call name="sokf_read" id="plan-{nnn}-{slug}" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{slug}" when="if the plan delivers an issue" />
<tool_call name="sokf_read" id="{each contract the plan's Contract changes name}" when="always" />
<tool_call name="sokf_read" id="testing-strategy" when="always" />
<tool_call name="sokf_read" id="schema-security-review" when="if the change touches auth, input handling, or data exposure" />
<tool_call name="sokf_search" query="{the work}" when="if no id is given" />
</bootstrap_actions>

<process_actions>
<step name="REVIEW THE CHANGE" task="Review the whole change against the merge target for correctness and for simplifications (`/code-review`); wait for a background review with a blocking TaskOutput call and do not end the turn while it runs — the completion notification cannot wake a stopped subagent. Put every finding to the user: a finding the user wants fixed returns to `/build`, and nothing is fixed here" />
<step name="WRITE FINDINGS" task="Write the review's findings per `schema-code-review`" />
<step name="CHECK CRITERIA" task="Walk the criteria of every contract the change touched on the merged code — each promise, and each `AC_` criterion nested under it — confirming each is covered by a passing case, in the acceptance environment the `testing-strategy` concept names (CI e2e, simulator, staging, device, deployed service). CI's e2e and regression results count; do not repeat what CI has run. Drive the app with `/run` for the manual checks" />
<step name="CHECK DOCUMENTATION" task="Check the project's user documentation describes the change, as the issue's Behaviour section states it" />
<step name="SECURITY REVIEW" task="Run `/security-review` when the change touches auth, input handling, or data exposure" />
<step name="FILE GAPS" task="File each gap found per `schema-issue`: an issue `issue-{nnn}-{slug}` carrying its `kind` and `lifecycle: open`, linked to the issue this change delivered; `/scope` takes it up" />
<gate check="The change is documented" on-fail="file the gap" />
<gate check="No gap found is left unfiled" on-fail="file it" />
<gate check="No contract the change touched still carries `PENDING`, uppercase and whole, in its Behaviour or Stability section" on-fail="file the unbuilt promise as a gap — a prose promise may run ahead of its code while the work runs, never once it settles (ADR-044)" />
<step name="CLOSE OUT" task="Clean pass? Set the issue's `lifecycle` to `done` and write its Resolution section per `schema-issue`, and confirm the plan already reads `done` — build sets it after the last block; `superdev validate --fix` refiles both" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<skill_call name="/build" when="if the user wants a finding fixed now" input="the finding" />
</process_actions>


<rules>
<rule level="MUST">run manual checks only where the `testing-strategy` concept says automation cannot reach</rule>
<rule level="MUST NOT">fix a finding or a gap here</rule>
<rule level="MUST NOT">run unasked; the user invokes acceptance, once the work has stopped changing</rule>
<rule level="SHALL NOT">release as part of acceptance; the release follows the `release-procedure` concept</rule>
</rules>
</skill>
