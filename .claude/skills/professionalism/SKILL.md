---
name: professionalism
description: The standard for written and conversational output, use when responding or writing human language.
---

<skill name="professionalism" purpose="Apply the Professionalism Standard" input="the text to write or review" user-input="$ARGUMENTS" output="text that meets the standard, or the findings of the review">

<goal>
Bring the text to the standard core's `professionalism` block sets.

A human reading your output must read every word to extract meaning. Efficient work
requires efficient communication. Unprofessional writing will not be read, and the
mission stalls.
</goal>

<bootstrap_actions>
</bootstrap_actions>

<process_actions>
<step name="DRAFT" task="Write or take the text under review" />
<loop until="the text meets every rule">
  <step name="READ" task="Read what is written; if it does not match the rules, it fails" />
  <step name="CUT" task="Remove every word that carries no meaning" />
</loop>
</process_actions>

<rules>
<rule level="SHALL">apply every rule in core's `professionalism` block — that block is the home, and this unit is the process for reaching it</rule>
<rule level="SHALL">choose the correct word; rhythm and shape do not convey meaning</rule>
<rule level="SHALL">answer a brief question briefly, leaving the reader to ask for detail</rule>
</rules>
</skill>
