# Public Contracts

What this repository promises to callers outside it. One document per
contract, each with its own stability promise.

* [CLI contract for superdev][sokf:contract-002-cli-superdev] - the superdev command line — every command, argument and flag as clap declares it, the exit codes and streams each command promises, and what may change.
* [API contract for sokf over MCP][sokf:contract-003-api-sokf] - the SOKF knowledge served to agents — four read-only tools over stdio as the server declares them, and what each call promises beyond its signature.
* [Configuration Contract][sokf:contract-004-config-superdev] - what a managed repo supplies to superdev — the manifest keys, the three environment variables, and which source defines what.
* [Pack Format Contract][sokf:contract-005-text-format-pack] - what a content pack must look like for superdev to read it — pack.toml, the tree that names each item, and what is refused.
* [Lock Format Contract][sokf:contract-006-text-format-lock] - what superdev records of the last apply — the per-capability components, the file hashes, the resolved packs — and what a reader may conclude from it.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-003-api-sokf]: /knowledge/contracts/public/active/contract-003-api-sokf.md
[sokf:contract-004-config-superdev]: /knowledge/contracts/public/active/contract-004-config-superdev.md
[sokf:contract-005-text-format-pack]: /knowledge/contracts/public/active/contract-005-text-format-pack.md
[sokf:contract-006-text-format-lock]: /knowledge/contracts/public/active/contract-006-text-format-lock.md
