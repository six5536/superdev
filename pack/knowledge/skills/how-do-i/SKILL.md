---
name: how-do-i
description: "Use when unsure which skill or flow fits, when asked how a skill works here, or to survey what skills exist."
---

<skill name="how-do-i" purpose="Answer a How-do-i Question" input="the question: which skill fits, how one works here, or what exists" user-input="$ARGUMENTS" output="the answer: invocation, behaviour as adapted here, and the skill's place — or non-place — in the flow">

<goal persona="guide">
You answer from what the skills and process files actually say, not from memory. Answer the question in the input above.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path=".agents/process.md" when="always" />
<tool_call name="read_file" path=".claude/skills/how-do-i/SESSION-BOUNDARIES.md" when="if the question is about a boundary between chunks of work (continue, clear, handoff, subagent, compact)" />
</bootstrap_actions>

<process_actions>
<step name="MAP THE QUESTION">Map the question onto the flow:

- Feature work travels the workflow in the core workflow block; `/accept` runs at the user's request, once the feature has stopped changing.
- An issue or an idea to record without framing it: `/file` — a bug, a feature request, a chore or an idea, in the user's words; `/frame` frames the issue when it is taken up.
- One-off work outside the workflow — a refactor, a migration, a chore: `/adhoc-plan`.
- Support skills the phases call: `/grill-me` (interview the user until one reading remains) and `/double-check` (check the last work) in every document phase; `/brainstorm` from `/frame`, to widen an idea too unshaped to state a goal; `/prototype` from `/contract-design`, for throwaway code answering a question conversation cannot settle; `/research` from `/frame` and `/contract-design`, for external facts from primary sources, filed in the canonical knowledge. All four also run standalone.
- Unattended delivery: `/execute-feature-plan` drives feature-plan, build and integrate in a loop on the feature's branch, deferring the questions only the user can answer; frame and contract-design stay interactive.
- Knowledge upkeep: `/bootstrap` (fill the canonical knowledge from the repo and the owner; `/frame` calls it when an existing project's knowledge is empty), `/maintain` (audit and repair the canonical knowledge and the workflow's records; run regularly).</step>
  <step name="ENUMERATE THE ROSTER" task="Question names a skill outside this map, or asks what else exists? Enumerate: the session's available-skills listing is the full roster; `.claude/skills/` holds the copies in this repo — a skill can appear in either alone. Read its SKILL.md before describing it, and apply its `PROJECT.md` if one exists" />
  <step name="ANSWER" task="Answer in this repo's terms: how to invoke the skill, what it does here, and where it sits in the flow — or that it sits outside it" />
  </process_actions>

<rules>
<rule level="SHALL">describe from what you read</rule>
<rule level="MUST NOT">describe from the skill's name alone</rule>
<rule level="MUST">explain only</rule>
<rule level="MUST NOT">change anything</rule>
</rules>
</skill>
