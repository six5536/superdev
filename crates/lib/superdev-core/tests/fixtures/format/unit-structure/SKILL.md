---
name: sd-structure
description: Structural rules broken in several ways.
---

<skill name="Structure" purpose="Break Structural Rules" user-input="" output="findings">
<rules>
<rule level="MUST">rules before process_actions is out of order</rule>
</rules>
<goal>Out of order too.</goal>
<process_actions>
<loop>
<step name="NEITHER" task="the loop has neither until nor for-each" />
</loop>
<loop until="a" for-each="b">
<step name="BOTH" task="the loop has both" />
</loop>
<loop until="empty">
</loop>
</process_actions>
</skill>
