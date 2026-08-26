//! pack/resolve.rs — the phase that turns pack entries into content.
//!
//! Runs before anything plans, so `Component::plan` stays side-effect free
//! and `status` provably never fetches (ADR-002).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::content::{ContentSet, Item, Origin, items_from, snapshot_items};
use crate::error::{Error, Result};
use crate::lock::{Lock, PackLock};
use crate::manifest::{Manifest, PackEntry};
use crate::runner::CommandRunner;

use super::fetch;
use super::manifest::{PACK_MANIFEST, PackManifest, SUPPORTED_FORMATS, check_path};
use super::source::{DEFAULT_PACK, PackSource};

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

/// What one entry resolved to.
enum Resolved {
    /// Its own items, as a layer above the embedded pack, with the record
    /// the lock keeps of the bytes it was.
    Layer(Vec<Item>, PackLock),
    /// The embedded pack itself — the pin names exactly what is compiled in,
    /// so there is nothing to fetch.
    Embedded,
    /// Not satisfiable without reaching out, which `Offline` may not do.
    Pending,
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
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (index, entry) in manifest.packs.iter().enumerate() {
        // Settled before the identity below compares it: a relative path
        // is a location, and two spellings of one directory are one pack.
        let source = PackSource::parse(entry)?.rooted(root);
        // Two entries naming one source cannot both be layered, and one of
        // them naming the base would layer over it and silently win. The
        // manifest refuses a provider listed twice for the same reason.
        if let Some(first) = seen.insert(source.identity(), &entry.source) {
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

/// Whether an entry names the source the embedded pack is a copy of.
///
/// Compared on the normalised identity, so every spelling of that repository
/// is the base: comparing the strings would treat three of the four common
/// forms as a stranger's pack, and removals would stop propagating with
/// nothing on screen to say why. ADR-004.
fn is_base(source: &PackSource) -> bool {
    let default = PackSource::Git {
        url: DEFAULT_PACK.source.to_string(),
        rev: DEFAULT_PACK.rev.to_string(),
    };
    source.identity() == default.identity()
}

/// Resolve one entry.
fn resolve_one(
    root: &Path,
    runner: &dyn CommandRunner,
    entry: &PackEntry,
    source: &PackSource,
    lock: &Lock,
    mode: ResolveMode,
) -> Result<Resolved> {
    match source {
        PackSource::Path { path } => {
            // A directory has no rev to pin, so it is read every run and its
            // digest simply records what was read — the point of a path
            // source being that editing it lands without a re-pin.
            let (items, files) = read_pack(&entry.source, path)?;
            Ok(Resolved::Layer(
                items,
                record(entry, source, &fetch::digest(&files)),
            ))
        }
        PackSource::Git { rev, .. } => {
            // A pin naming exactly what this binary carries is the default
            // path written out, and must cost no request.
            if is_base(source) && rev == DEFAULT_PACK.rev {
                return Ok(Resolved::Embedded);
            }
            let locked = lock.packs.iter().find(|p| {
                p.identity == source.identity() && p.rev.as_deref() == Some(rev.as_str())
            });

            // Cached from an earlier resolve of this same rev: the bytes are
            // already here and already proven, so neither mode reaches out.
            if let Some(locked) = locked {
                let cached = fetch::cache_path(root, &locked.digest);
                if cached.is_dir() {
                    let (items, files) = read_pack(&entry.source, &cached)?;
                    return verified(entry, source, items, &files, Some(&locked.digest));
                }
            }
            if mode == ResolveMode::Offline {
                return Ok(Resolved::Pending);
            }

            let staging = root.join(".superdev/cache/packs/.fetch");
            // Only what verified is kept, and a failure leaves nothing at
            // all: bytes that did not match the pin have no business
            // surviving the run that rejected them.
            // `clone_url`, not `url`: git does not know superdev's
            // `github:owner/repo` shorthand and reads it as an ssh host.
            let outcome = fetch_verified(runner, root, entry, source, rev, &staging, locked);
            let _ = fs::remove_dir_all(&staging);
            outcome
        }
    }
}

/// Fetch, verify, and put what verified where a later run will find it.
///
/// Split out so the caller can clear the staging directory on every path:
/// what fails verification must not outlive the run that rejected it.
fn fetch_verified(
    runner: &dyn CommandRunner,
    root: &Path,
    entry: &PackEntry,
    source: &PackSource,
    rev: &str,
    staging: &Path,
    locked: Option<&PackLock>,
) -> Result<Resolved> {
    let pack_root = fetch::fetch(runner, &entry.source, &source.clone_url(), rev, staging)?;
    let (items, files) = read_pack(&entry.source, &pack_root)?;
    let resolved = verified(
        entry,
        source,
        items,
        &files,
        locked.map(|l| l.digest.as_str()),
    )?;
    let digest = fetch::digest(&files);
    move_into_cache(&entry.source, &pack_root, &fetch::cache_path(root, &digest))?;
    Ok(resolved)
}

/// Check a resolved pack against what the lock recorded for its rev.
///
/// The bytes a rev produced once are the bytes it must produce again: a tag
/// that moved is the case this exists for, and substituting the new content
/// silently would apply bytes nobody pinned. There is no flag to accept it —
/// the user re-pins, which is itself the new trust decision.
fn verified(
    entry: &PackEntry,
    source: &PackSource,
    items: Vec<Item>,
    files: &[(String, String)],
    locked: Option<&str>,
) -> Result<Resolved> {
    let digest = fetch::digest(files);
    if let Some(locked) = locked
        && locked != digest
    {
        return Err(Error::Pack {
            pack: entry.source.clone(),
            message: format!(
                "resolved to different bytes than the lock recorded — expected {locked}, \
                 got {digest}. The rev moved; re-pin it in {} to accept the new content",
                crate::manifest::CONFIG_PATH
            ),
        });
    }
    Ok(Resolved::Layer(items, record(entry, source, &digest)))
}

/// The lock record for one resolved pack.
fn record(entry: &PackEntry, source: &PackSource, digest: &str) -> PackLock {
    PackLock {
        source: entry.source.clone(),
        identity: source.identity(),
        rev: entry.rev.clone(),
        digest: digest.to_string(),
        format: SUPPORTED_FORMATS[0],
    }
}

/// Put a verified pack where a later run will find it without the network.
fn move_into_cache(pack: &str, from: &Path, to: &Path) -> Result<()> {
    if to.is_dir() {
        return Ok(());
    }
    let parent = to.parent().expect("cache path has a parent");
    fs::create_dir_all(parent).map_err(|e| Error::Pack {
        pack: pack.to_string(),
        message: format!("{}: {e}", parent.display()),
    })?;
    // A rename keeps it atomic where the two sit on one filesystem, which
    // they do: both are under the repo's own cache directory.
    fs::rename(from, to).map_err(|e| Error::Pack {
        pack: pack.to_string(),
        message: format!("{}: {e}", to.display()),
    })
}

/// Every item one pack directory provides.
///
/// `pack.toml` is read first: the format decides whether the rest means what
/// this binary thinks. Every path is checked before any of them becomes an
/// item, so a pack carrying a refused file contributes nothing rather than
/// contributing most of itself.
#[allow(clippy::type_complexity)]
fn read_pack(pack: &str, dir: &Path) -> Result<(Vec<Item>, Vec<(String, String)>)> {
    let manifest_path = dir.join(PACK_MANIFEST);
    let declared = fs::read_to_string(&manifest_path).map_err(|e| Error::Pack {
        pack: pack.to_string(),
        message: format!("{}: {e}", manifest_path.display()),
    })?;
    PackManifest::parse(pack, &declared)?;

    let mut files = Vec::new();
    read_dir(pack, dir, dir, &mut files)?;
    for (path, _) in &files {
        check_path(pack, path)?;
    }
    let items = items_from(
        files
            .iter()
            .map(|(path, body)| (path.as_str(), body.as_str())),
    );
    Ok((items, files))
}

/// Collect every file under `dir` as (path relative to `root`, content).
///
/// Paths are forward-slashed so a pack means the same thing on every
/// platform. `.git` is skipped: a path source is often a working checkout,
/// and its history is not content. A symlinked directory is skipped for the
/// same reason and one more — a link back to an ancestor would otherwise be
/// walked until the OS refused, and report a path forty `loop/` deep instead
/// of what was wrong.
fn read_dir(pack: &str, root: &Path, dir: &Path, files: &mut Vec<(String, String)>) -> Result<()> {
    let entries = fs::read_dir(dir).map_err(|e| Error::Pack {
        pack: pack.to_string(),
        message: format!("{}: {e}", dir.display()),
    })?;
    for entry in entries {
        let path = entry
            .map_err(|e| Error::Pack {
                pack: pack.to_string(),
                message: format!("{}: {e}", dir.display()),
            })?
            .path();
        if path.file_name().is_some_and(|name| name == ".git") {
            continue;
        }
        // `symlink_metadata` does not follow the link, which is what tells a
        // linked directory from a real one.
        let linked = fs::symlink_metadata(&path)
            .map(|meta| meta.file_type().is_symlink())
            .unwrap_or(false);
        if linked && path.is_dir() {
            continue;
        }
        if path.is_dir() {
            read_dir(pack, root, &path, files)?;
            continue;
        }
        let body = fs::read_to_string(&path).map_err(|e| Error::Pack {
            pack: pack.to_string(),
            message: format!("{}: {e}", path.display()),
        })?;
        files.push((relative(root, &path), body));
    }
    Ok(())
}

/// One file's path under the pack root, forward-slashed.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;
    use crate::content::{ItemKind, Owner};
    use crate::runner::FakeRunner;

    /// A pack directory holding one skill.
    fn write_pack(dir: &Path, skill: &str, body: &str) {
        fs::create_dir_all(dir.join(format!("knowledge/skills/{skill}"))).unwrap();
        fs::write(
            dir.join(PACK_MANIFEST),
            "format = 1\nname = \"acme\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        fs::write(dir.join(format!("knowledge/skills/{skill}/SKILL.md")), body).unwrap();
    }

    fn manifest_with(source: &str, rev: Option<&str>) -> Manifest {
        let mut manifest = Manifest::default_for("0.2.0", &[]);
        manifest.packs = vec![PackEntry {
            source: source.into(),
            rev: rev.map(Into::into),
        }];
        manifest
    }

    fn knowledge() -> Owner {
        Owner::Capability(Capability::Knowledge)
    }

    #[test]
    fn a_local_pack_contributes_its_items() {
        let repo = tempfile::tempdir().unwrap();
        write_pack(
            &repo.path().join("packs/acme"),
            "brand-new",
            "# Brand new\n",
        );
        let resolved = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap();
        let item = resolved
            .content
            .item(knowledge(), ItemKind::Skill, "brand-new")
            .expect("the pack's skill");
        assert_eq!(
            item.files,
            [("SKILL.md".to_string(), "# Brand new\n".to_string())]
        );
        assert!(resolved.pending.is_empty());
    }

    /// Precedence: what the pack provides wins over the embedded copy.
    #[test]
    fn a_pack_item_wins_over_the_embedded_one() {
        let repo = tempfile::tempdir().unwrap();
        write_pack(&repo.path().join("packs/acme"), "frame", "# Ours\n");
        let resolved = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap();
        let item = resolved
            .content
            .item(knowledge(), ItemKind::Skill, "frame")
            .expect("frame");
        assert_eq!(item.files[0].1, "# Ours\n");
        // Everything the pack does not carry still comes from the embedded
        // copy.
        assert!(
            resolved
                .content
                .item(knowledge(), ItemKind::Skill, "verify")
                .is_some()
        );
    }

    /// Case 2: a pin naming exactly what is compiled in resolves from it.
    #[test]
    fn a_pin_at_the_embedded_rev_resolves_without_reaching_out() {
        let repo = tempfile::tempdir().unwrap();
        for mode in [ResolveMode::Offline, ResolveMode::Fetching] {
            let resolved = resolve(
                repo.path(),
                &FakeRunner::new(),
                &manifest_with(DEFAULT_PACK.source, Some(DEFAULT_PACK.rev)),
                &Lock::default(),
                mode,
            )
            .unwrap();
            assert!(resolved.pending.is_empty(), "{mode:?}");
            assert!(
                resolved
                    .content
                    .item(knowledge(), ItemKind::Skill, "frame")
                    .is_some(),
                "{mode:?}: the embedded pack still supplies the items"
            );
            // Layer 0 is the embedded pack, and naming it again does not
            // make it a pack layer.
            assert_eq!(
                resolved
                    .content
                    .origin(knowledge(), ItemKind::Skill, "frame"),
                Some(&Origin::Snapshot),
                "{mode:?}"
            );
        }
    }

    /// Any spelling of the default source at the embedded rev is the same
    /// pin, so none of them reaches out either.
    #[test]
    fn any_spelling_of_the_embedded_pin_resolves_the_same_way() {
        let repo = tempfile::tempdir().unwrap();
        for source in [
            "https://github.com/six5536/superdev.git",
            "git@github.com:six5536/superdev.git",
        ] {
            let resolved = resolve(
                repo.path(),
                &FakeRunner::new(),
                &manifest_with(source, Some(DEFAULT_PACK.rev)),
                &Lock::default(),
                ResolveMode::Fetching,
            )
            .unwrap();
            assert!(resolved.pending.is_empty(), "{source}");
        }
    }

    /// Two packs, each providing the same skill name.
    fn two_packs(repo: &Path) {
        for (name, body) in [("a", "# from a\n"), ("b", "# from b\n")] {
            let dir = repo.join(format!("packs/{name}"));
            write_pack(&dir, "shared", body);
        }
    }

    fn manifest_with_packs(sources: &[&str]) -> Manifest {
        let mut manifest = Manifest::default_for("0.2.0", &[]);
        manifest.packs = sources
            .iter()
            .map(|source| PackEntry {
                source: (*source).into(),
                rev: None,
            })
            .collect();
        manifest
    }

    fn resolved(repo: &Path, sources: &[&str]) -> Resolution {
        resolve(
            repo,
            &FakeRunner::new(),
            &manifest_with_packs(sources),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect("resolves")
    }

    /// Case 5: superseding layer 0 is what a pack is for, so it passes
    /// unreported — a report on every stock item a pack replaces would be
    /// noise the user asked for.
    #[test]
    fn a_pack_superseding_the_embedded_pack_is_not_reported() {
        let repo = tempfile::tempdir().unwrap();
        write_pack(&repo.path().join("packs/acme"), "frame", "# Ours\n");
        let resolved = resolved(repo.path(), &["./packs/acme"]);
        assert_eq!(
            resolved
                .content
                .item(knowledge(), ItemKind::Skill, "frame")
                .expect("frame")
                .files[0]
                .1,
            "# Ours\n"
        );
        assert!(
            resolved.content.shadowed().is_empty(),
            "{:?}",
            resolved.content.shadowed()
        );
    }

    /// Case 6: one pack hiding another's item is a collision the user did
    /// not ask for and cannot otherwise see.
    #[test]
    fn one_pack_superseding_another_is_reported_with_both_names() {
        let repo = tempfile::tempdir().unwrap();
        two_packs(repo.path());
        let resolved = resolved(repo.path(), &["./packs/a", "./packs/b"]);
        let shadowed = resolved.content.shadowed();
        assert_eq!(shadowed.len(), 1, "{shadowed:?}");
        assert_eq!(shadowed[0].name, "shared");
        assert_eq!(shadowed[0].kind, ItemKind::Skill);
        assert_eq!(
            shadowed[0].winner,
            Origin::Pack {
                index: 1,
                name: "./packs/b".into()
            }
        );
        assert_eq!(
            shadowed[0].loser,
            Origin::Pack {
                index: 0,
                name: "./packs/a".into()
            }
        );
    }

    /// Case 7: manifest order is the only tiebreak, and reversing it changes
    /// the winner and nothing else.
    #[test]
    fn reversing_manifest_order_flips_the_winner_and_nothing_else() {
        let repo = tempfile::tempdir().unwrap();
        two_packs(repo.path());
        let forward = resolved(repo.path(), &["./packs/a", "./packs/b"]);
        let reverse = resolved(repo.path(), &["./packs/b", "./packs/a"]);

        let body = |r: &Resolution| {
            r.content
                .item(knowledge(), ItemKind::Skill, "shared")
                .expect("shared")
                .files[0]
                .1
                .clone()
        };
        assert_eq!(body(&forward), "# from b\n");
        assert_eq!(body(&reverse), "# from a\n");
        assert_eq!(forward.content.shadowed().len(), 1);
        assert_eq!(reverse.content.shadowed().len(), 1);

        // Everything neither pack carries is unchanged by the ordering.
        let names = |r: &Resolution| {
            r.content
                .items_of(knowledge(), ItemKind::Skill)
                .map(|i| i.name.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&forward), names(&reverse));
    }

    /// ADR-004: the embedded pack is a copy of the default source at an
    /// older rev, so an entry naming that source is layer 0 rather than a
    /// layer over it — and what its rev drops is simply gone.
    #[test]
    fn the_base_replaces_layer_zero_rather_than_layering_over_it() {
        let repo = tempfile::tempdir().unwrap();
        let at_default = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with(DEFAULT_PACK.source, Some(DEFAULT_PACK.rev)),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap();
        assert_eq!(
            at_default.content.base(),
            Some(&Origin::Pack {
                index: 0,
                name: DEFAULT_PACK.source.into()
            }),
            "status must be able to name which entry it treated as the base"
        );
        // A pack from any other source is a layer, not the base.
        write_pack(&repo.path().join("packs/acme"), "brand-new", "# New\n");
        let layered = resolved(repo.path(), &["./packs/acme"]);
        assert_eq!(layered.content.base(), None);
    }

    #[test]
    fn an_unresolvable_git_pin_is_pending_offline_and_an_error_when_fetching() {
        let repo = tempfile::tempdir().unwrap();
        let manifest = manifest_with("github:someone/other", Some("v9"));
        let offline = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest,
            &Lock::default(),
            ResolveMode::Offline,
        )
        .unwrap();
        assert_eq!(offline.pending.len(), 1);
        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest,
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap_err();
        assert!(err.to_string().contains("github:someone/other"), "{err}");
    }

    /// A path source is often a working checkout. A directory symlink back
    /// to an ancestor would be walked until the OS refused, and the pack
    /// would fail naming a path forty `loop/` deep rather than anything a
    /// reader could act on.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_directory_inside_a_pack_is_skipped() {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path().join("packs/acme");
        write_pack(&dir, "brand-new", "# Brand new\n");
        std::os::unix::fs::symlink(&dir, dir.join("loop")).unwrap();
        let resolved = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect("the cycle is skipped, not walked");
        assert!(
            resolved
                .content
                .item(knowledge(), ItemKind::Skill, "brand-new")
                .is_some(),
            "the real files still resolve"
        );
    }

    /// Two entries naming one repository cannot both be layered, and one of
    /// them naming the base would layer over it and silently win — the base
    /// beaten by a duplicate of itself.
    /// A real git repository holding a pack, so the fetch path runs against
    /// git rather than a script of what git might say.
    fn git_fixture(dir: &Path, tag: &str, skill: &str, body: &str) -> String {
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(dir)
                .output()
                .expect("git runs");
            assert!(out.status.success(), "git {args:?}: {out:?}");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        fs::create_dir_all(dir.join(format!("pack/knowledge/skills/{skill}"))).unwrap();
        fs::write(
            dir.join("pack/pack.toml"),
            "format = 1\nname = \"fixture\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        fs::write(
            dir.join(format!("pack/knowledge/skills/{skill}/SKILL.md")),
            body,
        )
        .unwrap();
        if !dir.join(".git").exists() {
            git(&["init", "-q", "-b", "main"]);
            git(&["config", "user.email", "fixture@example.com"]);
            git(&["config", "user.name", "fixture"]);
            // The developer's global config may sign commits and tags; a
            // fixture must not depend on a key being available.
            git(&["config", "commit.gpgsign", "false"]);
            git(&["config", "tag.gpgSign", "false"]);
        }
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", tag]);
        git(&["tag", "-f", tag]);
        git(&["rev-parse", "HEAD"])
    }

    fn git_manifest(url: &str, rev: &str) -> Manifest {
        let mut manifest = Manifest::default_for("0.2.0", &[]);
        manifest.packs = vec![PackEntry {
            source: url.into(),
            rev: Some(rev.into()),
        }];
        manifest
    }

    /// Case 3: the first resolve of a rev fetches it, and the lock records
    /// the digest that proves what it got.
    #[test]
    fn a_first_resolve_fetches_and_records_the_digest() {
        let repo = tempfile::tempdir().unwrap();
        let fixture = tempfile::tempdir().unwrap();
        git_fixture(fixture.path(), "v1", "fixture-skill", "# v1\n");
        // `file://` so it is a git source rather than a directory to read.
        let url = format!("file://{}", fixture.path().display());

        let resolution = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &git_manifest(&url, "v1"),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect("fetches");

        assert_eq!(
            resolution
                .content
                .item(knowledge(), ItemKind::Skill, "fixture-skill")
                .expect("the fetched skill")
                .files[0]
                .1,
            "# v1\n"
        );
        assert_eq!(resolution.packs.len(), 1);
        let record = &resolution.packs[0];
        assert!(record.digest.starts_with("sha256:"), "{record:?}");
        assert_eq!(record.rev.as_deref(), Some("v1"));
        assert_eq!(record.format, 1);
        // Cached, so the next run needs nothing from outside.
        assert!(fetch::cache_path(repo.path(), &record.digest).is_dir());
    }

    /// Case 4: with the pack cached and the lock recording it, a later
    /// resolve spawns nothing — the `FakeRunner` scripts no git, so any
    /// attempt to fetch would fail the test rather than reach a network.
    #[test]
    fn a_cached_pack_resolves_without_spawning_anything() {
        let repo = tempfile::tempdir().unwrap();
        let fixture = tempfile::tempdir().unwrap();
        git_fixture(fixture.path(), "v1", "fixture-skill", "# v1\n");
        // `file://` so it is a git source rather than a directory to read.
        let url = format!("file://{}", fixture.path().display());
        let manifest = git_manifest(&url, "v1");

        let first = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &manifest,
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap();
        let lock = Lock {
            packs: first.packs.clone(),
            ..Lock::default()
        };

        for mode in [ResolveMode::Offline, ResolveMode::Fetching] {
            let fake = FakeRunner::new();
            let again = resolve(repo.path(), &fake, &manifest, &lock, mode).expect("from cache");
            assert!(
                fake.calls().is_empty(),
                "{mode:?} spawned {:?}",
                fake.calls()
            );
            assert!(again.pending.is_empty(), "{mode:?}");
            assert_eq!(
                again
                    .content
                    .item(knowledge(), ItemKind::Skill, "fixture-skill")
                    .expect("still resolved")
                    .files[0]
                    .1,
                "# v1\n"
            );
        }
    }

    /// A commit sha cannot be reached by `--branch`, so it takes its own
    /// path — and must resolve to the same content the tag did.
    #[test]
    fn a_commit_sha_pin_resolves() {
        let repo = tempfile::tempdir().unwrap();
        let fixture = tempfile::tempdir().unwrap();
        let sha = git_fixture(fixture.path(), "v1", "fixture-skill", "# v1\n");
        // `file://` so it is a git source rather than a directory to read.
        let url = format!("file://{}", fixture.path().display());

        let resolution = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &git_manifest(&url, &sha),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect("resolves a sha");
        assert_eq!(
            resolution
                .content
                .item(knowledge(), ItemKind::Skill, "fixture-skill")
                .expect("the fetched skill")
                .files[0]
                .1,
            "# v1\n"
        );
    }

    /// Case 12: a tag that moved resolves to bytes the lock did not record.
    /// Substituting them silently would apply content nobody pinned.
    #[test]
    fn a_moved_tag_fails_naming_the_mismatch_and_writes_nothing() {
        let repo = tempfile::tempdir().unwrap();
        let fixture = tempfile::tempdir().unwrap();
        git_fixture(fixture.path(), "v1", "fixture-skill", "# v1\n");
        // `file://` so it is a git source rather than a directory to read.
        let url = format!("file://{}", fixture.path().display());
        let manifest = git_manifest(&url, "v1");

        let first = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &manifest,
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap();
        let lock = Lock {
            packs: first.packs.clone(),
            ..Lock::default()
        };
        let cached = fetch::cache_path(repo.path(), &first.packs[0].digest);
        fs::remove_dir_all(&cached).expect("force a re-fetch");

        // The same tag, different content.
        git_fixture(fixture.path(), "v1", "fixture-skill", "# tampered\n");

        let err = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &manifest,
            &lock,
            ResolveMode::Fetching,
        )
        .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("different bytes"), "{message}");
        assert!(message.contains(&first.packs[0].digest), "{message}");
        assert!(!cached.is_dir(), "nothing unverified may be left cached");
        // Nothing written means nothing: the rejected bytes must not survive
        // the run that rejected them, even where nothing would read them.
        assert!(
            !repo.path().join(".superdev/cache/packs/.fetch").exists(),
            "staging survived a failed verification"
        );
    }

