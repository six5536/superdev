---
type: Contract
id: contract-007-interface-pack-resolution
kind: interface
title: Interface contract for pack resolution
description: The internal interfaces that carry external content to components — pack source identity, the item model, the resolved content set, the resolution phase, the pin update proves, the process seam, and the Ctx that keeps planning pure.
lifecycle: active
resource: /crates/lib/superdev-core/src/pack
links:
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The definition is materialised from the `pack-resolution` regions and bound by the include; the boundary, flow and cross-cutting promises are bound by the pack, content and pipeline tests.
  - rel: references
    to: contract-004-config-superdev
    note: The manifest keys a repo supplies, the [[packs]] section included.
  - rel: references
    to: contract-005-format-pack
    note: What a pack must look like on disk for the resolver to read it, the refused paths included.
  - rel: references
    to: contract-006-format-lock
    note: What the resolver records of the last apply.
---

# Interface contract: pack resolution

How externally sourced content reaches components. The Definition is
the surface as the source declares it, one region name across the
files that hold it: the pack source and its identity, the item model, the
resolved content set, the resolution phase, the pin update, the process
seam, the planning context and the error resolution adds. The decisions:
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
fetch-by-spawn ([ADR-007][sokf:adr-007-git-fetch-by-spawn]), the
deadline on the process seam
([ADR-015][sokf:adr-015-the-spawn-seam-carries-a-deadline]) and the
materialised definition
([ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]).
What outside callers rely on lives in the public contracts: the manifest
keys in [contract-004][sokf:contract-004-config-superdev], the pack
format — its layout table and the paths it refuses — in
[contract-005][sokf:contract-005-format-pack], and the lock in
[contract-006][sokf:contract-006-format-lock].

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/pack/source.rs#pack-resolution -->
```rust
/// The pack this binary embeds and defaults to.
///
/// A manifest entry whose identity matches replaces the pack compiled in
/// rather than layering over it, so a rev that drops an item removes it from
/// the repo. ADR-004.
pub struct DefaultPack {
    /// The source, as `init` writes it into a fresh manifest.
    pub source: &'static str,
    /// The pack tag this binary's embedded content was cut at.
    pub rev: &'static str,
}

/// The default pack: superdev's own repository, at the content tag matching
/// the `version` in `/pack/pack.toml`. The release script sets both, and a
/// test holds them together.
pub const DEFAULT_PACK: DefaultPack = DefaultPack {
    source: "github:six5536/superdev",
    rev: "assets-v0.1.0",
};

/// The transports a pack may be fetched over.
///
/// [`PackSource::parse`] refuses anything else, naming the source, and no
/// config on the machine can lift that. `git://` and `http://` are left out
/// deliberately: neither authenticates, so anyone on the path can answer for
/// a pack — and a source keys the same however it is spelled, so the pack
/// they answer with can be the one that replaces superdev's own content.
///
/// The git overrides admit the same set explicitly over a
/// `protocol.allow=never` default. Both halves are needed and neither is
/// sufficient: `parse` cannot see a `url.<base>.insteadOf` rewrite, which
/// turns an approved `https://` source into whatever the machine's config
/// says, and among the overrides only the named refusals outrank a user
/// config. ADR-012.
pub const SUPPORTED_SCHEMES: &[&str] = &["https", "ssh", "file"];

/// What a pack release tag is called: this prefix and a three-part version.
///
/// One repository cuts two kinds of release — the binary at `vX.Y.Z` and its
/// content here — so the content tag carries a prefix that keeps the two
/// apart. `update` moves a default-source pin between these and nothing
/// else. ADR-008.
pub const PACK_TAG_PREFIX: &str = "assets-v";

/// Where a pack is resolved from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackSource {
    /// A git repository at a revision.
    Git {
        /// The URL as the manifest wrote it.
        url: String,
        /// Tag, branch or commit sha.
        rev: String,
    },
    /// A directory on this machine. A relative path is resolved against the
    /// repo root by the resolver before its identity is taken.
    Path {
        /// The directory, as the manifest wrote it until the resolver
        /// absolutises it.
        path: PathBuf,
    },
}

