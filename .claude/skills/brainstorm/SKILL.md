---
name: brainstorm
description: Use when asked to brainstorm, explore ideas, or analyze trade-offs.
---

<skill name="brainstorm" purpose="Brainstorm Ideas" input="the topic to brainstorm" user-input="$ARGUMENTS" output="a brainstorm summary">

<goal>
Brainstorm ideas, explore solutions, and evaluate options for the topic given in the input above.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="aokf_search" query="{existing knowledge on the topic}" when="always" />
<tool_call name="aokf_read" id="architecture" when="if exists" />
<tool_call name="codegraph_explore" query="{existing code}" when="if relevant" />
</bootstrap_actions>

<process_actions>
<step name="DIVERGE" task="Generate multiple distinct approaches without filtering" />
<step name="ANALYZE" task="Evaluate options against constraints and trade-offs" />
<step name="CONVERGE" task="Recommend top candidates with rationale" />
<skill_call name="/adhoc-plan" when="if the user asks to formalise the outcome" input="the recommended candidates" />
</process_actions>

<rules>
<rule level="SHALL">explore multiple distinct approaches before converging</rule>
<rule level="SHALL">identify trade-offs honestly, not just pros</rule>
<rule level="SHALL">distinguish between facts and assumptions</rule>
<rule level="SHOULD">challenge assumptions and explore alternatives</rule>
<rule level="SHALL NOT">dismiss ideas prematurely during divergence</rule>
<rule level="SHALL">engage in dialogue, asking clarifying questions</rule>
<rule level="MAY">use web search for research and validation</rule>
<rule level="SHALL">clarify open points with the user</rule>
<rule level="MAY">use todos and tools as needed</rule>
</rules>
</skill>
