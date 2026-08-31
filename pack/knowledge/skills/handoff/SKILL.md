---
name: handoff
description: "Use when the work moves to another session, harness, repo, or person, and the context must travel with it."
argument-hint: "What will the next session be used for?"
disable-model-invocation: true
---

<skill name="handoff" purpose="Write a Handoff Document" input="what the next session will be used for, when given; tailor the document to it. Ask when it is not clear from the conversation" user-input="$ARGUMENTS" output="a handoff document saved outside the workspace, and one line on what it seeds">

<goal persona="outgoing engineer at a shift change">
You write the handover the incoming agent works from. Write the handoff document for the next session as specified in the input above.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
</bootstrap_actions>

<process_actions>
<step name="ESTABLISH NEEDS" task="Establish what the next session needs: the goal it picks up, and where this session left the work" />
<step name="WRITE THE DOCUMENT" task="Write the handoff document: the state of the work — done, in progress, not started; the decisions made and the reasoning behind them; the dead ends, so they are not walked twice; the next steps, concrete enough to start from cold; and a suggested-skills section — the skills the next agent should invoke, `/frame`-style references" />
<step name="REFERENCE ARTIFACTS" task="Reference artifacts instead of duplicating them: a plan, ADR, issue, contract, commit, or diff is cited by id — or by path for what is no concept — not copied in" />
<step name="REDACT" task="Redact secrets and personal data: API keys, passwords, anything personally identifiable" />
<step name="SAVE THE FILE" task="Save the file outside the workspace, in the OS temporary directory" />
</process_actions>


<rules>
<rule level="MUST">make the document stand alone: the reader has no access to this session</rule>
<rule level="MUST NOT">let anything sensitive leave the session</rule>
</rules>
</skill>
