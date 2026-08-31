---
name: frame
description: "Superdev process: use at the start of a feature or project — frame the problem and record it as a tracker issue with concrete acceptance criteria."
---

<skill name="frame" purpose="Frame a Project or Feature" input="New project or feature (existing project) to frame" user-input="$ARGUMENTS" output="the framed issue: goal, user, constraints and EARS acceptance criteria, filed in the tracker">

<goal persona="product manager">
You define the problem, not the solution, and you describe done from outside, as a user or caller sees it. Frame the project or feature given in the input above and record it as a tracker issue.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/feature-request.md" when="if framing a feature" />
<tool_call name="read_file" path="knowledge/schemas/bug-report.md" when="if framing a bug fix" />
<tool_call name="read_file" path="knowledge/schemas/chore.md" when="if framing a chore" />
<tool_call name="sokf_overview" when="always" />
<tool_call name="sokf_search" query="{existing knowledge on the project or feature}" when="always" />
<tool_call name="sokf_read" id="glossary" when="always" />
<tool_call name="sokf_read" id="issue-tracker" when="unless a new project" />
<tool_call name="sokf_read" id="project-overview" when="if framing a feature" />
<tool_call name="sokf_read" id="constraints-non-goals" when="if framing a feature" />
<tool_call name="sokf_read" id="backlog" when="if framing a feature" />
<tool_call name="sokf_read" id="technology-stack" when="if framing a feature" />
<tool_call name="sokf_read" id="visual-system" when="if framing a feature" />
<tool_call name="sokf_read" id="schema-readme" when="if new project" />
<tool_call name="sokf_read" id="schema-project-overview" when="if new project" />
<tool_call name="sokf_read" id="schema-technology-stack" when="if new project" />
<tool_call name="sokf_read" id="schema-constraints-non-goals" when="if new project" />
<tool_call name="sokf_read" id="schema-visual-system" when="if new project" />
<tool_call name="codegraph_explore" query="{existing code}" when="if relevant" />
</bootstrap_actions>

<process_actions>
<gate check="Project knowledge is initialised, not TBD" on-fail="/bootstrap" />
<gate check="The feature is not already decided against or out-of-scope" on-fail="tell the user why" />
<gate check="The idea is well shaped, so a goal can be stated" on-fail="/brainstorm, then frame its shortlist" />
<step name="STATE THE FRAME" task="State the goal, the user, and the constraints, using the `glossary`'s terms" />
<step name="FILE OR FETCH THE ISSUE" task="Create the tracker issue (`lifecycle: open`) per its kind's schema — a feature-request for something absent, a bug-report for a defect, a chore for known-shape work — or fetch the existing one; `superdev validate --fix` places the file" />
<step name="BRANCH" task="Create the feature's branch off the default branch and switch to it — the `development-procedure` concept's convention wins; where it names none, use `feature/<slug>` and record the convention in that concept" />
<step name="DESCRIBE BEHAVIOUR" task="Describe the proposed behaviour from outside — what a user sees or a caller gets — as the draft of the user documentation; accept uses it" />
<step name="WRITE ACCEPTANCE CRITERIA" task="Write the Acceptance criteria as numbered EARS sentences, each opening with its pattern tag: [event] WHEN x THE SYSTEM SHALL y. Each checkable pass/fail without interpretation. A bug's repro steps and expected behaviour are its criteria; a chore's definition of done is" />
<step name="COVER FAILURE" task="State the expected behaviour for bad input and failure, not just the happy path — as criteria or in the proposed behaviour" />
<step name="BOUND SCOPE" task="State what is in scope and, separately, what is deliberately out" />
<step name="CHOOSE TECH STACK" when="if new project" task="Choose the tech stack — `/research` settles an open technology question"/>
<step name="CHOOSE VISUAL SYSTEM" when="if new project" task="set the visual system with `/frontend-design`" />
<step name="CREATE README AND KNOWLEDGE" when="if new project" task="create the README and the canonical knowledge, each document from its schema's contract and worked example" />
<step name="INTERVIEW THE USER" task="/grill-me: resolve the open decisions, the gaps, every competing reading of the intent, and every criterion readable two ways until one reading remains" />
<step name="RECORD THE DECISIONS" task="Record the decisions: move a feature taken up out of the backlog; record a rejected idea under decided-against with the reasoning; add a term the project will keep to the glossary." />
<loop until="the check finds nothing left to fix" max="3">
<step name="DOUBLE-CHECK" task="/double-check the framed issue; fix what it finds" />
</loop>
<gate check="No criterion reads TBD, and contract-design and accept can check every one pass/fail without interpretation" on-fail="/grill-me, then rework the criterion" />
<gate check="The issue contradicts no convention or contract" on-fail="report the conflict; never override it" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<step name="COMMIT THE FRAME" task="Commit the framed issue and the knowledge edits on the feature's branch, per the `development-procedure` concept's commit convention" />
<skill_call name="/contract-design" when="always" />
</process_actions>

<rules>
<rule level="MUST NOT">produce a design or code</rule>
<rule level="SHALL">treat frame rejections as scope, not solutions</rule>
<rule level="SHALL">record a rejected solution alternative in a contract-design ADR</rule>
<rule level="SHALL">describe behaviour in the project's terms as defined in the glossary; say what the feature does, never how</rule>
<rule level="SHALL">treat the issue as the feature's durable record: its criteria become the tests and the cases, and completion is recorded by its `lifecycle`</rule>
</rules>
</skill>
