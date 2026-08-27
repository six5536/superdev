---
name: integrate
description: "Superdev process: merge the slice once its verification has passed."
---

<skill name="integrate" purpose="Merge the Verified Slice" input="the slice, when not handed off" user-input="$ARGUMENTS" output="the slice merged, the changelog and the canonical knowledge brought up to date, and the slice ticked in the feature's plan">

<goal persona="integration manager">
You merge verified work into the shared branch and keep the project's records current. Integrate the slice specified in the input above once its verification has passed.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="aokf_read" id="feature-plan-{nnn}-{slug}" when="always" />
<tool_call name="aokf_read" id="development-procedure" when="always" />
<tool_call name="aokf_read" id="template-commit-message" when="before merging" />
<tool_call name="aokf_read" id="template-pr-description" when="if the convention is a PR" />
<tool_call name="aokf_read" id="template-changelog" when="if the change is user-visible" />
<tool_call name="aokf_read" id="template-migration-guide" when="if an interface change breaks users" />
<tool_call name="aokf_read" id="template-architecture" when="if a new concept is needed" />
<tool_call name="aokf_read" id="template-api-contracts" when="if a new concept is needed" />
<tool_call name="aokf_read" id="template-coding-standards" when="if a new concept is needed" />
</bootstrap_actions>

<process_actions>
<step name="UPDATE ONTO TARGET" task="The merge target moved since verify? Update the slice onto it again" />
<step name="RUN CHECKS" task="Run the full build, the linter, all integration tests, and a smoke test" />
<gate check="No conflict, and no check failed" on-fail="/build with the failure as input" />
<step name="MERGE" task="Merge the slice per the convention: target branch, PR or direct, required checks" />
<step name="UPDATE CHANGELOG" task="User-visible change? Add a line to the changelog's Unreleased section" />
<step name="UPDATE THE KNOWLEDGE" task="New convention, changed interface, or new term? Update the canonical knowledge so later slices follow it: the glossary for terms; a new concept starts from its schema (see the knowledge-concepts section of `knowledge/schemas/index.md`)" />
<step name="WRITE MIGRATION GUIDE" task="Interface change breaks users? Write the migration guide" />
<step name="MARK THE SLICE DONE" task="Mark the slice done in the feature's plan (`knowledge/feature-plans/`). Last slice? Tag the plan concept `done`" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<skill_call name="/build" when="if a next slice remains" input="the next slice" />
<skill_call name="/feature-plan" when="if the slice list needs re-cutting" />
</process_actions>


<rules>
<rule level="MUST NOT">write new code</rule>
<rule level="SHALL">record at merge time; later slices depend on it</rule>
</rules>
</skill>
