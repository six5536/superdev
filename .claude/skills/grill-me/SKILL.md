---
name: grill-me
description: Interview the user relentlessly about a plan or design until reaching shared understanding, resolving each branch of the decision tree. Use when user wants to stress-test a plan, get grilled on their design, or mentions "grill me".
---

<skill name="grill-me" purpose="Interview the User About a Plan or Design" user-input="$ARGUMENTS" output="a shared understanding of the plan or design, reached in conversation">

<goal>
Interview the user relentlessly about every aspect of the plan or design under discussion until you reach a shared understanding. Walk down each branch of the design tree, resolving dependencies between decisions one-by-one.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
</bootstrap_actions>


<rules>
<rule level="SHALL">provide your recommended answer for each question</rule>
<rule level="SHALL">ask the questions one at a time</rule>
<rule level="SHALL">explore the codebase instead when a question can be answered by exploring the codebase</rule>
</rules>
</skill>
