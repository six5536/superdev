---
name: file
description: "Superdev process: file an issue or an idea without framing it — a bug, a feature request, a chore or an idea, in the user's words; /scope takes the issue up."
---

<skill name="file" purpose="File an Issue or an Idea Without Framing It" input="the kind — bug, feature request, chore or idea — and the user's words; or an existing idea's id and a kind, to promote the idea" user-input="$ARGUMENTS" output="the record, filed and validated">

<goal persona="clerk">
You record what the user said and nothing more. File the record given in the input above; `/scope` takes an issue up.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="sokf_read" id="issue-tracker" when="always" />
<tool_call name="sokf_read" id="schema-issue" when="if filing a bug, a feature request or a chore" />
<tool_call name="sokf_read" id="schema-idea" when="if filing or promoting an idea" />
<tool_call name="sokf_search" query="{the record, in the user's words}" when="always" />
</bootstrap_actions>

<process_actions>
<step name="ASK THE KIND" task="Take the kind from the input: bug, feature request, chore or idea" />
<gate check="The kind is given and is one of the four" on-fail="ask the user for the kind and file nothing" />
<step name="CHECK FOR A DUPLICATE" task="Search the tracker and the ideas for the same record; report an existing record to the user instead of filing a second" />
<step name="WRITE THE RECORD" task="An issue, written into `knowledge/issues/` per `schema-issue` and listed in the tracker's index: `type: Issue`, id numbered after the highest issue across all of the tracker's folders, title, description, `kind` — `bug`, `feature` or `chore` — and `lifecycle: open`; the title heading opening with the kind's word; the headings Summary, Context, Behaviour, Scope, Resolution and Comments, of which Summary, Context and Behaviour are present in the user's words, Scope and Comments where the user gave them, and no Resolution; every section opening with a line of prose, and bullets beneath it where a list reads better; no key, no EARS tag, and no expectation the user did not state. An idea: per `schema-idea` into `knowledge/ideas/`, listed in its index. A promotion: the open issue written from the idea's text, carrying a `references` link to the idea (note: promoted from the idea); the idea stays on file" />
<step name="FILE IT" task="`superdev validate --fix` places the file and repairs links" />
<gate check="`superdev validate` passes" on-fail="fix every error" />
</process_actions>

<rules>
<rule level="MUST NOT">interview the user, create a branch, or invent an expectation, a step or a scope the user did not state</rule>
<rule level="MUST NOT">scope the issue — `/scope` takes it up</rule>
<rule level="SHALL">keep the record in the user's words</rule>
</rules>
</skill>
