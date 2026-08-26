---
type: Reference
id: configuration
title: Configuration & Environments
description: The .superdev directory — the config.toml manifest, the lock file, and the gitignored cache — plus the embeddings opt-in, the custom lists, the many-provider skills shape, the guided errors, the .mcp.json and .claude/settings.json merges, the bash-output-filter files, and the user-level model cache.
status: stable
resource: /crates/lib/superdev-core/src/manifest.rs
---

Everything superdev keeps in a managed repo lives under `.superdev/`.

# `config.toml` — the manifest

What the repo wants. Committed and hand-editable; superdev rewrites it on
`update` and to stamp `blueprint`. `init` writes it from the registry
defaults.

```toml
blueprint = "0.1.0"      # the superdev version last applied

[bash-output-filter]
provider = "rtk"
version = "0.45.0"

[code-index]
provider = "codegraph"
version = "1.5.0"

[frontend]
provider = "frontend-design"

[knowledge]
provider = "aokf"

[skills]
provider = "superdev-skills"
version = "0.1.0"
```

A `[[packs]]` array names the content packs to layer, in layer order, and is
absent from the manifest above because absence is the default:

```toml
[[packs]]
source = "github:six5536/superdev"   # git: a rev is required
rev    = "assets-v1.4.0"

[[packs]]
source = "./packs/acme"              # a path on this machine: no rev
```

It is a top-level array rather than a capability table because the two
absences differ: an absent capability means disabled, while an absent pack
list means the pack compiled into the binary
([ADR-001](decisions/D001-packs-manifest-section.md)). Nothing about a repo
that names no pack changes, and `sync` never adds an entry to a manifest that
lacks one.

Entries layer in the order written, and a later item of the same name wins.
An entry naming the source the embedded pack is a copy of *replaces* it
rather than layering over it, so what that rev drops leaves the repo
([ADR-004](decisions/D004-base-pack-identity.md)); every other entry sits
above. Two entries naming one source are refused — each pack appears once,
as each capability provider does.
A source is compared normalised, so any spelling of one repository is one
source; a pin naming exactly what this binary embeds resolves from it and
makes no request. `github:owner/repo` and `gitlab:owner/repo` are the only
shorthands, expanded to `https://<forge>.com/owner/repo` before `git` sees
them; every other spelling reaches `git` as written, so an ssh alias, an
`insteadOf` prefix and a mirror all keep working. A path is resolved against
the repo root, not the working directory — the file is committed, so it means
the same thing wherever the command runs from — and canonicalised before it is
compared, so two spellings of one directory are one pack. Its identity is that
canonical location written relative to the repo root, with forward slashes, so
the lock it lands in reads the same in every checkout and on every platform; a
pack beside the repo keeps its `..`
([ADR-011](decisions/D011-path-pack-identity-is-root-relative.md)). A directory
and a repository are never the same source, however alike their identities
read. It is read from disk
on every run, so editing a local pack and running `sync` again lands the new
bytes with no rebuild.

`init` writes the default entry rather than leaving the array absent: both
resolve the same way, but the written pin is the one a reader can see and
edit without first knowing that absence means the embedded pack. `update` —
only its untargeted form, never `update <capability>` — asks the default
source for the newest `assets-v<major>.<minor>.<patch>` tag it carries and
moves that pin there, ahead of what the binary embeds if need be
([ADR-009](decisions/D009-update-queries-default-source.md)). That is the one
path by which a content fix reaches an unchanged binary, and the one place
superdev reaches the network without being asked to fetch something. A pin
never moves backwards and never below what the binary carries; a pin naming
any other source, or resting on a branch, a sha or a pre-release, is reported
and left alone. When the source cannot be reached, carries no release, or
carries only releases older than the pin, the run says so and the pin stays
where the binary would put it. A manifest an earlier binary wrote gains the
default entry on the first `update`, which is the only command that adds one.

A pin resting on a **candidate** content tag — `assets-vA.B.C-rc.N`, what a
binary release candidate cuts and what a repo that candidate set up is pinned
to — is the one non-release pin `update` moves. It comes forward onto a
release something vouches for: the one this binary carries, or one the source
answered with. Its own version does not count, because a candidate tag says
nothing about whether the release it is a candidate for was ever cut. A branch
or a sha stays where it is.

A pack is the files it contains. A symlink inside one is skipped rather than
read through — following it would put bytes from anywhere on the machine into
the repo as pack content — and a link standing in for the pack's own root or
its `pack.toml` is refused, because then the pack is not where it says it is.
Nothing says which links were skipped
([I009](issues/I009-a-skipped-symlink-says-nothing.md)).

