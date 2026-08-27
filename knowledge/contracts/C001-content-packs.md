---
type: Contract
id: contract-content-packs
title: Content Packs Interface Contract
description: The interfaces build codes against for externally sourced content packs — the manifest and lock schemas, the pack format, the resolver, the content set components read from, and the Ctx change that keeps planning pure.
status: draft
links:
  - rel: implements
    to: spec-content-packs
---

# Interface contract: externally sourced content packs

The interfaces [S014](../specs/S014-content-packs-design.md) adds: a
`[[packs]]` manifest section ([ADR-001](../decisions/D001-packs-manifest-section.md)),
a resolver that runs before planning and hands components their content
through `Ctx` ([ADR-002](../decisions/D002-resolve-before-plan.md)), a content
set keyed by items the pack layout names
([ADR-003](../decisions/D003-items-by-layout.md)), source identity that
decides replace-versus-layer ([ADR-004](../decisions/D004-base-pack-identity.md),
[ADR-011](../decisions/D011-path-pack-identity-is-root-relative.md)),
and a machine-local cache with on-demand fetching
([ADR-005](../decisions/D005-pack-cache-and-fetch.md)). The decisions taken
against the issues acceptance left open are folded in where they land: the
transport allowlist
([ADR-012](../decisions/D012-pack-source-schemes-are-allowlisted.md)), the
pin `update` proves before it writes
([ADR-013](../decisions/D013-update-proves-a-pin-before-it-writes-it.md)),
the symlink refusal
([ADR-014](../decisions/D014-a-symlink-in-a-pack-is-refused.md)), the
deadline on the process seam
([ADR-015](../decisions/D015-the-spawn-seam-carries-a-deadline.md)) and the
digest a path pack no longer records
([ADR-016](../decisions/D016-a-path-pack-records-no-digest.md)).
A working document:
build codes against it and it is discarded once the code is canonical.

## Data model & API

### Manifest — `.superdev/config.toml`

```toml
blueprint = "0.3.0"

# Layer order. Absent entirely = resolve from the embedded snapshot.
[[packs]]
source = "github:six5536/superdev"
rev    = "assets-v1.4.0"

[[packs]]
source = "./packs/acme"          # local path: no rev

[knowledge]                       # capability tables: unchanged
provider = "aokf"
```

```rust
/// One content pack the repo wants. Order in the manifest is layer order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackEntry {
    /// Where the pack comes from, as the user wrote it.
    pub source: String,
    /// Git revision — tag, branch or commit sha. Absent for a path source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
}

pub struct Manifest {
    pub blueprint: String,
    /// Empty when the manifest carries no `[[packs]]`, which resolves from
    /// the embedded snapshot — never "disabled". ADR-001.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<PackEntry>,
    pub capabilities: BTreeMap<String, Vec<CapabilityConfig>>,
    pub template: Option<TemplateConfig>,
}
```

### Pack source and identity

```rust
/// Where a pack is resolved from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackSource {
    /// A git repository at a revision.
    Git { url: String, rev: String },
    /// A directory on this machine; relative paths resolve against the repo
    /// root.
    Path { path: PathBuf },
}

impl PackSource {
    /// Parse one manifest entry. Rejects a git source carrying no `rev`, a
    /// path source that names one, and a source or rev beginning with `-`,
    /// which git reads as an option wherever it meets it. ADR-004, I007.
    ///
    /// Rejects a transport outside [`SUPPORTED_SCHEMES`], naming the source
    /// and the transport, before anything is spawned — and a
    /// `<name>::<address>` remote helper as one, whatever its address, since
    /// a helper names a program rather than a protocol. The `github:` and
    /// `gitlab:` shorthands are https and the scp form is ssh, so neither
    /// spelling changes. ADR-012.
    pub fn parse(entry: &PackEntry) -> Result<PackSource>;

    /// The comparison key: scheme, userinfo, a `.git` suffix and any trailing
    /// slash removed, host and path lowercased. Every spelling of one
    /// repository shares a key, so `github:six5536/superdev`,
    /// `https://github.com/six5536/superdev.git` and the ssh form are one
    /// source.
    ///
    /// A path source's key is its canonicalised path taken relative to
    /// `root`, with forward slashes — `./pack` and `pack/` are both `pack` —
    /// so the lock that records it reads the same in every checkout and on
    /// every platform. A pack outside the root keeps its `..` prefix, and
    /// where no relative form exists (a different Windows drive) the
    /// canonical absolute path stands. `root` is what makes that possible and
    /// is why it is a parameter: a path identity means nothing without the
    /// repository it was taken from. A git source ignores it.
    ///
    /// Keys never compare across source kinds: a directory and a repository
    /// are different sources however alike their keys read, so every
    /// comparison is within a kind. Without that a directory named
    /// `github.com/six5536/superdev` would key as the base pack and silently
    /// replace the embedded content. ADR-004, ADR-011.
    pub fn identity(&self, root: &Path) -> String;
}

