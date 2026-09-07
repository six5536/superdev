---
name: sokf-authoring
description: Author and revise canonical SOKF project knowledge with Pi's SOKF-aware read, edit, write, search, and graph tools. Use when creating concepts, changing metadata or relationships, or resolving validation findings.
---

# SOKF authoring in Pi

## Gather only the required context

1. Use `sokf_search` when the relevant concept ID is unknown.
2. Read `sokf:schema-{type}` before creating or changing a concept.
3. If `schema-{type}` does not exist, create `knowledge/schemas/<type>.md` first and list it in `knowledge/schemas/index.md`. Model the new schema on an existing schema, define the type's frontmatter and document structure, and include a conforming example.
4. Read the target with `read path="sokf:<id>"` before changing it.
5. Read `.agents/sokf/SPEC.md` only when the task depends on SOKF syntax or semantics.

## Write through the SOKF adapter

- Create a concept with `write path="knowledge/<path>.md"`.
- Before creating a numbered concept, read its destination index and choose the next unused number. Verify that path, `id`, and numbered title agree before the first write; do not create a concept and then renumber its stable identity.
- Change an existing concept with `edit path="sokf:<id>"` when possible.
- Keep each existing `id` stable.
- Preserve an existing `verified` field byte-for-byte.
- Never add stamped fields.
- Record derivation materials in `sources`. Use keyed footnotes for individual claims.
- Use allowed typed `links` and add the body mirrors required by the concept schema.
- Let the mutation tool repair generated definition and include blocks. Do not hand-edit generated content.

## Converge

Treat `applied: true` as a completed write even when validation is `invalid` or `unknown`. Continue with related changes instead of retrying that mutation. Pi runs final validation after the turn and provides at most two automatic repair follow-ups.

Before finishing, update every concept affected by changes to code, behavior, contracts, or outward-facing documentation. Keep only a concise, cited summary of outward-facing information in SOKF.