A pack-provided file is superdev's on exactly the terms an embedded one is:
hashed into the lock, rewritten by `sync`, reported as drift when edited, and
released by naming it in a `custom` list. Dropping a pack entry removes its
files by the ordinary orphan rule — pruned while they still hash to the locked
value, left in place and released once the user has changed them.

`blueprint` is the version last applied, not the version that wrote the file.
A successful `sync` stamps this binary's version, rewriting `config.toml` only
when the value changes; `--dry-run` never stamps. `status` prints
`blueprint <a>, binary <b> — sync will update it` and leaves the exit code
alone, so a binary upgrade that changes nothing keeps CI green.

`provider` names the implementation filling the slot. An id the registry
does not carry fails with `<capability> provider must be one of: …`. Every
capability except skills holds exactly one provider; skills is a many
slot — additional packs are written as `[[skills]]` array-of-tables
entries, one per pack, each with its own `provider`, `version` and
`custom`. The single `[skills]` table is the one-entry case and keeps its
shape on rewrites; the array form appears only from two entries up. The
same pack listed twice, the array form on an exclusive slot, and an empty
entry list are all refused at load with guided errors
([spec](specs/S011-skills-cardinality-design.md)). A manifest still naming the
removed `workflows` capability fails at load with a guided error telling the
user to delete the table (moving any custom names to `[knowledge]`) — the
skill set now ships with the knowledge capability, and superpowers users can
`claude plugin install superpowers` by hand. The error never rewrites
`config.toml`; the manifest is the user's file
([spec](specs/S009-knowledge-carried-skills-design.md)).

An optional sub-table opts the knowledge bundle out of the local embedding
model and onto an API:

```toml
[knowledge.embeddings]
provider = "openai"              # the only provider implemented
model = "text-embedding-3-small"
```

The API key comes from `OPENAI_API_KEY` and never from the file. The recorded
model id is part of the index manifest, so switching provider rebuilds the
index by itself. Absent the table, embedding is local and offline after the
first download.

`[skills]` takes an optional `custom` list naming skills released from
management:

```toml
[skills]
provider = "superdev-skills"
version = "0.1.0"
custom = ["humanise"]
```

A released skill keeps whatever content it has as a starting point, drops out
of the plan and out of the lock, and `status` prints it as unmanaged rather
than drifted. Delete the name to get stock content back on the next sync. A
name the pack does not ship reports `skills: custom names
unknown skill '<name>' — no effect` and changes nothing, so a pack that drops
a skill does not break a repo that had marked it custom.

`init` seeds that list: a repo that already has a skill under a pack name,
with content of its own, keeps it — the name goes into `custom` and the
adoption reports it. Content byte-identical to the shipped skill is superdev's
own text and is left managed.

`[knowledge] custom` releases an aokf-carried skill the same way — the whole
skill directory, companions included — with the same `init` adoption and the
same `<capability>: custom names unknown skill '<name>' — no effect` line
for a name the capability does not ship. The lists are name-guarded: a name
in one capability's list never releases another capability's file, even
though both write into `.claude/skills/`.

A `[template]` table records the project template `init` seeded the repo
from, with the substituted token values:

```toml
[template]
name = "rust-npm"
project-name = "My Tool"
project-slug = "my-tool"
```

Provenance, not management: template files are scaffolds, so nothing here is
hashed and no verb re-plans them. Absent when init chose no template.

One table per enabled capability, keyed by the capability name — an absent
table means disabled, which is what `init --no-<capability>` produces. An
unknown capability name is rejected. `version` is omitted where the source
manages versions itself. A capability pinned below the registry default reads
as behind; a pin above it is deliberate and left alone.

# `lock.toml`

What superdev last applied. Committed, never hand-edited. It records the
provider and version applied per capability, plus a sha256 of every file
superdev owns. Entries superdev merges into a shared file are hashed under
`<file>:<pointer>` instead — `.mise.toml:<tool>` for a pin,
`.claude/settings.json:hooks.PostToolUse[<marker>]` for the hook. Drift is
found by comparing a file against the content the blueprint wants, not against
the lock; the hashes are what lets an apply tell that the file it just
overwrote had been edited by hand, and say so (after backing it up).

A `[[packs]]` entry records each pack the last apply resolved — the source as
the manifest wrote it, the normalised identity every spelling of that source
shares, the rev, a digest over the resolved tree and the `format` the pack
declared — so a later run can prove it got the same bytes. The per-file hashes
stay in `files` with everything else, which is what makes a dropped pack's
files orphans by the ordinary rule. Absent when no pack was named.

A legacy `owners` table may remain from binaries that materialised skills
from provider checkouts. Nothing writes or reads it any more: every shipped
file's name comes from the binary, so claims need no attribution. The first
sync clears the whole table — a per-file retirement would strand entries on
files that never need rewriting — and an empty table is not serialised.

