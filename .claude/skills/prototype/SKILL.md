---
name: prototype
description: "Use when the user wants to sanity-check whether a state model or logic feels right, or explore what a UI should look like."
---

<skill name="prototype" purpose="Prototype a Design Question" input="the design question, when given" user-input="$ARGUMENTS" output="the verdict: the question, the answer, and the validated decision folded into the real code and recorded on the driving concept; the prototype stays on its throwaway branch">

<goal persona="design engineer">
You write throwaway code that answers one design question.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path=".claude/skills/prototype/LOGIC.md" when="if logic branch" />
<tool_call name="read_file" path=".claude/skills/prototype/UI.md" when="if UI branch" />
</bootstrap_actions>

<process_actions>
<step name="IDENTIFY THE QUESTION" task="Identify the question — from the prompt, the surrounding code, or by asking the user" />
<step name="PICK THE BRANCH">Pick the branch; the wrong one wastes the whole prototype:

- "Does this logic or state model feel right?" → LOGIC.md: a single shareable HTML file — free-play buttons plus tabbed guided walkthroughs — that pushes the state machine through cases hard to reason about on paper, and that a non-developer can drive.
- "What should this look like?" → UI.md: several radically different UI variations on a single route, switched by a URL search param and a floating bottom bar.</step>
  <gate check="The question is unambiguous, or the user is reachable" on-fail="match the surrounding code (backend module → logic; page or component → UI) and state the assumption at the top of the prototype" />
  <step name="BUILD" task="Build it per the chosen branch file" />
  <step name="FOLD DECISIONS" task="Fold each validated decision into the real code" />
  <step name="CAPTURE THE PROTOTYPE" task="Capture the prototype as a primary source: commit it to a throwaway branch, out of main, and leave a pointer to that branch — with the verdict and the question it settled — on the issue or plan driving the work" />
  </process_actions>


<rules>
<rule level="SHALL">keep it throwaway from day one, and marked as such: place it next to the module or page it prototypes so context is obvious, named so a casual reader sees it is not production</rule>
<rule level="SHALL">follow the project's routing convention for a throwaway UI route</rule>
<rule level="MUST NOT">invent new top-level structure</rule>
<rule level="SHALL">make it trivial to run: a UI prototype starts from one command in the project's task runner; a logic demo is one HTML file the user double-clicks</rule>
<rule level="MUST NOT">persist by default: state lives in memory — persistence is what the prototype is checking; if the question involves a database, use a scratch DB or a local file with a clear "PROTOTYPE — wipe me" name</rule>
<rule level="SHALL">skip the polish: no tests, no abstractions, no error handling beyond what makes it runnable — the point is to learn fast</rule>
<rule level="SHALL">surface the state: after every action (logic) or on every variant switch (UI), show the full relevant state</rule>
</rules>
</skill>