impl PackSource {
    /// Parse one manifest entry.
    ///
    /// Rejects a git source carrying no `rev` and a path source that names
    /// one: a pin is what makes a git source reproducible, and a directory
    /// has no revision to pin. Also refused, before anything spawns: a
    /// source or rev beginning with `-`, and a transport outside
    /// [`SUPPORTED_SCHEMES`], a `<name>::<address>` remote helper included.
    /// ADR-004, ADR-012.
    pub fn parse(entry: &PackEntry) -> Result<PackSource> {
        let source = entry.source.trim();
        // A value beginning with `-` is an option wherever git meets it, and
        // no source or rev worth having starts that way. Refused here so
        // nothing downstream has to reason about where in an argument vector
        // it lands. I007.
        for (what, value) in [
            ("source", Some(source)),
            ("rev", entry.rev.as_deref().map(str::trim)),
        ] {
            let Some(value) = value.filter(|v| v.starts_with('-')) else {
                continue;
            };
            return Err(Error::Pack {
                pack: entry.source.clone(),
                message: format!(
                    "its {what} `{value}` begins with `-`, which git reads as an \
                     option rather than as a {what}"
                ),
            });
        }
        if is_git(source) {
            // Before the `rev`, because a transport superdev will not fetch
            // over is wrong whatever revision it names.
            if let Some(refusal) = unsupported_transport(source) {
                return Err(Error::Pack {
                    pack: entry.source.clone(),
                    message: refusal,
                });
            }
            let Some(rev) = entry
                .rev
                .as_deref()
                .map(str::trim)
                .filter(|r| !r.is_empty())
            else {
                return Err(Error::Pack {
                    pack: entry.source.clone(),
                    message: "a git source needs a `rev` — name the tag, branch or commit to pin"
                        .into(),
                });
            };
            return Ok(PackSource::Git {
                url: source.to_string(),
                rev: rev.to_string(),
            });
        }
        if entry.rev.is_some() {
            return Err(Error::Pack {
                pack: entry.source.clone(),
                message: "a path source takes no `rev` — a directory has no revision to pin".into(),
            });
        }
        Ok(PackSource::Path {
            path: PathBuf::from(source),
        })
    }

