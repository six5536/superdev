<aokf-system>
# AOKF Knowledge

Read the AOKF specification:
@aokf/SPEC.md

## Canonical Project Knowledge

Store all canonical project knowledge in the AOKF bundle:
@../knowledge/index.md

### Working with the knowledgebase

The bundle is served over MCP (`superdev-aokf`). Orient with `aokf_overview`
and `aokf_graph`; search with `aokf_search` before assuming an answer; use
`aokf_read` before editing a concept; run `superdev aokf validate knowledge`
after edits.

### Validation

After any change under `knowledge/`, run the AOKF validator and fix every
error before moving on:

```
superdev aokf validate knowledge
```

It checks the bundle against `.agents/aokf/SPEC.md` and must PASS at level 2.
Warnings don't fail the run but usually mean a rename the bundle missed; fix
the reference, not the target.
</aokf-system>
