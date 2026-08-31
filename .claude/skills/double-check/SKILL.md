---
name: double-check
description: Double-check the last work, whether it be a plan, design, implementation, or documentation, ensuring all aspects have been fully considered. Use when user wants to check work, or mentions "double check".
---

<skill name="double-check" purpose="Double-Check the Last Work" user-input="$ARGUMENTS" output="findings of the check and any corrections applied">

<goal>
Double-check the last work, whether it be a plan, design, implementation, or documentation, ensuring all aspects have been fully considered.
</goal>

<bootstrap_actions>
</bootstrap_actions>

<process_actions>
<loop until="you are confident in the work">
<step name="CHECK IMPLEMENTATION AGAINST PLAN" when="if last work was an implementation from a plan or design" task="check each aspect has been correctly and completely implemented as specified, correcting when not the case" />
<step name="CHECK COMPLETENESS AND CORRECTNESS" when="if last work was implementation or documentation" task="check for completeness, correctness, and potential issues" />
</loop>
</process_actions>

<rules>
<rule level="SHALL">clarify open points with the user</rule>
<rule level="SHALL">if double-check was just completed, triple-check, and so on, until you are confident in the work.</rule>
<rule level="SHALL">raise open points with your questions tool</rule>
</rules>
</skill>
