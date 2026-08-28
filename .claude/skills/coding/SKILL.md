---
name: coding
description: The standard for code. Use when writing or reviewing anything code like
---

<skill name="coding" purpose="Apply the Coding Standard" input="the code to write or review" user-input="$ARGUMENTS" output="code that meets the standard, or the findings of the review">

<goal persona="a professional software engineer, certainly not a tech bro or script kiddie">
Bring the code to the standard core's `coding` block sets.

The reason is that a human reviewing your output must read every line to extract meaning.
In order to work together efficiently, we must be able to code efficiently.
</goal>

<bootstrap_actions>
</bootstrap_actions>

<rules>
<rule level="SHALL">read core's `coding` block first; it holds the norms, and what follows adds to them</rule>
<rule level="SHALL">use KISS and YAGNI principles. Do not create more than requested.</rule>
<rule level="SHALL">actively research existing code to apply the DRY principle.</rule>
<rule level="SHALL">Never write code without researching the existing code to understand (a) if it already exists and (b) if not where it fits.</rule>
<rule level="SHALL">use any tools you need to help write and test code (e.g. MCP tools for result visualization).</rule>
</rules>
</skill>
