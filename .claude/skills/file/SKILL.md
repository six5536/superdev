---
name: file
description: "Superdev process: file an issue or an idea without framing it — a bug, a feature request, a chore or an idea, in the user's words; /frame frames the issue when it is taken up."
---

<skill name="file" purpose="File an Issue or an Idea Without Framing It" input="the kind — bug, feature request, chore or idea — and the user's words; or an existing idea's id and a kind, to promote the idea" user-input="$ARGUMENTS" output="the record, filed and validated">

<goal persona="clerk">
You record what the user said and nothing more. File the record given in the input above; `/frame` frames an issue when it is taken up.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="sokf_read" id="issue-tracker" when="always" />
<tool_call name="read_file" path="knowledge/schemas/bug-report.md" when="if filing a bug" />
<tool_call name="read_file" path="knowledge/schemas/feature-request.md" when="if filing a feature request" />
<tool_call name="read_file" path="knowledge/schemas/chore.md" when="if filing a chore" />
<tool_call name="sokf_read" id="schema-idea" when="if filing or promoting an idea" />
<tool_call name="sokf_search" query="{the record, in the user's words}" when="always" />
</bootstrap_actions>

<process_actions>
<step name="ASK THE KIND" task="Take the kind from the input: bug, feature request, chore or idea" />
<gate check="The kind is given and is one of the four" on-fail="ask the user for the kind and file nothing" />
<step name="CHECK FOR A DUPLICATE" task="Search the tracker and the ideas for the same record; report an existing record to the user instead of filing a second" />
<step name="WRITE THE RECORD" task="An issue, written into `knowledge/issues/` and listed in the tracker's index: the minimum record — `type`, id numbered after the highest across all of the kind's folders, title, description, `lifecycle: unframed`, Summary and Motivation in the user's words, every other heading of the kind present with the user's words or `TBD — <the open question>`, and no criterion, step or done item the user did not state. An idea: per `schema-idea` into `knowledge/ideas/`, listed in its index. A promotion: the unframed issue written from the idea's text, carrying a `references` link to the idea (note: promoted from the idea); the idea stays on file" />
<step name="FILE IT" task="`superdev validate --fix` places the file and repairs links" />
<gate check="`superdev validate` passes" on-fail="fix every error" />
</process_actions>

<rules>
<rule level="MUST NOT">interview the user, create a branch, or invent a criterion, step or done item</rule>
<rule level="MUST NOT">frame the issue — `/frame` frames it when it is taken up</rule>
<rule level="SHALL">keep the record in the user's words</rule>
</rules>
</skill>
