---
type: InterfaceContract
id: contract-007-interface-pack-resolution
title: Pack Resolution Interface Contract
description: The internal interfaces that carry external content to components — pack source identity, the item model, the resolved content set, the resolution phase, the pin update proves, the process seam, and the Ctx that keeps planning pure.
lifecycle: active
links:
  - rel: references
    to: contract-004-config-superdev
    note: The manifest keys a repo supplies, the [[packs]] section included.
  - rel: references
    to: contract-005-file-format-pack
    note: What a pack must look like on disk for the resolver to read it.
  - rel: references
    to: contract-006-file-format-lock
    note: What the resolver records of the last apply.
---

# Interface contract: pack resolution

How externally sourced content reaches components. The decisions:
identity and layering
([ADR-004][sokf:adr-004-base-pack-identity],
[ADR-011][sokf:adr-011-path-pack-identity-is-root-relative]), the item
model ([ADR-003][sokf:adr-003-items-by-layout],
[ADR-010][sokf:adr-010-concepts-entry-is-the-item]), resolve before
planning ([ADR-002][sokf:adr-002-resolve-before-plan]), the cache
([ADR-005][sokf:adr-005-pack-cache-and-fetch]), the transport allowlist
([ADR-012][sokf:adr-012-pack-source-schemes-are-allowlisted]), the
proven pin
([ADR-013][sokf:adr-013-update-proves-a-pin-before-it-writes-it]), the
symlink refusal
([ADR-014][sokf:adr-014-a-symlink-in-a-pack-is-refused]), the
fetch-by-spawn ([ADR-007][sokf:adr-007-git-fetch-by-spawn]), and the
deadline on the process seam
([ADR-015][sokf:adr-015-the-spawn-seam-carries-a-deadline]). What
outside callers rely on lives in the public contracts: the manifest
keys in [contract-004][sokf:contract-004-config-superdev], the pack
format in [contract-005][sokf:contract-005-file-format-pack], and the
lock in [contract-006][sokf:contract-006-file-format-lock].

## Data model & API

### Pack source and identity

```rust
/// Where a pack is resolved from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackSource {
    /// A git repository at a revision.
    Git { url: String, rev: String },
    /// A directory; a relative path resolves against the repo root.
    Path { path: PathBuf },
}

impl PackSource {
    /// Parse one manifest entry. MUST reject: a git source with no
    /// `rev`; a path source with one; a source or rev starting `-`;
    /// a transport outside [`SUPPORTED_SCHEMES`], a `<name>::<address>`
    /// remote helper included, before anything spawns. ADR-004, ADR-012.
    pub fn parse(entry: &PackEntry) -> Result<PackSource>;

    /// The comparison key. A git source: scheme, userinfo, `.git` and
    /// trailing slash removed, host and path lowercased. A path source:
    /// the canonicalised path relative to `root`, forward slashes.
    /// Keys MUST NOT compare across source kinds. ADR-004, ADR-011.
    pub fn identity(&self, root: &Path) -> String;
}

/// The pack this binary embeds and defaults to; a manifest entry whose
/// identity matches replaces the snapshot instead of layering over it.
pub struct DefaultPack {
    pub source: &'static str,
    pub rev: &'static str,
}
pub const DEFAULT_PACK: DefaultPack = DefaultPack { /* … */ };

/// The transports a pack may be fetched over; `parse` refuses the
/// rest, and the git overrides enforce the same set on the spawned
/// side. ADR-012.
pub const SUPPORTED_SCHEMES: &[&str] = &["https", "ssh", "file"];
```

### Items — what a pack provides

A pack's tree names all three parts of an item's identity — the owning
capability, the kind, and the name. `<name>` is the entry directly under
the kind directory; `/**` marks a directory item, superseded whole:

```
pack.toml
knowledge/skills/<name>/**             → .claude/skills/<name>/**              owned
knowledge/concepts/<name>              → knowledge/<name>                      scaffold
knowledge/schemas/<name>.md            → knowledge/schemas/<name>.md           owned
knowledge/schemas/fragments/<name>.md  → knowledge/schemas/fragments/<name>.md owned
skills/<name>/**                       → .claude/skills/<name>/**              owned
agents/<name>.md                       → .agents/<name>.md                     scaffold
projects/<name>/**                     → repo root, tokenised                  scaffold
```

