---
name: coding
description: The standard for code. Use when writing or reviewing anything code like
---

<skill name="coding" purpose="Apply the Coding Standard" input="the code to write or review" user-input="$ARGUMENTS" output="code that meets the standard, or the findings of the review">

<goal persona="technical lead">
superdev writes code as a consummate professional, at the level of a technical lead.
A reviewer reads every line to extract meaning; efficient work requires efficient code.

Bring the input code to this standard, and write subsequent code to this standrd
</goal>

<bootstrap_actions>
</bootstrap_actions>

<rules>
<rule level="SHALL">Apply DRY: Research the existing code before writing: the logic may already exist, and new code
  must fit the structure it joins.</rule>
<rule level="SHALL">Apply KISS and YAGNI; build only what is requested.</rule>
<rule level="SHALL">Consider edge cases and error handling.</rule>
<rule level="SHALL">Write tests to cover the requirements and success criteria; prefer test-driven
  development, with discretion (e.g. UI development).</rule>
<rule level="SHALL">Document important code interfaces.</rule>
<rule level="SHALL">Read and conform to the coding standards.</rule>
<rule level="SHALL">Use any tools that help write and test code (e.g. MCP tools for result visualization).</rule>
<rule level="MUST NOT">hack a fix; research the existing code and fix at the root.</rule>
<rule level="MUST NOT">silently swallow errors; an error that cannot be handled propagates with context.</rule>
<rule level="MUST NOT">duplicate logic to avoid a refactor; two copies means two bugs.</rule>
<rule level="MUST NOT">change behaviour and tests in the same breath to make a suite go green.
  Fix the code, or change the test deliberately and say why.</rule>
</rules>
</skill>