/// The pack this binary embeds and defaults to; a manifest entry whose
/// identity matches replaces the snapshot instead of layering over it.
pub struct DefaultPack {
    pub source: &'static str,
    pub rev: &'static str,
}
pub const DEFAULT_PACK: DefaultPack = DefaultPack { /* … */ };

/// The transports a pack may be fetched over. `parse` refuses anything else,
/// naming the source, and no config on the machine can lift that.
///
/// The git overrides carry the same set as `protocol.<name>.allow=always`
/// over a `protocol.allow=never` default, plus `never` naming `git`, `http`
/// and `ext`. Both halves are needed and neither is sufficient: `parse`
/// cannot see a `url.<base>.insteadOf` rewrite, which turns an approved
/// `https://` source into whatever the machine's config says; and among the
/// overrides only the named `never` lines outrank a user config, since git
/// resolves `protocol.<name>.allow` ahead of `protocol.allow` whatever their
/// sources. ADR-012.
pub const SUPPORTED_SCHEMES: &[&str] = &["https", "ssh", "file"];
```

### Pack format — `pack.toml` at the pack root

```toml
format      = 1                    # refused when the binary does not know it
name        = "superdev-assets"
version     = "1.4.0"
description = "superdev's stock skills, templates and scaffolds"
```

```rust
/// A pack's own manifest. Unknown keys within a known format are ignored;
/// an unknown `format` is refused before any file is read.
#[derive(Debug, Clone, Deserialize)]
pub struct PackManifest {
    pub format: u32,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Formats this binary can read. An entry outside the set fails with
/// `pack `<name>` declares format <n>; this superdev supports <set>`.
pub const SUPPORTED_FORMATS: &[u32] = &[1];
```

### Items — what a pack provides

A pack's tree names all three parts of an item's identity — the owning
capability, the kind, and the name. `<name>` is the entry directly under the
kind directory; where the table shows `/**` that entry is a directory and the
item is its whole subtree:

```
pack.toml
knowledge/skills/<name>/**       → .claude/skills/<name>/**        owned
knowledge/concepts/<name>        → knowledge/<name>                scaffold
knowledge/templates/<name>.md    → knowledge/templates/<name>.md   owned
skills/<name>/**                 → .claude/skills/<name>/**        owned
agents/<name>.md                 → .agents/<name>.md               scaffold
projects/<name>/**               → repo root, tokenised            scaffold
```

```rust
/// Which capability materialises an item, or none for the repo-level kinds.
/// Part of an item's identity because two capabilities both write into
/// `.claude/skills/` and their `custom` lists are name-guarded. ADR-003.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Owner {
    Capability(Capability),
    Repo,
}

/// The kinds of content a pack may carry, each named by where it sits under
/// its owner directory. ADR-003.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemKind {
    /// `<owner>/skills/<name>/**` — owned files under `.claude/skills/`.
    Skill,
    /// `knowledge/concepts/<name>` — a write-once knowledge scaffold. `<name>`
    /// is any entry directly under `concepts/`, file or directory, and the
    /// subtree mirrors the repo's `knowledge/`: the canonical knowledge ships skeletons
    /// that are not one `.md` each — `manifest.sokf.yaml`, and the
    /// `plans/` and `specs/` index directories. ADR-010.
    KnowledgeSkeleton,
    /// `knowledge/templates/<name>.md` — an owned document template.
    DocTemplate,
    /// `agents/<name>.md` — a write-once general-rules scaffold.
    AgentScaffold,
    /// `projects/<name>/**` — write-once repo scaffolds, token-substituted.
    ProjectTemplate,
}

/// One item and every file it owns, paths relative to the item's own root.
/// `(owner, kind, name)` is the identity a later layer supersedes on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub owner: Owner,
    pub kind: ItemKind,
    pub name: String,
    pub files: Vec<(String, String)>,
}

/// Paths a pack may not carry, refused at resolve with the reason. The
/// instruction files and the AOKF spec move with the binary that pins or
/// validates them; `PROJECT.md` is the project's own extension layer, which
/// superdev never writes or tracks.
pub const REJECTED: &[&str] = &[
    "agents/aokf.md",
    "agents/codegraph.md",
    "agents/rtk.md",
    "knowledge/agents/SPEC.md",
];

/// Refused wherever it appears: the project's own extension layer, which
/// superdev never writes or tracks.
pub const REJECTED_BASENAME: &str = "PROJECT.md";
```

A symlink anywhere in a pack is refused the same way, naming the path: a pack
resolves whole or not at all, and an item silently missing is the failure
`read_pack` says it does not have. What counts as one is decided by whoever
knows. For a fetched pack it is git's index — mode `120000`, asked for after
the checkout and before anything is read, because on Windows without
`core.symlinks` git materialises a link as a plain file the filesystem cannot
tell from content, and the same rev would otherwise digest differently there.
Mode `160000`, a submodule, is refused with it: a shallow sparse clone leaves
it empty. For a path pack there is no index and no second platform, so
`symlink_metadata` decides. The filesystem check also stays at read time,
which is where the cache is read and has no index of its own. ADR-014.

### The resolved content set — what components read

```rust
/// Which layer an item came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// The binary's embedded snapshot.
    Snapshot,
    /// A pack, by its manifest index and name.
    Pack { index: usize, name: String },
}

/// One item a later layer hid. Reported only when both layers are packs:
/// superseding the snapshot is the ordinary case. S014, Layering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shadowed {
    pub owner: Owner,
    pub kind: ItemKind,
    pub name: String,
    pub winner: Origin,
    pub loser: Origin,
}

/// Every item the layers resolved to. Built once per run, borrowed by `Ctx`.
pub struct ContentSet { /* private */ }