    /// The comparison key every spelling of one source shares.
    ///
    /// Scheme, userinfo, port, a `.git` suffix and any trailing slash are
    /// removed and host and path lowercased, so `github:six5536/superdev`,
    /// `https://github.com/six5536/superdev.git` and the ssh form are one
    /// source.
    ///
    /// A path source's key is its canonicalised location — which
    /// [`PackSource::rooted`] settles — expressed relative to `root`, with
    /// forward slashes. The lock this is written to is committed, so an
    /// absolute path would put the author's own directory layout in a tracked
    /// file and every other checkout's first `sync` would rewrite it. That is
    /// why `root` is a parameter: a path key means nothing apart from the
    /// repository it was taken from. A git source ignores it. ADR-004,
    /// ADR-011.
    ///
    /// Two keys are only ever compared within a source kind — see
    /// [`PackSource::is_default`].
    pub fn identity(&self, root: &Path) -> String {
        match self {
            PackSource::Git { url, .. } => git_identity(url),
            PackSource::Path { path } => path_identity(path, root),
        }
    }
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/content/item.rs#pack-resolution -->
```rust
/// What materialises an item: a capability's component, superdev's own SOKF
/// component, or nothing in particular for the repo-level kinds.
///
/// Part of an item's identity because the SOKF component and the skills
/// capability both write into `.claude/skills/` and their `custom` lists are
/// name-guarded: a name in one list must never release the other's file.
/// ADR-003.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Owner {
    /// Materialised by one capability's component.
    Capability(Capability),
    /// Materialised by the SOKF component, which fills no slot.
    Knowledge,
    /// Repo-level: written outside any component's claim.
    Repo,
}

/// The kinds of content a pack may carry, each named by where it sits under
/// its owner directory. ADR-003.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemKind {
    /// `<owner>/skills/<name>/**` — owned files under `.claude/skills/`.
    Skill,
    /// `knowledge/concepts/<name>` — a write-once bundle scaffold. `<name>`
    /// is any entry directly under `concepts/`, file or directory, because
    /// the bundle ships scaffolds that are not one `.md` each. ADR-010.
    KnowledgeSkeleton,
    /// `knowledge/schemas/<name>.md` — an owned document schema.
    DocSchema,
    /// `knowledge/schemas/fragments/<name>.md` — an owned fragment, the
    /// authored home of content other documents materialize through an
    /// include block. Ships with the schema set. ADR-027.
    Fragment,
    /// `agents/<name>.md` — a write-once general-rules scaffold.
    AgentScaffold,
    /// `projects/<name>/**` — write-once repo scaffolds, token-substituted.
    ProjectTemplate,
}

/// One item and every file it owns, paths relative to the item's own root.
///
/// `(owner, kind, name)` is the identity a later layer supersedes on.
///
/// `name` is what the kind's path pattern calls `<name>`: the directory name
/// where the entry is a directory, and the file name without the `.md` the
/// pattern spells out — `agents/<name>.md` names `coding`, not `coding.md`.
/// A knowledge skeleton is the exception the pattern already states, since
/// `knowledge/concepts/<name>` admits any entry (ADR-010), so its name carries
/// whatever extension the entry has. A single-file item carries one file whose
/// relative path is empty: the item root *is* the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Which capability materialises it, or `Repo`.
    pub owner: Owner,
    /// What kind of content it is.
    pub kind: ItemKind,
    /// The entry's name in the pack tree.
    pub name: String,
    /// (path relative to the item root, content), in path order. A single-file
    /// item has exactly one entry, with an empty path.
    pub files: Vec<(String, String)>,
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/content/set.rs#pack-resolution -->
```rust
/// Which layer an item came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// The binary's embedded snapshot.
    Snapshot,
    /// A pack, by its manifest index and name.
    Pack {
        /// Position in the manifest's `[[packs]]` array.
        index: usize,
        /// The pack as the user named it.
        name: String,
    },
}

/// One item a later layer hid.
///
/// Reported only when both layers are packs: superseding the snapshot is the
/// ordinary case and passes unreported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shadowed {
    /// The hidden item's owner.
    pub owner: Owner,
    /// The hidden item's kind.
    pub kind: ItemKind,
    /// The hidden item's name.
    pub name: String,
    /// The layer whose item is in force.
    pub winner: Origin,
    /// The layer whose item it hid.
    pub loser: Origin,
}

/// Every item the layers resolved to. Built once per run, borrowed by `Ctx`.
#[derive(Debug)]
pub struct ContentSet {
    /// Keyed by identity so lookup and `items_of` are both ordered.
    items: BTreeMap<(Owner, ItemKind, String), (Item, Origin)>,
    shadowed: Vec<Shadowed>,
    base: Option<Origin>,
}

    /// One item, or `None` when no layer provides it.
    pub fn item(&self, owner: Owner, kind: ItemKind, name: &str) -> Option<&Item> {
        self.entry(owner, kind, name).map(|(item, _)| item)
    }

    /// Every item of one kind, in name order.
    pub fn items_of(&self, owner: Owner, kind: ItemKind) -> impl Iterator<Item = &Item> {
        self.items
            .range((owner, kind, String::new())..)
            .take_while(move |((o, k, _), _)| *o == owner && *k == kind)
            .map(|(_, (item, _))| item)
    }

    /// Where an item came from, for reporting.
    pub fn origin(&self, owner: Owner, kind: ItemKind, name: &str) -> Option<&Origin> {
        self.entry(owner, kind, name).map(|(_, origin)| origin)
    }

    /// Pack-over-pack shadowing, for the report.
    pub fn shadowed(&self) -> &[Shadowed] {
        &self.shadowed
    }

    /// The entry that replaced the snapshot, when one did. ADR-004.
    pub fn base(&self) -> Option<&Origin> {
        self.base.as_ref()
    }
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/pack/resolve.rs#pack-resolution -->
```rust
/// How far the resolver may go. ADR-002.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveMode {
    /// `status`: never fetch, never write the cache. A pin it cannot satisfy
    /// is returned as pending, not as an error — a checkout that has not
    /// fetched yet is not drift.
    Offline,
    /// `init`, `sync`, `update`: fetch what is missing. An unsatisfiable pin
    /// is an error.
    Fetching,
}

/// What one resolve produced.
#[derive(Debug)]
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

/// Resolve the manifest's packs over the embedded pack.
///
/// The only phase in a run that reads anything outside the repo: local
/// paths, the machine's cache, and — under `Fetching`, for bytes it does not
/// already have — the user's own `git`. An entry pinning exactly what this
/// binary embeds resolves from it and makes no request.
///
/// Every pack is verified against the digest the lock recorded for its rev.
/// A mismatch fails the run and writes nothing, rather than applying bytes
/// nobody pinned.
pub fn resolve(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    lock: &Lock,
    mode: ResolveMode,
) -> Result<Resolution> {
    let mut packs = Vec::new();
    let mut layers = vec![(snapshot_items(), Origin::Snapshot)];
    let mut base = None;
    let mut pending = Vec::new();
    // Keyed on the kind as well as the identity: a path key is relative now,
    // so it can spell a repository exactly, and a vendored tree at
    // `github.com/owner/repo` beside a pin on that repository is two sources,
    // not one named twice. ADR-011.
    let mut seen: BTreeMap<(&str, String), &str> = BTreeMap::new();
    for (index, entry) in manifest.packs.iter().enumerate() {
        // Settled before the identity below compares it: a relative path
        // is a location, and two spellings of one directory are one pack.
        let source = PackSource::parse(entry)?.rooted(root);
        // Two entries naming one source cannot both be layered, and one of
        // them naming the base would layer over it and silently win. The
        // manifest refuses a provider listed twice for the same reason.
        let kind = match source {
            PackSource::Git { .. } => "git",
            PackSource::Path { .. } => "path",
        };
        if let Some(first) = seen.insert((kind, source.identity(root)), &entry.source) {
            return Err(Error::Pack {
                pack: entry.source.clone(),
                message: format!(
                    "names the same source as `{first}` — each pack appears once; \
                     delete one entry"
                ),
            });
        }
        let origin = Origin::Pack {
            index,
            name: entry.source.clone(),
        };
        match resolve_one(root, runner, entry, &source, lock, mode)? {
            Resolved::Layer(items, record) if is_base(&source) => {
                packs.push(record);
                // The embedded pack is a convenience copy of this same pack
                // at an older rev, not a rival content set: the pinned rev
                // becomes the whole of layer 0, including what it no longer
                // carries. ADR-004.
                layers[0] = (items, origin.clone());
                base = Some(origin);
            }
            Resolved::Layer(items, record) => {
                packs.push(record);
                layers.push((items, origin));
            }
            // The pin names exactly what is compiled in, so layer 0 already
            // is this entry; naming it again would shadow it with itself.
            Resolved::Embedded => base = Some(origin),
            Resolved::Pending => pending.push(entry.clone()),
        }
    }
    Ok(Resolution {
        content: ContentSet::from_layers(layers, base),
        pending,
        packs,
    })
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/pack/pin.rs#pack-resolution -->
```rust
/// Bring the manifest's pack pins current, and say what happened.
///
/// A manifest carrying no entry gains the default one: an absent `[[packs]]`
/// means the pack compiled in, and writing it out is what makes the pin
/// visible and editable. The default source is then asked for its newest
/// release and the pin moves there, even ahead of what this binary embeds —
/// the one path by which a content fix reaches an unchanged binary. Every
/// other source is reported and left alone: moving it would pull unreviewed
/// content on a routine command, and naming a source is the user's trust
/// decision to make again, not superdev's to make for them. ADR-009.
///
/// A pin never moves backwards. Going to look is meant to bring content
/// forward, and a source that has lost a tag, or a hand-pin ahead of its
/// releases, must not quietly take content away.
///
/// Nothing here fails the run: `update` syncs immediately afterwards, and
/// that is where a pin that cannot be resolved is reported properly.
pub fn update_pins(
    runner: &dyn CommandRunner,
    root: &Path,
    manifest: &mut Manifest,
    lock: &Lock,
) -> Vec<String> {
    let mut lines = Vec::new();
    if manifest.packs.is_empty() {
        manifest.packs.push(PackEntry {
            source: DEFAULT_PACK.source.to_string(),
            rev: Some(DEFAULT_PACK.rev.to_string()),
        });
        lines.push(format!(
            "packs: wrote the default entry {} at {}",
            DEFAULT_PACK.source, DEFAULT_PACK.rev
        ));
    }
    // What this binary carries is the floor: an older pin comes up to it
    // with no request at all, because the content is already on disk. A
    // candidate binary pins a candidate tag, which is no release and so no
    // floor — its content is exactly what a stable pin must not be raised to.
    let floor = release(DEFAULT_PACK.rev);
    // Asked once however many entries spell it, and not at all when none
    // names the default source — an update that touches nothing of
    // superdev's makes no request.
    let mut asked: Option<Newest> = None;
    for entry in &mut manifest.packs {
        // A malformed entry is not this function's to report: `update` syncs
        // next, and resolution says what is wrong with it in full.
        let Ok(source) = PackSource::parse(entry) else {
            continue;
        };
        // A directory has no revision to move; it is read afresh every run.
        let PackSource::Git { rev, .. } = &source else {
            continue;
        };
        let rev = rev.clone();
        let named = entry.source.clone();
        if !source.is_default() {
            lines.push(format!(
                "packs: {named} stays at {rev} — superdev does not ship this source, \
                 so moving it is yours to do"
            ));
            continue;
        }
        // A candidate content tag is superdev's own, cut beside a binary
        // release candidate, and the repo an rc binary set up is pinned to
        // one. Nothing else ever rewrites a manifest, so leaving it alone the
        // way a branch is left alone would strand that repo on candidate
        // content for good. Its release is what it is a candidate for.
        let (current, candidate) = match release(&rev) {
            Some(current) => (current, false),
            None => match candidate_release(&rev) {
                Some(core) => (core, true),
                None => {
                    lines.push(format!(
                        "packs: {named} stays at {rev} — not a release tag, \
                         so nothing says what is newer"
                    ));
                    continue;
                }
            },
        };
        let newest = asked.get_or_insert_with(|| newest_release(runner, root, &source));
        let (target, unchecked) = choose(current, floor, newest);
        // A candidate only comes forward onto a release something vouches
        // for — this binary's own, or one the source answered with. Its own
        // core is not that: `assets-v0.2.0-rc.1` says nothing about whether
        // `assets-v0.2.0` was ever cut.
        let covered = !candidate
            || floor.is_some_and(|floor| floor >= current)
            || matches!(newest, Newest::Answered(Some(remote)) if *remote >= current);
        let moved = tag(target);
        if covered && moved != rev {
            // Proven before it is written. `update` saves the manifest before
            // the `sync` that follows validates anything, and the
            // never-backwards rule means it cannot undo what it just did — so
            // a pin that lands on a pack this binary cannot read is a state no
            // superdev command can leave. ADR-013, I001.
            if let Err(refused) = probe(runner, root, entry, &moved, lock) {
                lines.push(format!("packs: {named} stays at {rev} — {refused}"));
                continue;
            }
            entry.rev = Some(moved.clone());
            lines.push(match unchecked {
                None => format!("packs: {named} moved to {moved}"),
                Some(why) => format!(
                    "packs: {named} moved to {moved} — {why}, so no further than this binary carries"
                ),
            });
        } else {
            lines.push(match (candidate, unchecked) {
                (true, _) => format!(
                    "packs: {named} stays at {rev} — a candidate, and no release covers it yet"
                ),
                (false, None) => format!("packs: {named} is at the newest release {rev}"),
                (false, Some(why)) => format!("packs: {named} stays at {rev} — {why}"),
            });
        }
    }
    lines
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/runner.rs#pack-resolution -->
```rust
/// Captured result of a finished process.
#[derive(Debug, Clone)]
pub struct Output {
    /// Exit status (`-1` when terminated by a signal).
    pub status: i32,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
}

/// How a command is run: everything beyond the program and its arguments.
///
/// One struct rather than a method per concern, so the next thing that needs
/// the process boundary has somewhere to go. ADR-015.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Kill the child and fail after this long. `None` waits as long as it
    /// takes, which is what a toolchain install needs.
    pub timeout: Option<Duration>,
    /// Extra environment for the child, over the inherited one.
    pub env: Vec<(String, String)>,
}

/// The single seam to the outside world for process execution.
///
/// Two calling forms, one implementation: [`CommandRunner::run_with`] is the
/// required method and [`CommandRunner::run`] defaults onto it with no
/// options. A caller that needs neither a deadline nor an environment writes
/// `run` and is unaffected by either existing.
pub trait CommandRunner {
    /// Run `program args…` in `cwd`, capturing output. A missing program is
    /// [`Error::Command`] with `status: None` and stderr `"not found"`.
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<Output> {
        self.run_with(program, args, cwd, &RunOptions::default())
    }

    /// The same, with a deadline and an environment.
    ///
    /// A deadline that expires is an [`Error::Command`] like any other failed
    /// spawn, so a caller that only wants to know it did not work — the pin
    /// query reporting "could not reach it" — needs no new arm.
    fn run_with(
        &self,
        program: &str,
        args: &[String],
        cwd: &Path,
        opts: &RunOptions,
    ) -> Result<Output>;
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/component.rs#pack-resolution -->
```rust
/// Everything a component may look at while planning. Read-only.
pub struct Ctx<'a> {
    /// Target repo root.
    pub root: &'a Path,
    /// Process seam for observation commands.
    pub runner: &'a dyn CommandRunner,
    /// Desired state.
    pub manifest: &'a Manifest,
    /// Last-applied state.
    pub lock: &'a Lock,
    /// The content the layers resolved to: every skill, scaffold and template
    /// a component materialises. Resolved before planning, so `plan` stays
    /// side-effect free (ADR-002).
    pub content: &'a ContentSet,
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/error.rs#pack-resolution -->
```rust
    /// A pack could not be resolved, or resolved to bytes that do not match
    /// what the lock recorded.
    Pack {
        /// The pack as the user named it, so the message points at the
        /// manifest entry they would edit.
        pack: String,
        /// What is wrong.
        message: String,
    },
```
<!-- /sokf:include -->

## Behaviour

### Module boundaries

`superdev-core::pack` owns sources, identity, the cache, fetching and
digest verification. `superdev-core::content` owns the item model and
the layout rules, and depends on `std` alone; `pack` depends on
`content`. `/pack/` at the repo root is the first-party pack and the
third-party reference ([ADR-006][sokf:adr-006-pack-at-repo-root]); the
embedded snapshot is built from it through the same layout rules as a
fetched pack. `pipeline` calls `pack::resolve` before `plan_repo`, and
`engine` stays the only side-effect site for repo writes. A fetched
pack is judged by git's index modes, a path pack by `symlink_metadata`,
and the cache is re-checked at read time (ADR-014).

- `P_pack-knows-no-component` [ubiquitous] `superdev-core::pack` SHALL
  NOT know components or capabilities.
- `P_content-below-pack` [ubiquitous] `superdev-core::content` SHALL NOT
  depend on `pack`.
- `P_component-reads-ctx` [ubiquitous] A component SHALL read content
  only through `Ctx`.
- `P_component-calls-no-pack` [ubiquitous] A component SHALL NOT call
  `pack`.
- `P_pack-carries-no-symlink` [ubiquitous] A pack SHALL NOT carry a
  symlink or a submodule anywhere (ADR-014).
- `P_symlink-fails-resolution` [event] WHEN a pack carries a symlink or
  a submodule, `resolve` SHALL fail naming the path (ADR-014).

### Key flows

1. Default `init`, offline: the manifest pins `DEFAULT_PACK` at the
   embedded rev → `resolve` returns the snapshot with no cache read and
   no network → plan → apply.
2. Pinning a newer rev: `sync` → `resolve(Fetching)` fetches into the
   cache, parses `pack.toml`, refuses unknown formats and the paths
   [contract-005][sokf:contract-005-format-pack] names as `REJECTED` →
   items replace or layer by identity → plan writes the changes and the
   orphan pass removes what the rev dropped → apply records `PackLock`.
3. CI drift check: `status` → `resolve(Offline)` → cache and lock agree
   → no fetch, no findings, exit 0.
4. Repairing drift offline: an edited pack file → `resolve(Fetching)`
   finds the pack cached → plan emits the `WriteFile` → apply backs up
   and rewrites.
5. Moving a pin: `update` → `update_pins` asks the default source for
   its newest release → the moved pin is proven → the pin is written,
   or stays put when resolution refuses, the refusal reported in the
   line that would have announced the move. ADR-009, ADR-013.
6. Spawning: an expired deadline is an `Error::Command` like any failed
   spawn. Only unprompted work takes a deadline: the pin query runs
   with one and `GIT_TERMINAL_PROMPT=0`; a user-requested clone takes
   the environment and no deadline. ADR-015.

- `P_moved-pin-proven` [event] WHEN `update_pins` moves a pin, the moved
  pin SHALL resolve before `update_pins` writes it (ADR-013).

### Cross-cutting concerns

Security: fetching spawns the user's own `git`; trust in a pack's
content is the user's, made by naming the source.

- `P_transport-allowlisted` [ubiquitous] `resolve` SHALL admit a
  transport only from `SUPPORTED_SCHEMES`, checked at parse and again
  by git's protocol policy (ADR-012).
- `P_no-token-stored` [ubiquitous] The fetch SHALL NOT store a token.
- `P_pack-verified-against-lock` [ubiquitous] `resolve` SHALL verify
  every pack against the lock's digest, with no override flag.
- `P_pack-declares-no-action` [ubiquitous] A pack SHALL NOT declare an
  executable action.
- `P_refused-paths-before-read` [ubiquitous] `resolve` SHALL refuse
  `REJECTED` paths and symlinks before it reads any file.

Performance: one resolve per run; an unchanged pin costs no fetch; the
whole content set is held in memory.

Migration/rollout: an absent `[[packs]]` array resolves from the
snapshot, so every pre-pack manifest works untouched; `update` adds the
`[[packs]]` entry. Rollback is deleting the entries, and the orphan
pass prunes.

- `P_sync-adds-no-entry` [ubiquitous] `sync` SHALL NOT add the
  `[[packs]]` entry.

Observability: `status` prints one content line per layer, the base
marked; pack-over-pack shadowing prints per item; a pending pin names
`sync` as the next step.

## Stability

Internal.

- `P_internal` [ubiquitous] Every item above MAY change with the crate.

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
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:contract-004-config-superdev]: /knowledge/contracts/public/active/contract-004-config-superdev.md
[sokf:contract-005-format-pack]: /knowledge/contracts/public/active/contract-005-format-pack.md
[sokf:contract-006-format-lock]: /knowledge/contracts/public/active/contract-006-format-lock.md
