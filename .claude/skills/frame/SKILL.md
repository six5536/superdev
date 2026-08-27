---
name: frame
description: "Superdev process: use at the start of a feature or project."
---

<skill name="frame" purpose="Frame a Project or Feature" input="New project or feature (existing project) to frame" user-input="$ARGUMENTS" output="the frame: goal, user, constraints, stack">

<goal persona="product manager">
You define the problem, not the solution. Frame the project or feature given in the input above.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="sokf_overview" when="always" />
<tool_call name="sokf_search" query="{existing knowledge on the project or feature}" when="always" />
<tool_call name="sokf_read" id="glossary" when="always" />
<tool_call name="sokf_read" id="project-overview" when="if framing a feature" />
<tool_call name="sokf_read" id="constraints-non-goals" when="if framing a feature" />
<tool_call name="sokf_read" id="backlog" when="if framing a feature" />
<tool_call name="sokf_read" id="technology-stack" when="if framing a feature" />
<tool_call name="sokf_read" id="visual-system" when="if framing a feature" />
<tool_call name="sokf_read" id="template-readme" when="if new project" />
<tool_call name="sokf_read" id="template-project-overview" when="if new project" />
<tool_call name="sokf_read" id="template-technology-stack" when="if new project" />
<tool_call name="sokf_read" id="template-constraints-non-goals" when="if new project" />
<tool_call name="sokf_read" id="template-visual-system" when="if new project" />
<tool_call name="codegraph_explore" query="{existing code}" when="if relevant" />
</bootstrap_actions>

<process_actions>
<gate check="Project knowledge is initialised, not TBD" on-fail="/bootstrap" />
<gate check="The feature is not already decided against or out-of-scope" on-fail="tell the user why" />
<gate check="The idea is well shaped, so a goal can be stated" on-fail="/brainstorm, then frame its shortlist" />
<step name="STATE THE FRAME" task="State the goal, the user, and the constraints, using the `glossary`'s terms" />
<step name="INTERVIEW THE USER" task="/grill-me: resolve the open decisions, the gaps, and every competing reading of the intent" />
<step name="CHOOSE TECH STACK" when="if new project" task="Choose the tech stack — `/research` settles an open technology question"/>
<step name="CHOOSE VISUAL SYSTEM" when="if new project" task="set the visual system with `/frontend-design`" />
<step name="CREATE README AND KNOWLEDGE" when="if new project" task="create the README and the canonical knowledge from the templates" />
<step name="RECORD THE DECISIONS" task="Record the decisions: move a feature taken up out of the backlog; record a rejected idea under decided-against with the reasoning; add a term the project will keep to the glossary." />
<loop until="the check finds nothing left to fix" max="3">
<step name="DOUBLE-CHECK" task="/double-check the frame; fix what it finds" />
</loop>
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<gate check="The frame is clear enough for the spec skill" on-fail="/grill-me" />
<skill_call name="/spec" when="always" />
</process_actions>

<rules>
<rule level="MUST NOT">produce a spec, a design, or code</rule>
<rule level="SHALL">treat frame rejections as scope, not solutions</rule>
<rule level="SHALL">record a rejected solution alternative in an interface-design ADR</rule>
</rules>
</skill>