impl ContentSet {
    /// One item, or None when no layer provides it.
    pub fn item(&self, owner: Owner, kind: ItemKind, name: &str) -> Option<&Item>;
    /// Every item of one kind, in name order.
    pub fn items_of(&self, owner: Owner, kind: ItemKind) -> impl Iterator<Item = &Item>;
    /// Where an item came from, for reporting.
    pub fn origin(&self, owner: Owner, kind: ItemKind, name: &str) -> Option<&Origin>;
    /// Pack-over-pack shadowing, for the report.
    pub fn shadowed(&self) -> &[Shadowed];
    /// The entry that replaced the snapshot, when one did. ADR-004.
    pub fn base(&self) -> Option<&Origin>;
}
```

### Resolution — the phase before planning

```rust
/// How far the resolver may go. ADR-002.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveMode {
    /// `status`: never fetch, never write the cache. A pin it cannot satisfy
    /// from the cache is returned as pending, not as an error.
    Offline,
    /// `init`, `sync`, `update`: fetch what is missing and populate the
    /// cache. An unsatisfiable pin is an error.
    Fetching,
}

/// What one resolve produced.
pub struct Resolution {
    /// Content from every layer that resolved.
    pub content: ContentSet,
    /// Pins `Offline` could not satisfy. Always empty under `Fetching`,
    /// which errors instead.
    pub pending: Vec<PackEntry>,
    /// One record per pack that resolved, in manifest order, for the lock.
    /// Apply writes these; nothing else knows the digest it verified.
    pub packs: Vec<PackLock>,
}

