# Public Contracts

What this repository promises to callers outside it. One document per
contract, each with its own stability promise.

* [CLI Contract](contract-002-cli-superdev.md) - the superdev command line — the manage verbs, the knowledge verbs, and what each one promises its callers.
* [MCP Contract](contract-003-mcp-sokf.md) - the SOKF knowledge served to agents — four read-only tools over stdio, and what each one promises.
* [Configuration Contract](contract-004-config-superdev.md) - what a managed repo supplies to superdev — the manifest keys, the three environment variables, and which source defines what.
* [Pack Format Contract](contract-005-file-format-pack.md) - what a content pack must look like for superdev to read it — pack.toml, the tree that names each item, and what is refused.
* [Lock Format Contract](contract-006-file-format-lock.md) - what superdev records of the last apply — the per-capability components, the file hashes, the resolved packs — and what a reader may conclude from it.