```rust
/// Which capability materialises an item, or none for the repo-level
/// kinds. Part of identity: two capabilities write into
/// `.claude/skills/`. ADR-003.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Owner {
    Capability(Capability),
    Repo,
}

/// The kinds of content a pack may carry, named by layout. ADR-003.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemKind {
    /// `<owner>/skills/<name>/**` — owned files under `.claude/skills/`.
    Skill,
    /// `knowledge/concepts/<name>` — a write-once knowledge scaffold;
    /// `<name>` is a file or a directory. ADR-010.
    KnowledgeSkeleton,
    /// `knowledge/schemas/<name>.md` — an owned document schema.
    DocSchema,
    /// `knowledge/schemas/fragments/<name>.md` — an owned fragment,
    /// shipped with the schema set. ADR-027.
    Fragment,
    /// `agents/<name>.md` — a write-once general-rules scaffold.
    AgentScaffold,
    /// `projects/<name>/**` — write-once repo scaffolds, tokenised.
    ProjectTemplate,
}

/// One item and every file it owns, paths relative to the item's own
/// root. `(owner, kind, name)` is the identity a later layer
/// supersedes on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub owner: Owner,
    pub kind: ItemKind,
    pub name: String,
    pub files: Vec<(String, String)>,
}

/// Paths a pack MUST NOT carry, refused at resolve naming the reason:
/// these move with the binary that pins or validates them.
pub const REJECTED: &[&str] = &[
    "agents/sokf.md",
    "agents/codegraph.md",
];

/// Refused wherever it appears: the project's own extension layer.
pub const REJECTED_BASENAME: &str = "PROJECT.md";
```

A pack MUST NOT carry a symlink or a submodule anywhere; resolution
MUST fail naming the path. A fetched pack is judged by git's index
modes, a path pack by `symlink_metadata`, and the cache is re-checked
at read time. ADR-014.

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

/// One item a later layer hid. Reported only when both layers are
/// packs: superseding the snapshot is the layering rule at work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shadowed {
    pub owner: Owner,
    pub kind: ItemKind,
    pub name: String,
    pub winner: Origin,
    pub loser: Origin,
}

/// Every item the layers resolved to. Built once per run, borrowed by
/// `Ctx`.
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
    /// `status`: MUST NOT fetch or write the cache; an unsatisfiable
    /// pin returns as pending.
    Offline,
    /// `init`, `sync`, `update`: fetch what is missing and populate
    /// the cache; an unsatisfiable pin is an error.
    Fetching,
}

/// What one resolve produced.
pub struct Resolution {
    /// Content from every layer that resolved.
    pub content: ContentSet,
    /// Pins `Offline` could not satisfy; always empty under `Fetching`.
    pub pending: Vec<PackEntry>,
    /// One record per resolved pack, in manifest order, for the lock.
    pub packs: Vec<PackLock>,
}

/// Resolve the manifest's packs over the embedded snapshot.
///
/// The only phase in a run that reads the cache, the network (under
/// `Fetching`) or the local pack paths. Every resolved pack MUST match
/// the digest the lock recorded for its rev; a mismatch fails the run
/// and writes nothing. A git source is fetched by spawning the user's
/// own `git` through `runner`. ADR-005, ADR-007.
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
/// Bring the manifest's pack pins current, and say what happened.
///
/// A moved pin MUST resolve before it is written, and stays put when
/// resolution refuses — the refusal is reported in the line that would
/// have announced the move. Nothing here fails the run. ADR-009,
/// ADR-013, I001.
pub fn update_pins(
    runner: &dyn CommandRunner,
    root: &Path,
    manifest: &mut Manifest,
    lock: &Lock,
) -> Vec<String>;
```

### The process seam

```rust
/// How a command is run: everything beyond the program and its
/// arguments.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Kill the child and fail after this long; `None` waits.
    pub timeout: Option<Duration>,
    /// Extra environment for the child, over the inherited one.
    pub env: Vec<(String, String)>,
}

pub trait CommandRunner {
    /// Run `program args…` in `cwd`, capturing output.
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<Output> {
        self.run_with(program, args, cwd, &RunOptions::default())
    }

    /// The same, with a deadline and an environment: the one required
    /// method, so there is a single implementation to get right.
    fn run_with(
        &self,
        program: &str,
        args: &[String],
        cwd: &Path,
        opts: &RunOptions,
    ) -> Result<Output>;
}
```

An expired deadline is an `Error::Command` like any failed spawn. Only
unprompted work takes a deadline: the pin query runs with one and
`GIT_TERMINAL_PROMPT=0`; a user-requested clone takes the environment
and no deadline. ADR-015.

### Planning context — the one component-facing change

```rust
pub struct Ctx<'a> {
    pub root: &'a Path,
    pub runner: &'a dyn CommandRunner,
    pub manifest: &'a Manifest,
    pub lock: &'a Lock,
    /// Resolved content; the resolver ran before planning. ADR-002.
    pub content: &'a ContentSet,
}
```

### Errors

```rust
pub enum Error {
    // … existing variants unchanged …
    /// A pack could not be resolved, or resolved to bytes that do not
    /// match the lock. Carries the pack as the user named it.
    Pack { pack: String, message: String },
}
```

## Module boundaries

- `superdev-core::pack` owns sources, identity, the cache, fetching and
  digest verification; it MUST NOT know components or capabilities.
- `superdev-core::content` owns the item model and the layout rules; it
  depends on `std` alone.
- `/pack/` at the repo root is the first-party pack and the third-party
  reference ([ADR-006][sokf:adr-006-pack-at-repo-root]); the embedded
  snapshot is built from it through the same layout rules as a fetched
  pack.
- `pack` depends on `content`; `content` MUST NOT depend on `pack`.
- A component MUST read content only through `Ctx`; it MUST NOT call
  `pack`.
- `pipeline` calls `pack::resolve` before `plan_repo`; `engine` stays
  the only side-effect site for repo writes.

## Key flows

- Default `init`, offline: the manifest pins `DEFAULT_PACK` at the
  embedded rev → `resolve` returns the snapshot with no cache read and
  no network → plan → apply.
- Pinning a newer rev: `sync` → `resolve(Fetching)` fetches into the
  cache, parses `pack.toml`, refuses unknown formats and `REJECTED`
  paths → items replace or layer by identity → plan writes the changes
  and the orphan pass removes what the rev dropped → apply records
  `PackLock`.
- CI drift check: `status` → `resolve(Offline)` → cache and lock agree
  → no fetch, no findings, exit 0.
- Repairing drift offline: an edited pack file → `resolve(Fetching)`
  finds the pack cached → plan emits the `WriteFile` → apply backs up
  and rewrites.

## Cross-cutting concerns

- Security: a transport MUST be allowlisted, at parse and again by
  git's protocol policy (ADR-012). Fetching spawns the user's own
  `git`; superdev MUST NOT store a token. Every pack MUST verify
  against the lock's digest, with no override flag. A pack MUST NOT
  declare an executable action. `REJECTED` paths and symlinks MUST be
  refused before any file is read. Trust in a pack's content is the
  user's, made by naming the source.
- Performance: one resolve per run; an unchanged pin costs no fetch;
  the whole content set is held in memory.
- Migration/rollout: an absent `[[packs]]` array resolves from the
  snapshot, so every pre-pack manifest works untouched. `sync` MUST
  NOT add the `[[packs]]` entry; `update` does. Rollback is deleting
  the entries, and the orphan pass prunes.
- Observability: `status` prints one content line per layer, the base
  marked; pack-over-pack shadowing prints per item; a pending pin names
  `sync` as the next step.

<!-- sokf:links -->
[sokf:adr-002-resolve-before-plan]: /knowledge/adrs/active/adr-002-resolve-before-plan.md
[sokf:adr-003-items-by-layout]: /knowledge/adrs/active/adr-003-items-by-layout.md
[sokf:adr-004-base-pack-identity]: /knowledge/adrs/active/adr-004-base-pack-identity.md
[sokf:adr-005-pack-cache-and-fetch]: /knowledge/adrs/active/adr-005-pack-cache-and-fetch.md
[sokf:adr-006-pack-at-repo-root]: /knowledge/adrs/active/adr-006-pack-at-repo-root.md
[sokf:adr-007-git-fetch-by-spawn]: /knowledge/adrs/active/adr-007-git-fetch-by-spawn.md
[sokf:adr-010-concepts-entry-is-the-item]: /knowledge/adrs/active/adr-010-concepts-entry-is-the-item.md
[sokf:adr-011-path-pack-identity-is-root-relative]: /knowledge/adrs/active/adr-011-path-pack-identity-is-root-relative.md
[sokf:adr-012-pack-source-schemes-are-allowlisted]: /knowledge/adrs/active/adr-012-pack-source-schemes-are-allowlisted.md
[sokf:adr-013-update-proves-a-pin-before-it-writes-it]: /knowledge/adrs/active/adr-013-update-proves-a-pin-before-it-writes-it.md
[sokf:adr-014-a-symlink-in-a-pack-is-refused]: /knowledge/adrs/active/adr-014-a-symlink-in-a-pack-is-refused.md
[sokf:adr-015-the-spawn-seam-carries-a-deadline]: /knowledge/adrs/active/adr-015-the-spawn-seam-carries-a-deadline.md
[sokf:contract-004-config-superdev]: /knowledge/contracts/public/active/contract-004-config-superdev.md
[sokf:contract-005-file-format-pack]: /knowledge/contracts/public/active/contract-005-file-format-pack.md
[sokf:contract-006-file-format-lock]: /knowledge/contracts/public/active/contract-006-file-format-lock.md
