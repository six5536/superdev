---
name: contract-design
description: "Superdev process: use to design or update the contracts a feature touches, once the framed issue's criteria are clear."
---

<skill name="contract-design" purpose="Design the Contracts" input="the framed issue's id, when not handed off" user-input="$ARGUMENTS" output="the contracts the feature touches created or updated, the framed issue linked to each, and every decision recorded as an ADR">

<goal persona="systems architect">
You decide only the interfaces that will be expensive to change once other code depends on them. Design or update the contracts the feature given in the input above touches — public and internal — following each contract kind's schema and `schema-adr`. Contracts are durable: they describe the app's interfaces as they stand, never one feature.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/contract-interface.md" when="if an internal boundary changes" />
<tool_call name="read_file" path="knowledge/schemas/adr.md" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{kind}-{slug}" when="always" />
<tool_call name="sokf_search" query="{the feature}" when="if the issue id is not given" />
<tool_call name="sokf_read" id="architecture" when="always" />
<tool_call name="sokf_read" id="architectural-rules" when="always" />
<tool_call name="sokf_search" query="the contracts this project keeps, public and internal" when="always" />
<tool_call name="codegraph_explore" when="before adding new interfaces" />
</bootstrap_actions>

<process_actions>
<step name="DECIDE WHAT IS EXPENSIVE" task="Decide what is expensive to change: data schema, API contracts, module boundaries, auth surface, and the UI" />
<step name="ESTABLISH EXTERNAL FACTS" task="Does a contract rest on a third-party API or another external fact? Establish it with `/research`; the findings land in the canonical knowledge for later phases" />
<step name="UPDATE THE CONTRACTS" task="Create or update each contract the feature touches: a public contract per its kind's schema (`sokf_read schema-{kind}`), an internal boundary per `schema-contract-interface`, keyed to the interface — never to the feature. The document states current truth; git holds the history" />
<step name="LINK THE REQUEST" task="Add a `references` link from the framed issue to each contract created or changed, its note saying what changed — this is the feature's trace into the contracts" />
<step name="MOCK THE UI" task="UI: a mockup (`/design`; `/frontend-design` for the visual direction) or a throwaway prototype (`/prototype`). Discard it and build against it" />
<step name="INTERVIEW THE USER" task="Interview the user (`/grill-me`) on each decision and its alternatives before filing the ADR. A question conversation cannot settle gets a runnable answer (`/prototype`)" />
<step name="RECORD ADRS" task="Record each decision as an ADR per `schema-adr`, listed in the ADRs index" />
<step name="DOUBLE-CHECK" task="Double-check the contracts and ADRs (`/double-check`); fix what it finds" />
<gate check="A changed contract contradicts neither the architecture nor its rules, and a public contract's stability section allows the change" on-fail="reject it, or report the conflict for a deliberate change" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<gate check="Everything internal is left to build" on-fail="stop deciding it here" />
<skill_call name="/feature-plan" when="always" />
</process_actions>


<rules>
<rule level="SHALL">record a rejected alternative in the decision's ADR</rule>
<rule level="SHALL NOT">record a rejected alternative in the backlog</rule>
<rule level="MUST NOT">create a per-feature contract; a contract is keyed to the interface it describes</rule>
</rules>
</skill>
