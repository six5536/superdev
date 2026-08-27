---
name: accept
description: "Superdev process: feature-level acceptance on the merged code — run at the user's request, once the feature has stopped changing."
---

<skill name="accept" purpose="Run Feature-Level Acceptance on the Merged Code" input="the feature or spec id" user-input="$ARGUMENTS" output="issues filed for every gap found, and the spec tagged `done` on a clean pass">

<goal persona="acceptance tester acting for the end user">
You judge the whole feature on merged code, as the user will experience it. This finds what slice-level verify cannot: slices that do not work together, and regressions elsewhere in the app.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/bug-report.md" when="always" />
<tool_call name="aokf_read" id="spec-{nnn}-{feature-slug}" when="always" />
<tool_call name="aokf_read" id="testing-strategy" when="always" />
<tool_call name="aokf_read" id="template-security-review" when="if the feature touches auth, input handling, or data exposure" />
<tool_call name="aokf_search" query="{the feature or spec}" when="if the spec id is not given" />
</bootstrap_actions>

<process_actions>
<step name="CHECK CRITERIA" task="Check every acceptance criterion end to end on the merged code, following the spec's test plan, in the acceptance environment the `testing-strategy` concept names (CI e2e, simulator, staging, device, deployed service). CI's e2e and regression results count; do not repeat what CI has run. Drive the app with `/run` for the manual checks" />
<step name="CHECK DOCUMENTATION" task="Check the project's user documentation describes the feature, as the spec's behaviour description does" />
<step name="SECURITY REVIEW" task="Run `/security-review` when the feature touches auth, input handling, or data exposure" />
<step name="FILE GAPS" task="File each gap found per `schema-bug-report`: an Issue concept at `knowledge/issues/issue-{nnn}-{slug}.md`, linked to the spec" />
<gate check="The feature is documented" on-fail="file the gap" />
<gate check="No gap is left without a slice" on-fail="make each gap a new slice" />
<step name="TAG DONE" task="Clean pass? Tag the spec concept `done`" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<skill_call name="/feature-plan" when="if gaps found" input="the gap issues" />
</process_actions>


<rules>
<rule level="MUST">run manual checks only where the test plan says automation cannot reach</rule>
<rule level="MUST NOT">fix a gap here</rule>
<rule level="SHALL NOT">release as part of acceptance; the release follows the `release-procedure` concept</rule>
</rules>
</skill>