/// Resolve the manifest's packs over the embedded snapshot.
///
/// Takes the runner because a git source is fetched by spawning the user's
/// own `git` (ADR-007), and every spawn in the codebase goes through that
/// one seam so no test reaches a real network.
///
/// Reads the cache, the network (under `Fetching`) and the local paths; the
/// only phase in a run that does. Verifies each pack against the digest the
/// lock recorded for its rev and fails on a mismatch rather than applying
/// bytes nobody pinned. S014, Never substituting content.
pub fn resolve(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    lock: &Lock,
    mode: ResolveMode,
) -> Result<Resolution>;
```

### Where the pin moves

```rust
/// Bring the manifest's pack pins current, and say what happened. ADR-009.
///
/// A moved pin is resolved before it is written, and stays where it is when
/// resolution refuses — the reason is reported in the line that would have
/// announced the move. The manifest is what every later run reads and a pin
/// never moves backwards, so a pin written to content this binary cannot
/// read would leave no superdev command able to repair it. A moving pin
/// therefore fetches twice — the cache is found by the digest the lock
/// records, and apply does not write that until after the sync — so the
/// price is one extra clone on the rare run that advances a pin. The lock is
/// here because resolution reads it. ADR-013, I001.
///
/// Nothing here fails the run.
pub fn update_pins(
    runner: &dyn CommandRunner,
    root: &Path,
    manifest: &mut Manifest,
    lock: &Lock,
) -> Vec<String>;
```

### The process seam

Not `pack`'s own, but changed for it: the query `update` makes unprompted is
the first spawn in the codebase that needs a deadline, and the first that
needs an environment.

```rust
/// How a command is run: everything beyond the program and its arguments.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Kill the child and fail after this long. `None` waits as long as it
    /// takes, which is what a toolchain install needs.
    pub timeout: Option<Duration>,
    /// Extra environment for the child, over the inherited one.
    pub env: Vec<(String, String)>,
}

pub trait CommandRunner {
    /// Run `program args…` in `cwd`, capturing output.
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<Output> {
        self.run_with(program, args, cwd, &RunOptions::default())
    }

    /// The same, with a deadline and an environment. The one required
    /// method: `run` defaults onto it, so every existing call site is
    /// unchanged and there is a single implementation to get right.
    fn run_with(
        &self,
        program: &str,
        args: &[String],
        cwd: &Path,
        opts: &RunOptions,
    ) -> Result<Output>;
}
```

An expired deadline is an `Error::Command` like any other failed spawn, which
is what lets the pin query report it as "could not reach it" without knowing
why. Only what superdev does on its own initiative is bounded: the
`ls-remote` query takes a deadline of a few seconds and
`GIT_TERMINAL_PROMPT=0`; the clone takes the environment and no deadline,
because the user asked for it and a slow link is not superdev's to give up
on. ADR-015.

### Planning context — the one component-facing change

```rust
pub struct Ctx<'a> {
    pub root: &'a Path,
    pub runner: &'a dyn CommandRunner,
    pub manifest: &'a Manifest,
    pub lock: &'a Lock,
    /// Resolved content. The resolver ran before planning, so `plan` stays
    /// side-effect free and `status` provably never fetches. ADR-002.
    pub content: &'a ContentSet,
}
```

`Component` is unchanged — `capability`, `provider`, `plan`, `owned` keep
their signatures. Components stop reading `include_str!` constants and read
`ctx.content` instead.

### Lock — `.superdev/lock.toml`

```toml
[[packs]]
source   = "github:six5536/superdev"
identity = "github.com/six5536/superdev"
rev      = "assets-v1.4.0"
digest   = "sha256:9f2a…"
format   = 1

