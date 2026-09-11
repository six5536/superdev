---
type: SoftwareComponents
id: software-components
title: Software Components
description: The Rust crates, the npm launcher and platform packages, the platform matrix, and the CI/CD workflows.
status: stable
links:
  - rel: relates-to
    to: architecture
    note: The design these components implement.
---

The system is one Rust library plus one binary (workspace globs
`crates/lib/*`, `crates/app/*`), and six npm packages. The design they
implement is in [architecture][sokf:architecture].

# `crates/lib/superdev-core` (library)

All domain logic; no argument parsing. One module per concern:

- `capability` — the four slots; `registry` — their default providers and
  versions, baked into the binary. The SOKF knowledge is not among them:
  it is part of superdev, not a slot a provider fills.
- `manifest` / `lock` — `.superdev/config.toml` and `.superdev/lock.toml`.
- `component` — the provider trait (`plan` observes and returns actions);
  `action` — the action enum and file ownership.
- `components::{sokf, plugin, skillpack, codegraph}` — the SOKF component
  and legacy/optional providers, plus shared `mise`, pin, item, and enabled
  planning helpers. The first-party workflow no longer routes through the
  legacy Claude skillpack component. `item` is the declarative list components
  derive both `plan` and `owned` from — and `enabled`, the
  manifest-to-component resolution.
- `pack` — where content comes from: the source a pack is resolved from, the
  normalised identity that decides replace-versus-layer, `pack.toml` with the
  paths and keys a pack may not carry, and `resolve` — the phase that turns
  the manifest's entries into a content set before anything plans. Depends on
  `content`; `content` never depends on it, and neither knows about
  components.
- `content` — what a pack provides: `Item` and the `(owner, kind, name)`
  identity a later layer supersedes on, the layout rules that read that
  identity out of a pack tree, and the `ContentSet` a run resolves to. Depends
  on nothing but `std` and `capability`, and never on how a pack is fetched.
  The `sokf` and `skillpack` components and the general-rules scaffolds read
  their items from the set through `Ctx`, so adding a file to `/pack` ships it
  with no Rust edit. What the binary owns rather than the pack stays a
  constant: the canonical knowledge instructions and the SOKF spec describe a version it
  pins and a format its compiled validator enforces, as codegraph's
  instruction file does.
- `pipeline` — the verb pipeline between manifest and engine: `plan_repo`
  and `apply_repo`, owning the prune-before-plan and orphans-last ordering.
- `engine` — applies a plan and unwinds on failure, one file per concern:
  `tx` (the journal every side effect goes through), `pins` (the grouped
  mise pin phase), `materialise` (the skill copier), with the appliers in
  `apply`. `orphan` — plans the sweep of lock entries no live claim covers.
- `runner` — the process seam. `run_with` is its one required method, taking
  a `RunOptions` that carries a deadline and extra environment; `run` defaults
  onto it with neither, so a caller wanting only a command writes what it
  always did ([ADR-015][sokf:adr-015-the-spawn-seam-carries-a-deadline]).
  An expired deadline is an `Error::Command` like any other failed spawn.
  `report` — plan and apply rendering; `error` —
  the crate's error type; `fsutil` and `json_edit` — the pure file and
  JSON-pointer helpers the engine and planners share.
- `templates` — the project templates: token substitution, the init-only
  scaffold plan, and `rust_npm`, the embedded table mapping
  `assets/projects/rust-npm/` onto tokenised target paths.
- `sokf` — retrieval and safe mutation of the SOKF knowledge, one module per
  stage: `concept` (frontmatter and section parsing), `bundle` (loading and
  reserved-file rules), `graph` (link resolution and inverse synthesis),
  `embed` (the embedding providers), `index` (tantivy plus the vector store),
  `mutation` (exact edits, whole writes, policy, atomic persistence, repair
  and mutation results), and `mcp` (the shared service and its server adapter).
