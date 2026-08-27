---
name: double-check
description: Double-check the last work, whether it be a plan, design, implementation, or documentation, ensuring all aspects have been fully considered. Use when user wants to check work, or mentions "double check".
---

<skill name="double-check" purpose="Double-Check the Last Work" user-input="$ARGUMENTS" output="findings of the check and any corrections applied">

<goal>
Double-check the last work, whether it be a plan, design, implementation, or documentation, ensuring all aspects have been fully considered.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
</bootstrap_actions>

<process_actions>
<loop until="you are confident in the work">
<step name="CHECK IMPLEMENTATION AGAINST PLAN" task="If last work was an implementation from a plan or design, check each aspect has been correctly and completely implemented as specified, correcting when not the case" />
<step name="CHECK COMPLETENESS AND CORRECTNESS" task="If last work was implementation or documentation, check for completeness, correctness, and potential issues" />
</loop>
</process_actions>


<rules>
<rule level="SHALL">clarify open points with the user</rule>
<rule level="MAY">use todos and tools as needed</rule>
</rules>
</skill>