    /// Case 11: an unreachable source fails, and the embedded pack is not
    /// quietly substituted for what was pinned.
    #[test]
    fn an_unreachable_source_fails_rather_than_falling_back() {
        let repo = tempfile::tempdir().unwrap();
        let absent = format!("file://{}", repo.path().join("nowhere").display());
        let err = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &git_manifest(&absent, "v1"),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("clone"), "{message}");
        assert!(message.contains(&absent), "names the source: {message}");
    }

    /// Every superdev repo is a git repo, so a user without `git` is in an
    /// unusual state and deserves better than "not found".
    #[test]
    fn a_missing_git_says_what_to_install() {
        let repo = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.missing("git");
        let err = resolve(
            repo.path(),
            &fake,
            &git_manifest("github:someone/other", "v1"),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("needs `git`"), "{message}");
        assert!(message.contains("local path source"), "{message}");
    }

    /// A fetched pack from a source that is not the default layers over the
    /// embedded one, adding without removing. (Base *replacement* with a
    /// fetched pack needs the real default source, so its semantics are
    /// covered where they live, over `ContentSet`.)
    #[test]
    fn a_fetched_pack_layers_over_the_embedded_one() {
        let repo = tempfile::tempdir().unwrap();
        let fixture = tempfile::tempdir().unwrap();
        git_fixture(fixture.path(), "assets-v9.0.0", "only-skill", "# only\n");
        let url = format!("file://{}", fixture.path().display());
        let manifest = git_manifest(&url, "assets-v9.0.0");

        let resolution = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &manifest,
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap();
        assert!(
            resolution
                .content
                .item(knowledge(), ItemKind::Skill, "only-skill")
                .is_some()
        );
        assert!(
            resolution
                .content
                .item(knowledge(), ItemKind::Skill, "frame")
                .is_some(),
            "a layering pack adds without removing"
        );
    }

    #[test]
    fn two_entries_naming_one_source_are_refused() {
        let repo = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.2.0", &[]);
        manifest.packs = vec![
            PackEntry {
                source: "github:six5536/superdev".into(),
                rev: Some(DEFAULT_PACK.rev.into()),
            },
            PackEntry {
                source: "git@github.com:six5536/superdev.git".into(),
                rev: Some(DEFAULT_PACK.rev.into()),
            },
        ];
        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest,
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("git@github.com:six5536/superdev.git"),
            "{message}"
        );
        assert!(message.contains("github:six5536/superdev"), "{message}");
        assert!(message.contains("appears once"), "{message}");
    }

    #[test]
    fn a_missing_pack_directory_names_the_pack_and_the_path() {
        let repo = tempfile::tempdir().unwrap();
        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/absent", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("./packs/absent"), "{message}");
        assert!(message.contains(PACK_MANIFEST), "{message}");
    }

    #[test]
    fn a_pack_carrying_a_refused_path_contributes_nothing() {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path().join("packs/acme");
        write_pack(&dir, "brand-new", "# Brand new\n");
        fs::create_dir_all(dir.join("agents")).unwrap();
        fs::write(dir.join("agents/aokf.md"), "not yours to ship\n").unwrap();
        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap_err();
        assert!(err.to_string().contains("agents/aokf.md"), "{err}");
    }

    /// A relative source is the repo's, whatever directory the command runs
    /// from: the manifest is committed and has to mean one thing.
    #[test]
    fn a_relative_source_resolves_against_the_repo_root() {
        let repo = tempfile::tempdir().unwrap();
        write_pack(
            &repo.path().join("packs/acme"),
            "brand-new",
            "# Brand new\n",
        );
        let elsewhere = tempfile::tempdir().unwrap();
        // Resolving with a different root finds nothing, which is what makes
        // the path the repo's rather than the process's.
        assert!(
            resolve(
                elsewhere.path(),
                &FakeRunner::new(),
                &manifest_with("./packs/acme", None),
                &Lock::default(),
                ResolveMode::Fetching,
            )
            .is_err()
        );
    }
}
