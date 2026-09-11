---
type: Architecture
id: architecture
title: Architecture
description: The core/binary/blueprint layering, the capability-to-provider map, the knowledge-serving side, and the files superdev keeps in a managed repo.
status: stable
links:
  - rel: relates-to
    to: software-components
    note: The crates and packages these layers ship as.
---

superdev runs inside a target repo and keeps that repo's agent-development
setup current. Three layers:

- **`superdev-core`** — the domain: the manifest, the components, planning,
  the engine that applies a plan, and the `sokf` subsystem that reads and
  safely mutates the knowledge.
- **`superdev` (binary)** — argument parsing, output rendering, exit codes.
- **The blueprint** — superdev's opinion of a managed repo, compiled into the
  binary: the component set plus a registry of default versions tested
  together. The binary's version is the blueprint version.

# Serving the canonical knowledge

Installing the `knowledge` capability is half of it; the other half is reading
it back. The `sokf` subsystem parses and indexes the SOKF knowledge. A shared service
serves it through MCP (`superdev mcp sokf`) and the `superdev sokf overview`,
`search`, `read` and `graph` CLI commands — so an agent queries the knowledge
instead of preloading every concept. The same service backs CLI `edit` and
`write`: it resolves identities, applies atomic agent-safe mutations, runs the
validator's repair pass, and reports the final validation state plus requested
and generated diffs. MCP exposes those two mutations only under the agent-safe
policy; the deliberate human override remains CLI-only. The `validate`
subsystem remains the check both paths share. The MCP tools are in
[contract-003-api-sokf][sokf:contract-003-api-sokf]. The server serves three
read-only tools — `sokf_search`, `sokf_graph` and `sokf_overview` — and serves
no file tool: an agent reads and edits knowledge with its own `read`, `edit`
and `write` on physical paths, and every result the server returns carries the
repository-relative path of the concept it names
([ADR-055][sokf:adr-055-sokf-does-not-intercept-file-tools]).
Pi's project extension registers those three tools and nothing else. It lazily
starts one MCP process per repository and reuses it for the Pi session, while
leaving every file operation to Pi. That repository is the canonical active
checkout root: any `.git` directory or worktree pointer file on the upward walk
wins, and `.superdev/config.toml` selects the root only when the walk finds no
`.git` marker at all. Calls are serialized on that root. Every turn ends with
one unconditional `validate --fix`, so a write through `bash` or a patch is
covered as a tool call is; the run re-reads the tree before reporting and sends
at most one message. Format-sensitive authoring instructions
remain out of the standing prompt and load from the `sokf-authoring` skill on
demand. Freshness is lazy: search and the overview re-hash the
canonical knowledge and sync only what changed. The process initializes its
embedder on the first such call and reuses it; graph
traversal parses current files without opening the index or loading embeddings.
No repository daemon persists beyond the Pi session.

# Content resolves before planning

Every skill, scaffold and document template a component writes comes from one
resolved content set, built before any component plans and handed to them
through `Ctx`. Nothing is fetched or read during planning, which is what keeps
`plan` side-effect free and `status` free
([ADR-002][sokf:adr-002-resolve-before-plan]). A component asks the set
for the items it owns rather than carrying a list of them, so what superdev
ships is decided by the pack tree — see
[directory-structure][sokf:directory-structure] for its shape and
[ADR-003][sokf:adr-003-items-by-layout] for the rules that name an item.

Layer 0 is the pack compiled into the binary; each `[[packs]]` entry layers
over it in manifest order, and a later item of the same identity wins. A pin
naming exactly what the binary embeds resolves from it and makes no request,
whichever way the source is spelled. A local-path source is read from disk
every run, so editing it and syncing again lands the new bytes with no
rebuild.

# Capabilities and providers

A capability is a slot; the tool filling it is a swappable provider.

| Capability   | Provider          | Delivered as                    |
|--------------|-------------------|---------------------------------|

| `code-index` | `codegraph`       | checksummed release bundle (mise `http`) + `mise exec http:codegraph -- codegraph init` |
| `frontend`   | `frontend-design` | optional frontend plugin        |

The registry holds one entry per (capability, provider) pair — its version, its
checksum where it has one, and whether it is the default — and the manifest
picks among them with one `provider` field per slot, as described by
[configuration][sokf:configuration]. Every active capability has one
registry entry; an id no entry matches fails with `<capability> provider must
be one of: …`. A manifest still naming the removed `workflows` capability fails at
load with a guided error.

`code-index` is fetched by URL and verified against a checksum this binary
carries beside the version, so superdev installs the registry version and
refuses any other — see [contract-002-cli-superdev][sokf:contract-002-cli-superdev]. codegraph's bundles
vendor their own Node, so a managed repo needs no node of its own.

The first-party pack carries the owned `.pi/extensions/superdev/` package and
the independently invocable `.pi/skills/sokf-authoring/` skill. The workflow
role prompts are private extension files, not discoverable Pi skills. Rust's
content layout gives both Pi asset kinds ordinary lock hashes, drift reporting,
adoption, rewrite, and orphan cleanup. Retired Claude skills and hook settings
exist only under `archive/claude-code/`; sync cannot recreate them.

# Files and artefacts

The files superdev writes into a managed repo, and what each is for.

`.superdev/config.toml` records what the repo wants and `.superdev/lock.toml`
what superdev last applied; both are committed. `.superdev/cache/` holds
machine state and is gitignored. Their shape is in
[configuration][sokf:configuration]; the code implementing them is listed in
[software-components][sokf:software-components].

Two things superdev keeps are lines in files it does not own: the `.gitignore`
entries for machine state and `@.agents/superdev.md` in AGENTS.md. They are
added when missing, never rewritten, and never hashed.

AGENTS.md is the user's file: superdev's guidance sits behind that one import
in the owned `.agents/superdev.md`. The source is
`crates/lib/superdev-core/src/agent-instructions.md`; the pipeline expands its
code-index region only when that capability is enabled. Rust owns composition,
not the instruction prose. The enabled code-index capability also carries the
`mcpServers.codegraph` registration that serves the index over MCP.

Migrations are derived, not scripted: what the lock records minus what the
components claim is what `sync` removes, so a dropped file, a rename's old copy
and a disabled capability's pins all follow one rule.

<!-- sokf:links -->
[sokf:adr-002-resolve-before-plan]: /knowledge/adrs/active/adr-002-resolve-before-plan.md
[sokf:adr-003-items-by-layout]: /knowledge/adrs/active/adr-003-items-by-layout.md
[sokf:adr-055-sokf-does-not-intercept-file-tools]: /knowledge/adrs/active/adr-055-sokf-does-not-intercept-file-tools.md
[sokf:configuration]: /knowledge/configuration.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-003-api-sokf]: /knowledge/contracts/public/active/contract-003-api-sokf.md
[sokf:directory-structure]: /knowledge/directory-structure.md
[sokf:software-components]: /knowledge/software-components.md
