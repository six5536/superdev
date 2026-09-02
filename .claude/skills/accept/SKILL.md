---
name: accept
description: "Superdev process: feature-level acceptance on the merged code — run at the user's request, once the feature has stopped changing."
---

<skill name="accept" purpose="Run Feature-Level Acceptance on the Merged Code" input="the framed issue or plan id" user-input="$ARGUMENTS" output="issues filed for every gap found, and the feature's plan confirmed `lifecycle: done` on a clean pass">

<goal persona="acceptance tester acting for the end user">
You judge the whole feature on merged code, as the user will experience it. This finds what slice-level verification in integrate cannot: slices that do not work together, and regressions elsewhere in the app.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/bug-report.md" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{kind}-{slug}" when="always" />
<tool_call name="sokf_read" id="testing-strategy" when="always" />
<tool_call name="sokf_read" id="schema-security-review" when="if the feature touches auth, input handling, or data exposure" />
<tool_call name="sokf_search" query="{the feature}" when="if the issue id is not given" />
</bootstrap_actions>

<process_actions>
<step name="CHECK CRITERIA" task="Walk every acceptance criterion in the framed issue end to end on the merged code, confirming each is covered by a passing case, in the acceptance environment the `testing-strategy` concept names (CI e2e, simulator, staging, device, deployed service). CI's e2e and regression results count; do not repeat what CI has run. Drive the app with `/run` for the manual checks" />
<step name="CHECK DOCUMENTATION" task="Check the project's user documentation describes the feature, as the framed issue's proposed behaviour does" />
<step name="SECURITY REVIEW" task="Run `/security-review` when the feature touches auth, input handling, or data exposure" />
<step name="FILE GAPS" task="File each gap found per `schema-bug-report`: a BugReport concept `issue-{nnn}-bug-{slug}` (`lifecycle: open`), linked to the framed issue" />
<gate check="The feature is documented" on-fail="file the gap" />
<gate check="No gap is left without a slice" on-fail="make each gap a new slice" />
<gate check="No contract the feature touched still carries `PENDING`, uppercase and whole, in its Behaviour or Stability section" on-fail="file the unbuilt promise as a gap — a prose promise may run ahead of its code while a feature runs, never once it settles (ADR-044)" />
<step name="CLOSE OUT" task="Clean pass? Set the framed issue's `lifecycle` to `done` and confirm the plan already reads `done` (integrate sets it at the last slice); `superdev validate --fix` refiles both" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<skill_call name="/feature-plan" when="if gaps found" input="the gap issues" />
</process_actions>


<rules>
<rule level="MUST">run manual checks only where the test plan says automation cannot reach</rule>
<rule level="MUST NOT">fix a gap here</rule>
<rule level="SHALL NOT">release as part of acceptance; the release follows the `release-procedure` concept</rule>
</rules>
</skill>
