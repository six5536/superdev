# Contracts

Two kinds, by audience, both durable. A public contract is promised to
callers outside this repository and changes only as its stability section
allows. An internal contract binds modules inside it — build codes against
it, and CONTRACT-DESIGN updates it as features change the interface. A
feature's contract changes are traced through its feature-request's links
to the contracts it touched, and git holds the history.

## Public

* [CLI Contract][sokf:contract-002-cli-superdev] - the superdev command line — the manage verbs, the knowledge verbs, and what each one promises its callers.
* [MCP Contract][sokf:contract-003-mcp-sokf] - the SOKF knowledge served to agents — four read-only tools over stdio, and what each one promises.
* [Configuration Contract][sokf:contract-004-config-superdev] - what a managed repo supplies to superdev — the manifest keys, the three environment variables, and which source defines what.
* [Pack Format Contract][sokf:contract-005-file-format-pack] - what a content pack must look like for superdev to read it — pack.toml, the tree that names each item, and what is refused.
* [Lock Format Contract][sokf:contract-006-file-format-lock] - what superdev records of the last apply — the per-capability components, the file hashes, the resolved packs — and what a reader may conclude from it.
* [Template Format Contract][sokf:contract-008-file-format-template] - what a project template is — where its tree lives in the pack, the five substitution tokens, the write-once promise to a seeded repo, and one section per shipped template.

## Internal

* [Pack Resolution Interface Contract][sokf:contract-007-interface-pack-resolution] - the internal interfaces that carry external content to components — pack source identity, the item model, the resolved content set, the resolution phase, the pin update proves, the process seam, and the Ctx that keeps planning pure.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-003-mcp-sokf]: /knowledge/contracts/public/active/contract-003-mcp-sokf.md
[sokf:contract-004-config-superdev]: /knowledge/contracts/public/active/contract-004-config-superdev.md
[sokf:contract-005-file-format-pack]: /knowledge/contracts/public/active/contract-005-file-format-pack.md
[sokf:contract-006-file-format-lock]: /knowledge/contracts/public/active/contract-006-file-format-lock.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:contract-008-file-format-template]: /knowledge/contracts/public/active/contract-008-file-format-template.md
