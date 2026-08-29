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

### Writing a link

A link to a concept names its `id`, never its path, so moving or renaming a
document breaks nothing:

```markdown
The planner reads [config][sokf:config] before it plans.
```

A link to anything that is not a concept — a source file, `/CONTRIBUTING.md`,
a URL — stays an ordinary path link. That is the one exception.

The `<!-- sokf:links -->` block at the foot of each document is generated.
Do not write or edit one by hand; `superdev validate --fix` writes them all.

### Validation

After any change under `knowledge/`, `.claude/skills/` or `.agents/`, run the
validator and fix every error before moving on:

```
superdev validate --fix
```

`--fix` converts a path link to the id form and regenerates every
`<!-- sokf:links -->` block, then reports what is left. Run it before
committing.

One command checks both halves: the SOKF knowledge against
`.agents/sokf/SPEC.md`, and every document against the schema its `type`
names. It must PASS. Warnings don't fail the run but usually mean a rename
the knowledge missed; fix the reference, not the target.
</sokf-system>
