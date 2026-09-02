# Contracts

Two kinds, by audience, both durable. A public contract is promised to
callers outside this repository and changes only as its stability section
allows. An internal contract binds modules inside it — build codes against
it, and CONTRACT-DESIGN updates it as features change the interface. A
feature's contract changes are traced through its feature-request's links
to the contracts it touched, and git holds the history.

## Public

* [CLI contract for superdev][sokf:contract-002-cli-superdev] - the superdev command line — every command, argument and flag as clap declares it, the exit codes and streams each command promises, and what may change.
* [API contract for sokf over MCP][sokf:contract-003-api-sokf] - the SOKF knowledge served to agents — four read-only tools over stdio as the server declares them, and what each call promises beyond its signature.
* [Config contract for superdev][sokf:contract-004-config-superdev] - what a managed repo supplies to superdev — the manifest as the reader declares it, the four environment variables, which source defines what, and what an unknown or invalid setting does.
* [Format contract for pack.toml][sokf:contract-005-format-pack] - what a content pack must look like for superdev to read it — pack.toml as the reader declares it, the tree that names each item, and what is refused.
* [Format contract for lock.toml][sokf:contract-006-format-lock] - what superdev records of the last apply — lock.toml as the writer declares it, the per-capability components, the file hashes, the resolved packs — and what a reader may conclude from it.
* [Template Format Contract][sokf:contract-008-text-format-template] - what a project template is — where its tree lives in the pack, the five substitution tokens, the write-once promise to a seeded repo, and one section per shipped template.

## Internal

* [Interface contract for pack resolution][sokf:contract-007-interface-pack-resolution] - the internal interfaces that carry external content to components — pack source identity, the item model, the resolved content set, the resolution phase, the pin update proves, the process seam, and the Ctx that keeps planning pure.

* [Interface contract for run state][sokf:contract-009-interface-run-state] - the interface between the unattended loop's skill and its Stop hook — the run-state file, the verbs that write it, the hook's decision table, and the managed hook entry that arms it.

* [Interface contract for document schemas][sokf:contract-010-interface-document-schemas] - the declaration vocabulary a document schema may carry — frontmatter constraints, section rules and content kinds — and what each declaration obliges the validator to check.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-003-api-sokf]: /knowledge/contracts/public/active/contract-003-api-sokf.md
[sokf:contract-004-config-superdev]: /knowledge/contracts/public/active/contract-004-config-superdev.md
[sokf:contract-005-format-pack]: /knowledge/contracts/public/active/contract-005-format-pack.md
[sokf:contract-006-format-lock]: /knowledge/contracts/public/active/contract-006-format-lock.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:contract-008-text-format-template]: /knowledge/contracts/public/active/contract-008-text-format-template.md
[sokf:contract-009-interface-run-state]: /knowledge/contracts/internal/active/contract-009-interface-run-state.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
