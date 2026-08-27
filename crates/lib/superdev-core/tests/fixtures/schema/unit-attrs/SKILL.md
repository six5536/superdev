---
name: sd-attrs
description: Attribute rules broken in several ways.
---

<skill name="Attrs" purpose="Break Attribute Rules" user-input="">
<goal>No output attribute above.</goal>
<process_actions>
<step task="a step with no name" />
<step name="BAD CONDITION" when="sometimes" task="the condition form is closed" />
<step name="RENAMED" trigger="the old spelling" task="renamed attributes are refused by name" />
<step name="UNKNOWN ATTR" colour="red" task="not in the vocabulary" />
</process_actions>
<rules>
<rule level="MUST">carry on</rule>
</rules>
</skill>
