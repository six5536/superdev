---
name: interface-design
description: "Superdev process: use to design the interface once the spec is clear."
---

<skill name="interface-design" purpose="Design the Interface" input="the feature or spec id, when not handed off" user-input="$ARGUMENTS" output="the interface contract and, for UI, the mockup, with each decision recorded as an ADR">

<goal persona="systems architect">
You decide only the interfaces that will be expensive to change once other code depends on them. Design the feature's interfaces as specified in the input above, following `schema-interface-contract` and `schema-adr`.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/interface-contract.md" when="always" />
<tool_call name="read_file" path="knowledge/schemas/adr.md" when="always" />
<tool_call name="aokf_read" id="spec-{nnn}-{feature-slug}" when="always" />
<tool_call name="aokf_search" query="{the feature or spec}" when="if the spec id is not given" />
<tool_call name="aokf_read" id="architecture" when="always" />
<tool_call name="aokf_read" id="architectural-rules" when="always" />
<tool_call name="aokf_read" id="api-contracts" when="always" />
<tool_call name="codegraph_explore" when="before adding new interfaces" />
</bootstrap_actions>

<process_actions>
<step name="DECIDE WHAT IS EXPENSIVE" task="Decide what is expensive to change: data schema, API contracts, module boundaries, auth surface, and the UI" />
<step name="ESTABLISH EXTERNAL FACTS" task="Does a contract rest on a third-party API or another external fact? Establish it with `/research`; the findings land in the canonical knowledge for later phases" />
<step name="WRITE BACKEND CONTRACTS" task="Backend interfaces: a written contract per `schema-interface-contract`" />
<step name="MOCK THE UI" task="UI: a mockup (`/design`; `/frontend-design` for the visual direction) or a throwaway prototype (`/prototype`). Discard it and build against it" />
<step name="INTERVIEW THE USER" task="Interview the user (`/grill-me`) on each decision and its alternatives before filing the ADR. A question conversation cannot settle gets a runnable answer (`/prototype`)" />
<step name="RECORD ADRS" task="Record each decision as an ADR per `schema-adr`, listed in the decisions index" />
<step name="DOUBLE-CHECK" task="Double-check the contract and ADRs (`/double-check`); fix what it finds" />
<gate check="A new interface contradicts neither the architecture nor its rules" on-fail="reject it, or report the conflict for a deliberate change" />
<gate check="knowledge validates to PASS per the core knowledge block" on-fail="fix every error" />
<gate check="Everything internal is left to build" on-fail="stop deciding it here" />
<skill_call name="/feature-plan" when="always" />
</process_actions>


<rules>
<rule level="SHALL">record a rejected alternative in the decision's ADR</rule>
<rule level="SHALL NOT">record a rejected alternative in the backlog</rule>
</rules>
</skill>
