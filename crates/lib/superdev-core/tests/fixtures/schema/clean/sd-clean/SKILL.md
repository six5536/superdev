---
name: sd-clean
description: A fixture skill, used only to pin the reference validator's behaviour.
---

<skill name="sd-clean" purpose="Demonstrate a Valid Unit" user-input="$ARGUMENTS" output="nothing; this is a fixture">

<goal>
A fixture.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
</bootstrap_actions>

<process_actions>
<step name="DO THE THING" task="Carry out the one thing this fixture exists to name" />
</process_actions>

<rules>
<rule level="MUST">stay a fixture</rule>
</rules>
</skill>
