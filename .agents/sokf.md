<aokf-system>
# Canonical Project Knowledge

Read the AOKF specification:
@aokf/SPEC.md

## The canonical knowledge

Store all canonical project knowledge under `knowledge/`:
@../knowledge/index.md

### Working with the canonical knowledge

The canonical knowledge is served over MCP (`superdev-aokf`). Orient with
`aokf_overview` and `aokf_graph`; search with `aokf_search` before assuming
an answer; use `aokf_read` before editing a concept; run
`superdev aokf validate knowledge` after edits.

### Validation

After any change under `knowledge/`, run the AOKF validator and fix every
error before moving on:

```
superdev aokf validate knowledge
```

It checks the canonical knowledge against `.agents/aokf/SPEC.md` and must PASS.
Warnings don't fail the run but usually mean a rename the canonical
knowledge missed; fix the reference, not the target.
</aokf-system>
