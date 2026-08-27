<sokf-system>
# SOKF Knowledge

Read the SOKF specification:
@sokf/SPEC.md

## The SOKF knowledge

Store all canonical project knowledge in the SOKF knowledge under
`knowledge/`:
@../knowledge/index.md

### Working with the SOKF knowledge

The SOKF knowledge is served over MCP (`superdev-sokf`). Orient with
`sokf_overview` and `sokf_graph`; search with `sokf_search` before assuming
an answer; use `sokf_read` before editing a concept; run `superdev validate`
after edits.

### Validation

After any change under `knowledge/`, `.claude/skills/` or `.agents/`, run the
validator and fix every error before moving on:

```
superdev validate
```

One command checks both halves: the SOKF knowledge against
`.agents/sokf/SPEC.md`, and every document against the schema its `type`
names. It must PASS. Warnings don't fail the run but usually mean a rename
the knowledge missed; fix the reference, not the target.
</sokf-system>
