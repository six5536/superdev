---
name: sd-dup-a
description: A fixture skill, used only to pin the reference validator's behaviour.
---

<skill name="sd-dup-a" purpose="Hold One Half" user-input="$ARGUMENTS" output="nothing; this is a fixture">

<goal>
A fixture.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
</bootstrap_actions>

<process_actions>
<step name="DO THE THING" task="Carry out the one thing this fixture exists to name" />
</process_actions>

<rules>
<rule level="MUST">never write a lockfile entry that a fetch has not verified against its source</rule>
</rules>
</skill>