- `validate` — the check, both halves meeting in the parent so neither
  imports the other: `sokf` (the specification's document checks) and
  `schema` (documents against the schema their `type` names, and skills and
  the core file against the grammar, with `document`, `check`, `grammar`,
  `read` and `doc` beneath it).
- The SOKF spec, agent files, starter concept skeleton, Pi extension and skill,
  schemas, and project templates live in `/pack` at the repository root,
  reached from the crate as `assets/` through a
  symlink and embedded at compile time. `superdev-core/build.rs` enumerates
  that tree into the file list `content` reads, so a file added to the pack
  reaches the binary without a Rust edit; the contents are still `include_str!`
  literals, and only the list of them is generated.

The MCP server exposes four retrieval tools over stdio — `sokf_search`,
`sokf_resolve_source`, `sokf_retrieve` and `sokf_graph` — plus
mandatory-agent-safe `sokf_edit` and `sokf_write` (see
[contract-003-api-sokf][sokf:contract-003-api-sokf]).
`sokf_resolve_source` answers where a source is, not what it says: it takes an
unqualified `sokf:<id>` or a physical path entering `knowledge/` and returns the
ingress path, the canonical repository-relative target, whether it exists, and
its generated regions. `sokf_retrieve` owns the semantic side, taking `sokf:`,
`sokf:<id>` or `sokf:<id>#<heading>` with a line window and refusing physical
paths. The server treats the canonical active checkout root as its repository,
including when `.git` is a worktree pointer file, and refuses a path entering
another checkout. It holds one index directory and serialises its
tool calls: a call keeps the index open while another call's sync could rebuild
that directory. Search is hybrid — tantivy BM25 and cosine over section
embeddings, fused by reciprocal rank fusion — and drops to lexical-only when no
model loads. MCP initializes the configured embedder on the first search or
semantic overview retrieval and reuses that result for the process lifetime.

# `crates/app/superdev` (binary)

Depends on `superdev-core`. Binary name `superdev`. `main.rs` is clap parsing
and exit codes; `manage.rs` holds the `init`, `status`, `sync` and `update`
verbs — each loads, calls the core pipeline, renders its lines and turns
its facts into an exit code. `template_select.rs` decides init's project
template: flags and TTY-ness feed logic behind a `Prompter` trait, with the
dialoguer adapter as untested glue. `validate_cli.rs` holds `validate` and the hook. `sokf_cli.rs` holds the
`sokf` retrieval, mutation and index commands plus `mcp sokf`: request parsing,
path defaults, human and JSON output, and the current-thread tokio runtime the
server blocks on. Also present is the plumbing the release
pipeline needs:
`--version`, `completions` (clap_complete), and a hidden `man` subcommand
(clap_mangen). The CLI contract is in [contract-002-cli-superdev][sokf:contract-002-cli-superdev].

# Pi project extensions

`.pi/extensions/sokf.ts` is a transport adapter over the shared MCP service,
not a second SOKF implementation. It overrides Pi's `read`, `edit`, and `write`
slots only for virtual `sokf:` addresses or physical paths under the
repository's knowledge root, delegates all other paths to fresh built-in tool
instances rooted at the current working directory, and registers `sokf_search`
and `sokf_graph`. Routed `read` spreads Pi's own read definition, resolves the
argument through `sokf_resolve_source`, and executes Pi's read factory against
the canonical target, so schema, truncation, errors, details and rendering stay
Pi's; the `sokf:` overview and section addresses are refused with guidance
naming `sokf_search`. `.pi/extensions/sokf-mcp.ts` lazily starts and initializes one
narrow MCP stdio client per repository, serializes calls, reuses the child for
the session, bounds protocol diagnostics, restarts after failure and closes it
at session shutdown. Mutation responses convert standard MCP
`structuredContent` into Pi's built-in result shapes. The extension walks to
the canonical active checkout root before spawning `superdev`, so it also works
when Pi starts in a subdirectory or a linked worktree: a `.git` directory or
pointer file anywhere on the walk wins, and `.superdev/config.toml` selects the
root only when no `.git` marker exists. After a turn mutates knowledge, the extension runs final
validation through a one-shot CLI call. A failure queues at most two repair
turns with the validator report; a persistent failure stops automatic feedback
and waits for manual continuation.
`scripts/test/sokf-pi-adapter.test.mjs` loads the real extension through the
pinned Pi 0.85.1 test dependency. Its sandbox checks routed retrieval,
structured edit results, subdirectory physical writes, applied-invalid
handling, and bounded validation feedback without making a model call. Its
paired harness runs every read case twice — once through Pi's built-in read
against the physical file, once through the routed tool against the equivalent
SOKF argument — and requires the two results, errors and details to agree
exactly.

`.pi/extensions/superdev/index.ts` registers `/superdev`,
resume, cancellation, and human-only abandonment. Its
private Markdown prompts are appended to fresh child Pi processes with fixed
modifying or read-only tool sets. The extension delegates all durable changes
to the versioned Rust workflow CLI.

`.pi/skills/sokf-authoring/SKILL.md` is a genuine independently invocable Pi
skill around SOKF-aware tools and is mirrored in `pack/pi/skills/`.
Superdev's `file`, `scope`, `build`, and `accept` skills live under
`.pi/extensions/superdev/skills/`, mirrored in `pack/pi/extensions/superdev/skills/`.
The extension registers that directory through `resources_discover` on startup
and reload; Pi exposes the resources as native `/skill:*` commands and model
prompt entries. No copies live in the general `.pi/skills/` directory.
The `scope`, `build`, and `accept` skills invoke the typed workflow tools. The native
`file` skill lets the LLM choose an issue or idea number, author and validate the
record, and commit it on the default branch through ordinary tools, using a
worktree when elsewhere. Filing has no dedicated command, tool, or child role. All former Claude workflow skills and
providers are retained only under `archive/claude-code/`.

`.pi/extensions/system-prompt.ts` registers `/system-prompt` for inspecting the
effective Pi prompt. Its `.pi/current-system-prompt.md` output is machine-local
and ignored rather than project knowledge.

# SOKF behavior evaluations

`evals/sokf/behavioral.json` carries the versioned prompts, expected tool
sequences, forbidden operations, and acceptance threshold for the 12
progressive-context scenarios. The fixtures distinguish knowledge-worthy tasks
from code-local work and cover missing stores, format-sensitive work,
intermediate invalid mutations, and outward-facing documentation.
`scripts/sokf-eval.mjs` runs isolated Pi model sessions with or without the
standing instruction, scores tool sequences and forbidden operations, and
emits `sokf-evaluation-result/v1`. Manual outcome labels remain in the report
for semantic review. `scripts/test/sokf-behavior-fixtures.test.mjs` protects
the fixture protocol and scenario roster; the structural test alone does not
claim behavioral success.

# Publishing

`superdev-core` and `superdev` publish to crates.io. The compiled binary is
also redistributed via npm.

# npm (prebuilt-binary model, à la esbuild / `@swc`)

```
packages/
  superdev/              # published as superdev — launcher (bin: superdev)
  superdev-linux-x64/    # published as @six5536/superdev-linux-x64 — prebuilt binary, declares os/cpu
  superdev-linux-arm64/
  superdev-darwin-x64/
  superdev-darwin-arm64/
  superdev-win32-x64/    # superdev.exe
```

- The launcher (`superdev`) declares each platform package in
  `optionalDependencies` pinned to an **exact** version; npm installs only the
  host's match.
- A small JS shim `require.resolve`s the installed platform package's binary
  and `spawnSync`s it with `stdio: "inherit"`, forwarding `argv` and the exit
  code.
- **Version lockstep**: launcher + all platform packages share one version and
  publish atomically.
- **Unsupported platform** (no matching optional dep): fail with a message
  that lists the supported platforms and points at `cargo install superdev`.
  No auto-download or build-from-source fallback. The Linux packages are
  static musl builds, so they cover glibc and musl hosts alike — libc is not a
  dimension of this matrix.

# Platform matrix

Linux `x86_64`/`aarch64` (**static musl**), macOS `x86_64`/`aarch64`, and
Windows `x86_64` (msvc, built natively on the `windows-latest` runner).
`cargo-zigbuild` provides the cross C compiler for the musl targets; its musl
output is non-PIE, accepted for a local CLI with no network input.

# CI/CD

The workflows under `.github/workflows`.

All checks live in a reusable `workflow_call` workflow (`checks.yml`), called
by both `ci.yml` and `release.yml`, so the release gate cannot drift from CI.

- **`checks.yml`**: `cargo fmt --check`, `clippy -D warnings`, `nextest`,
  doctests, `cargo doc -D warnings`, the npm launcher tests, version
  consistency, the canonical knowledge and format validation (`check:validate`), the
  per-crate coverage gate, and `cargo-deny` for licences/bans/sources. Tests
  and doctests also run on Windows; the OS-independent checks run once, on
  macos.
- **`ci.yml`**: calls `checks.yml` on push and PR.
- **`audit.yml`**: scheduled `cargo-deny check advisories`, opening an issue
  rather than failing builds — advisories are exogenous and must not block an
  unrelated PR.
- **`release.yml`** (tag `v*`): verify the tag against every version in the
  tree and against a `CHANGELOG.md` section → run `checks.yml` in full →
  build the five binaries (cross for musl, native for macOS and Windows),
  assert the Linux ones are static, smoke-test each binary the runner can
  execute (`release-smoke.mjs`; linux-arm64 cannot run on the x64 runner) and
  run the packed launcher end-to-end where the package matches the host →
  dry-run every publish → publish platform packages, then the launcher, then
  `cargo publish --workspace --locked` → create a GitHub Release with
  archives (`.tar.gz`; `.zip` for Windows), a man page, completions and
  `SHA256SUMS`. Prerelease tags publish under the npm `next` dist-tag and are
  flagged as prereleases.

Cross-registry atomicity is impossible, so the guarantee is *ordered,
dry-run-gated and recoverable* rather than truly atomic.

<!-- sokf:links -->
[sokf:adr-015-the-spawn-seam-carries-a-deadline]: /knowledge/adrs/active/adr-015-the-spawn-seam-carries-a-deadline.md
[sokf:architecture]: /knowledge/architecture.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-003-api-sokf]: /knowledge/contracts/public/active/contract-003-api-sokf.md