# Its identity is relative to the repo root, so this file reads the same in
# every checkout of the repository that commits it. ADR-011.
# A directory on this machine. No `rev` — it is read afresh every run — and no
# digest: there are no pinned bytes to verify, and recording one would rewrite
# this line on every content commit. ADR-016.
[[packs]]
source   = "./pack"
identity = "pack"
format   = 1
```

```rust
/// One resolved pack, recorded so a later run can prove it got the same
/// bytes, and so a dropped entry's files become orphans by the existing
/// rule. Per-file hashes stay in the lock's existing `files` map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackLock {
    pub source: String,
    /// `PackSource::identity`. This file is committed, so a path source's key
    /// is relative to the repo root and reads the same in every checkout.
    /// ADR-011.
    pub identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// What a fetched pack was verified against. Absent for a path source:
    /// a directory is read afresh every run, so no pinned bytes exist to
    /// verify, and a recorded value would be rewritten by every content
    /// commit and read by nothing. Absent exactly when `rev` is. ADR-016.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    pub format: u32,
}
```

### Errors

```rust
pub enum Error {
    // … existing variants unchanged …
    /// A pack could not be resolved, or resolved to bytes that do not match
    /// what the lock recorded. Carries the pack as the user named it.
    Pack { pack: String, message: String },
}
```

## Module boundaries

- `superdev-core::pack` — new. Owns `PackSource`, identity, `PackManifest`,
  the cache under `.superdev/cache/packs/<digest>/`, fetching, and digest
  verification. Knows nothing about components or capabilities. A git source
  is fetched by spawning the user's `git` through the injected
  `CommandRunner` — shallow, blobless and sparse
  ([ADR-007](../decisions/D007-git-fetch-by-spawn.md)).
- `superdev-core::content` — new. Owns `ItemKind`, `Item`, `Origin`,
  `Shadowed`, `ContentSet`, and the layout rules that turn a directory tree
  into items. Depends on nothing but `std`.
- `/pack/` — the stock content at the repo root, in pack layout, with its own
  `pack.toml` ([ADR-006](../decisions/D006-pack-at-repo-root.md)). It is the
  first-party pack and the reference a third-party author copies, so it
  carries nothing that exists only to serve superdev's build.
  `crates/lib/superdev-core/assets` is a relative symlink to it, which is what
  keeps the files inside the published crate; `include_str!` paths are
  unchanged.
- The embedded snapshot is built through `content`'s same layout rules over
  those files, so snapshot and fetched pack take one code path rather than
  two. Moving and reorganising `crates/lib/superdev-core/assets` into `/pack/`
  is a slice of its own, and the first one.
- `pack` depends on `content`; `content` never depends on `pack`.
- Components depend on `content` only, through `Ctx`. No component may call
  `pack`.
- `pipeline` calls `pack::resolve` before `plan_repo` and threads the
  `ContentSet` into `Ctx`.
- `engine` is untouched: it still receives `Action`s and remains the only
  side-effect site for repo writes. Fetching is the resolver's, and happens
  before any plan exists.

## Key flows

**Default `init`, offline.** The manifest `init` builds carries the default
`[[packs]]` entry naming `DEFAULT_PACK`, so `resolve` sees a pin at exactly
the rev this binary embeds → returns the snapshot's `ContentSet` with no cache
read and no network → `plan_repo` → apply. An empty `manifest.packs` takes the
same path, and is what every manifest written before packs existed carries.

**Pinning a newer rev.** `sync` → `resolve(Fetching)` finds the pin absent
from the lock → fetches into `.superdev/cache/packs/<digest>/` → parses
`pack.toml`, refuses an unknown `format`, refuses a `REJECTED` path → the
entry's identity matches `DEFAULT_PACK`, so its items replace the snapshot's
rather than layering → `plan_repo` writes the changed items and the orphan
pass removes what the new rev dropped → apply records `PackLock` and the
per-file hashes.

**CI drift check.** `status` → `resolve(Offline)` → every pack's digest is in
the lock and its cache entry present, or its files are committed and hash to
the locked values → no fetch → `plan_repo` finds nothing → exit 0.

**Repairing drift offline.** A pack-provided skill was hand-edited. `sync` →
`resolve(Fetching)` finds the pack cached → no fetch → `plan_repo` emits the
`WriteFile` → apply backs the file up and rewrites it.

## Cross-cutting concerns

- **Security.** A source may only name https, ssh or `file`, refused at
  parse and refused again by `protocol.allow=never` with the same set
  admitted explicitly, so a transport anyone on-path can answer cannot be
  the one the base pack arrives over (ADR-012). A git source is fetched by
  the user's own `git`, so
  credentials, ssh agents and forge access are the user's and superdev holds
  no token. Every resolved pack is verified against the digest the lock
  recorded for that rev; a mismatch fails the run and writes nothing, with no
  flag to override. A pack declares no executable action — there is no
  variant it could reach `Action::Run` through. `REJECTED` paths, and every symlink,
  are refused before any file is read. Git credentials are the user's own; superdev
  stores no token and adds no auth surface. Trust in a pack's *content* is the
  user's, made by naming the source, and stated as such in the docs.
- **Performance.** One resolve per run; the cache makes a second run over an
  unchanged pin free. Packs are order-1MB, so the whole set is held in memory
  during a run.
- **Migration/rollout.** An absent `[[packs]]` array parses as empty and
  resolves from the snapshot, so every existing manifest works untouched.
  `sync` never adds the entry; `update` does. Rollback is deleting the
  `[[packs]]` entries: the orphan pass then prunes the pack's unmodified files
  and releases the edited ones, by the existing rule.
- **Observability.** `status` prints one content line per layer — the base
  first, marked as replacing the snapshot, then each layer with its item count
  — so a wrong identity match is visible on the next command. Pack-over-pack
  shadowing prints per item. A pending pin under `Offline` prints as pending
  with `sync` named as the next step.
