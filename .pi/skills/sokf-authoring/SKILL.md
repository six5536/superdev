---
name: sokf-authoring
description: Author and revise canonical SOKF project knowledge on physical paths with Pi's file tools. Use when creating concepts, changing metadata or relationships, or resolving validation findings.
---

# SOKF authoring in Pi

## Gather only the required context

1. Read a known physical `knowledge/` path directly. Use `sokf_graph` for a known concept ID or `sokf_search` for an unknown concept to obtain its path. Use `sokf_overview` for orientation.
2. Read the type's schema under `knowledge/schemas/` before creating or changing a concept.
3. If the type has no schema, create `knowledge/schemas/<type>.md` first and list it in `knowledge/schemas/index.md`. Model the new schema on an existing schema, define the type's frontmatter and document structure, and include a conforming example.
4. Read the target with `read path="knowledge/<path>.md"` before changing it.
5. Read `.agents/sokf/SPEC.md` only when the task depends on SOKF syntax or semantics.

## Write physical files

- Use Pi's own `read`, `edit`, and `write` on physical `knowledge/` paths, just as for any other file.
- Before creating a numbered concept, read its destination index and choose the next unused number. Verify that path, `id`, and numbered title agree before the first write; do not create a concept and then renumber its stable identity.
- Keep each existing `id` stable.
- Preserve an existing `verified` field byte-for-byte.
- Never add stamped fields.
- Record derivation materials in `sources`. Use keyed footnotes for individual claims.
- Use allowed typed `links` and add the body mirrors required by the concept schema.
- Leave generated definition and include blocks to turn-end repair. Do not hand-edit generated content.

## Converge

Complete related changes before the turn ends; intermediate files can be invalid. The SOKF Pi extension runs one unconditional `superdev validate --fix` at turn end, whatever wrote the files, including Bash or patches. File tools do not repair knowledge as they write it.

Resolve reported findings. The first report of a pending sequence triggers one follow-up turn; later reports remain visible without triggering another turn. A clean run resets the sequence. Repair can refile a concept after a lifecycle change; use its current physical path for subsequent file operations.

Before finishing, update every concept affected by changes to code, behaviour, contracts, or outward-facing documentation. Keep only a concise, cited summary of outward-facing information in SOKF.
