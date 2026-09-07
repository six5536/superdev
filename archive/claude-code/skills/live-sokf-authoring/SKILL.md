---
name: sokf-authoring
description: "Author and revise canonical SOKF project knowledge. Use when creating concepts, changing concept structure or metadata, adding sources or typed links, or resolving SOKF validation findings."
---

<skill name="sokf-authoring" purpose="Author Canonical SOKF Knowledge" input="the knowledge change" user-input="$ARGUMENTS" output="valid, current SOKF concepts">

<goal>
Create or update canonical project knowledge without loading the full SOKF specification unless format semantics matter.
</goal>

<bootstrap_actions>
<tool_call name="read" path="sokf:schema-{type}" when="before creating or changing a concept" />
<tool_call name="read" path="sokf:{id}" when="before changing an existing concept" />
<tool_call name="read" path=".agents/sokf/SPEC.md" when="if the task depends on SOKF syntax or semantics" />
</bootstrap_actions>

<process_actions>
<step name="ENSURE THE SCHEMA" task="If schema-{type} does not exist, create knowledge/schemas/{type}.md first, list it in knowledge/schemas/index.md, model it on an existing schema, define the type's frontmatter and document structure, and include a conforming example" />
<step name="ALLOCATE IDENTITY" task="Before creating a numbered concept, read its destination index, choose the next unused number, and verify that path, id and numbered title agree before the first write; never create a concept and then renumber its stable identity" />
<step name="AUTHOR" task="Create a concept at a physical knowledge/&lt;path&gt;.md path, or edit an existing concept through sokf:&lt;id&gt; when possible" />
<step name="RELATE" task="Record derivation materials in sources, use keyed footnotes for per-claim attribution, and use allowed typed links with required body mirrors" />
<step name="CONVERGE" task="Treat applied invalid or unknown mutations as completed writes, finish related changes without retrying them, then run superdev validate --fix until validation passes" />
</process_actions>

<rules>
<rule level="MUST NOT">change an existing id</rule>
<rule level="MUST NOT">add, edit, reorder, or delete an existing verified field</rule>
<rule level="MUST NOT">add stamped fields or hand-edit generated definition and include blocks</rule>
<rule level="SHALL">update every affected concept when code, behavior, contracts, or outward-facing documentation changes</rule>
<rule level="SHALL">keep only a concise, cited summary of outward-facing information in SOKF</rule>
</rules>
</skill>
