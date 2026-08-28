# Contracts

Two kinds, by audience. A public contract is promised to callers outside this
repository and changes only as its stability section allows. A private contract
is what the build codes against for one feature, and is discarded once the code
is canonical.

## Public

* [CLI Contract](public/contract-002-cli-superdev.md) - the superdev command line — the manage verbs, the knowledge verbs, and what each one promises its callers.
* [MCP Contract](public/contract-003-mcp-sokf.md) - the SOKF knowledge served to agents — four read-only tools over stdio, and what each one promises.
* [Configuration Contract](public/contract-004-config-superdev.md) - what a managed repo supplies to superdev — the manifest keys, the three environment variables, and which source defines what.
* [Pack Format Contract](public/contract-005-file-format-pack.md) - what a content pack must look like for superdev to read it — pack.toml, the tree that names each item, and what is refused.
* [Lock Format Contract](public/contract-006-file-format-lock.md) - what superdev records of the last apply — the per-capability components, the file hashes, the resolved packs — and what a reader may conclude from it.

## Private

* [Content Packs Interface Contract](private/contract-001-interface-content-packs.md) - the interfaces build codes against for externally sourced content packs — the manifest and lock schemas, the pack format, the resolver, the content set components read from, and the Ctx change that keeps planning pure.
