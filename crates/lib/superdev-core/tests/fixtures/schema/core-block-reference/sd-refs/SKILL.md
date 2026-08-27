---
name: sd-refs
description: A fixture skill naming a core block the core does not define.
---

<skill name="sd-refs" purpose="Reference a Core Block" user-input="$ARGUMENTS" output="nothing; this is a fixture">

<goal>
Name the core's workflow block, which exists, and its file_naming block, which does not.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
</bootstrap_actions>

<process_actions>
<step name="FILE IT" task="File it per the core file_naming block, which nothing defines" />
</process_actions>

<rules>
<rule level="MUST">follow the core workflow block</rule>
</rules>
</skill>