An entry no component claims any more is an orphan, and `sync` prunes it.
Content that still hashes to the locked value is superdev's own residue: it is
removed, backed up like any overwrite, and restored if a later step fails.
Content the user changed is left exactly where it is, dropped from the lock,
and reported once as `orphan: <key> changed since superdev wrote it — left in
place, released from the lock`. An entry whose target is already gone leaves
the lock silently; one that cannot be read fails the run. Pins and merged JSON
keys are pruned the same way, and a disabled capability's `components` record
goes with its files.

# `cache/`

Machine state, gitignored by `init`: backups of overwritten files under
`backup/<timestamp>/`, the search index under `aokf-index/` (tantivy, the
section vectors, and a manifest of per-file hashes, schema version and model
id), and each resolved pack under `packs/<digest>/`. Deleting it is safe — the
next tool call rebuilds the index, and the next `sync` re-fetches a pack.

A pack is cached by its digest so a later run reaches the network only for
bytes this machine does not have ([ADR-005](decisions/D005-pack-cache-and-fetch.md)):
a steady-state `sync`, a CI `status --drift` and a `--dry-run` after any
previous resolve all stay offline. Only what verified against the lock's
digest is kept; a pack that failed verification leaves nothing behind.

# Outside the repo

- `.mcp.json` at the repo root registers the MCP servers: the knowledge
  bundle under `mcpServers.superdev-aokf`, and — with code-index enabled —
  codegraph's under `mcpServers.codegraph`, launched through `mise exec`
  because the pinned binary is on no PATH the client can see. The file is
  shared with the user's own servers, so superdev manages and hashes its
  own keys and leaves the rest alone, the same rule it applies to
  `.mise.toml`. A managed repo gets `superdev mcp aokf`; in this repo the
  dev shim makes that resolve to `cargo run` against the working tree.
- AGENTS.md carries one ensured line, `@.agents/superdev.md`, and is
  otherwise the user's. The aggregator it imports and the per-capability
  instruction files beside it (`.agents/aokf.md`, `.agents/codegraph.md`)
  are owned files; the general rules (`.agents/professionalism.md`, `.agents/process.md`,
  `.agents/coding.md`) are write-once scaffolds, the user's to adapt.
- `.claude/settings.json` carries one managed `hooks.PostToolUse` element,
  owned by the knowledge capability (the hook validates the bundle, so it
  exists exactly where a bundle does): superdev finds its own element by the
  command string `superdev aokf hook validate`, adds or updates it, and
  leaves the user's hooks alone. Both files are re-serialised whole on
  write, so key order is not preserved; the lock hashes the merged value, not
  the file, so a reformat is not drift. The bash-output-filter capability
  manages one `hooks.PreToolUse` element the same way, found by its
  `mise exec http:rtk -- rtk hook claude` command string; the hook rewrites
  Bash commands through rtk and is fail-open — only exit code 2 blocks a
  command in Claude Code, and rtk's hook exits 0 on every failure path.
  One caveat rides the capability: permission allow/deny rules match the
  rewritten, rtk-prefixed command string (`rtk git status`, not
  `git status`), so string-matched rules should account for both forms;
  rtk's own per-command exclusion config is the opt-out.
- The bash-output-filter capability owns three whole files: `.miserc.toml`
  (turns on mise's `auto_env`, which needs mise ≥ 2026.8 — an older
  observed mise gets a guided error at plan time) and the platform-scoped
  pin files `mise.unix.toml` and `mise.windows-x64.toml`, holding the
  checksummed rtk pin for exactly the platforms rtk publishes. A platform
  without an artefact (windows-arm64) loads neither file and skips the tool
  silently — which is why the pin is not in the shared `.mise.toml`. Its
  agent guidance, `.agents/rtk.md`, tells agents output is auto-filtered
  and how to get raw output (`RTK_DISABLED=1` in the command text, or
  rtk's proxy passthrough)
  ([spec](specs/S012-bash-output-filter-design.md)).
- `.claude/skills/` holds the pack's three skills and the knowledge
  capability's 25 carried skill directories as owned files, plus the MIT
  notice for the derived set. A `PROJECT.md` beside a skill extends it —
  superdev never writes, hashes or reads that file, so a project layer
  survives every sync.
- The local embedding model lives in the *user* cache, not the repo:
  `$XDG_CACHE_HOME` (else `%LOCALAPPDATA%`, else `~/.cache`) +
  `/superdev/models/<model>/<revision>/`. Revision-scoped, so a pin bump
  downloads afresh instead of overwriting. One ~130 MB download serves every
  repo on the machine; see [technology-stack](technology-stack.md) for the pin
  itself.

The capability set is in [architecture](architecture.md); the file-ownership
rules that decide what gets hashed are in [glossary](glossary.md).
