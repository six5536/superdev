---
name: contract-design
description: "Superdev process: a sub-skill of /scope — make one plan's contract changes and record the decisions behind them as ADRs."
---

<skill name="contract-design" purpose="Make a Plan's Contract Changes" input="one plan's Contract changes — the contracts the work touches and the promises and criteria it adds, changes or withdraws" user-input="$ARGUMENTS" output="each named contract created or updated, its new definition elements declared in source, and every decision recorded as an ADR; handed back to `/scope`">

<goal persona="systems architect">
You decide only the interfaces that will be expensive to change once other code depends on them. Make the contract changes given in the input above, following `schema-contract` and `schema-adr`. Contracts are durable: they describe the app's interfaces as they stand, never one piece of work.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="sokf_read" id="schema-contract" when="always" />
<tool_call name="read_file" path="knowledge/schemas/adr.md" when="always" />
<tool_call name="sokf_read" id="plan-{nnn}-{slug}" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{slug}" when="if the plan carries a Request line" />
<tool_call name="sokf_read" id="architecture" when="always" />
<tool_call name="sokf_read" id="architectural-rules" when="always" />
<tool_call name="sokf_search" query="the contracts this project keeps, public and internal" when="always" />
<tool_call name="codegraph_explore" when="before adding new interfaces" />
</bootstrap_actions>

<process_actions>
<step name="READ THE CONTRACT CHANGES" task="Take the plan's Contract changes as the work list: one contract per bullet, with the promises and criteria it adds, changes or withdraws. A change the bullets do not name is not made here — it returns to `/scope` for the plan to name it first" />
<step name="DECIDE WHAT IS EXPENSIVE" task="Decide what is expensive to change: data schema, API contracts, module boundaries, auth surface, and the UI" />
<step name="UPDATE THE CONTRACTS" task="Create or update each contract the plan names per `schema-contract` — its kind's checklist, its Definition as source includes — keyed to the interface, never to the work. The document states current truth; git holds the history" />
<step name="DECLARE IN SOURCE" task="New definition element — a field, an argument, a path, a message? Write it into its marked source region with its behaviour unbuilt, so the include shows it and build implements behind it; `superdev validate --fix` materialises the include (ADR-044)" />
<step name="LINK THE REQUEST" task="Add a `references` link from the issue the plan delivers to each contract created or changed, its note saying what changed — this is the work's trace into the contracts. One-off work with no issue carries the trace in the plan's Contract changes alone" />
<step name="INTERVIEW THE USER" task="Interview the user (`/grill-me`) on every decision an ADR will record — contested or not — and its alternatives, before filing the ADR; the interview is an interaction, never a self-check. A question conversation cannot settle gets a runnable answer (`/prototype`)" />
<step name="RECORD ADRS" task="Record each decision as an ADR per `schema-adr`, listed in the ADRs index" />
<step name="DOUBLE-CHECK" task="Double-check the contracts and ADRs (`/double-check`); fix what it finds" />
<gate check="A changed contract contradicts neither the architecture nor its rules, and a public contract's stability section allows the change" on-fail="reject it, or report the conflict for a deliberate change" />
<gate check="`superdev validate` passes: the SOKF knowledge, and every document against its schema" on-fail="fix every error" />
<gate check="Everything internal is left to build" on-fail="stop deciding it here" />
<step name="PRESENT THE CHANGE SET" task="Present the complete change set to the user — every contract, source declaration and ADR created or changed, what each change binds, and the rejected alternatives — and ask for the go-ahead; the presentation is an interaction, never a self-check, and nothing commits before it" />
<gate check="The user has explicitly approved the presented change set" on-fail="apply the rework they name and present the revised change set again; while approval is withheld the edits stay uncommitted on the work's branch" />
<skill_call name="/scope" when="always" input="the approved contract, source declaration and decision-record edits, for the plan and the commit" />
</process_actions>


<rules>
<rule level="SHALL">record a rejected alternative in the decision's ADR</rule>
<rule level="MUST NOT">create a per-feature contract; a contract is keyed to the interface it describes</rule>
<rule level="MUST">write every contract to the contract style its schema carries</rule>
<rule level="MUST NOT">commit; `/scope` commits the approved edits with the plan</rule>
</rules>
</skill>
