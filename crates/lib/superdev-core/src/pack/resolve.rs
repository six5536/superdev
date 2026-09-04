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
use super::manifest::{PACK_MANIFEST, PackManifest, SUPPORTED_FORMATS, check_path, link_refusal};
use super::source::{DEFAULT_PACK, PackSource};

// The resolution phase is the pack resolution contract's Definition
// (contract-007): the `pack-resolution` regions below hold the mode, what a
// resolve produces and `resolve` itself, body included — a region is lines,
// and the signature's brace opens the body.
// sokf:begin pack-resolution
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

// sokf:end pack-resolution

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

// sokf:begin pack-resolution
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
// sokf:end pack-resolution

/// Whether an entry names the source the embedded pack is a copy of.
///
/// Compared on the normalised identity, so every spelling of that repository
/// is the base: comparing the strings would treat three of the four common
/// forms as a stranger's pack, and removals would stop propagating with
/// nothing on screen to say why. ADR-004.
fn is_base(source: &PackSource) -> bool {
    // Not an identity comparison of its own: a path key is relative now, so it
    // can read exactly like a repository's, and only the source kind tells the
    // two apart. `is_default` is where that guard lives. ADR-011.
    source.is_default()
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
            // A directory has no rev to pin, so it is read every run and
            // there is nothing for a digest to be checked against — the point
            // of a path source being that editing it lands without a re-pin.
            // Recorded, the value would be rewritten by every commit touching
            // the pack and read by nothing. ADR-016.
            let (items, _) = read_pack(&entry.source, path)?;
            Ok(Resolved::Layer(items, record(root, entry, source, None)))
        }
        PackSource::Git { rev, .. } => {
            // A pin naming exactly what this binary carries is the default
            // path written out, and must cost no request.
            if is_base(source) && rev == DEFAULT_PACK.rev {
                return Ok(Resolved::Embedded);
            }
            let locked = lock.packs.iter().find(|p| {
                p.identity == source.identity(root) && p.rev.as_deref() == Some(rev.as_str())
            });

            // Cached from an earlier resolve of this same rev: the bytes are
            // already here and already proven, so neither mode reaches out.
            if let Some(locked) = locked {
                // A git source always recorded one; `as_deref` is because
                // the field is now optional for the path arm's sake.
                if let Some(digest) = locked.digest.as_deref() {
                    let cached = fetch::cache_path(root, digest);
                    if cached.is_dir() {
                        let (items, files) = read_pack(&entry.source, &cached)?;
                        return verified(root, entry, source, items, &files, Some(digest));
                    }
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
        root,
        entry,
        source,
        items,
        &files,
        locked.and_then(|l| l.digest.as_deref()),
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
    root: &Path,
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
    Ok(Resolved::Layer(
        items,
        record(root, entry, source, Some(&digest)),
    ))
}

/// The lock record for one resolved pack.
///
/// `root` because a path source's key is relative to it — the lock is
/// committed, and an absolute one would be this checkout's alone. ADR-011.
fn record(root: &Path, entry: &PackEntry, source: &PackSource, digest: Option<&str>) -> PackLock {
    PackLock {
        source: entry.source.clone(),
        identity: source.identity(root),
        rev: entry.rev.clone(),
        digest: digest.map(str::to_string),
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
    // The root and the manifest are pack paths like any other, and neither
    // `is_dir` nor `read_to_string` can tell a link from the thing it points
    // at. Left unchecked they are the same hole the walk closes, one level
    // up: a linked root walks a directory outside the pack, and a linked
    // manifest decides the format gate on bytes the digest never covers —
    // two packs declaring different formats would digest alike. I008.
    refuse_a_link(pack, dir)?;
    let manifest_path = dir.join(PACK_MANIFEST);
    refuse_a_link(pack, &manifest_path)?;
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

/// Refuse a path that is a symlink, naming it.
///
/// One rule for the whole tree: the pack root, its manifest, and every path
/// the walk meets. A pack resolves whole or not at all, so a link cannot be
/// stepped over — that leaves the pack shipping everything but the item its
/// author meant the link to stand for, with nothing said. ADR-014, I009.
fn refuse_a_link(pack: &str, path: &Path) -> Result<()> {
    let linked = fs::symlink_metadata(path)
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false);
    if linked {
        return Err(link_refusal(pack, &path.display().to_string()));
    }
    Ok(())
}

/// Collect every file under `dir` as (path relative to `root`, content).
///
/// Paths are forward-slashed so a pack means the same thing on every
/// platform. `.git` is skipped: a path source is often a working checkout,
/// and its history is not content. Every symlink fails the walk, naming
/// itself: a linked file read through would put bytes from outside the pack
/// into the repo as pack content, and a linked directory pointing back at an
/// ancestor would be walked until the OS refused and report a path forty
/// `loop/` deep instead of what was wrong. Refused where it is met, so
/// neither happens and the pack does not resolve short of what it declares.
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
        // link from the thing it points at. Every link is refused, not only a
        // linked directory: `is_dir` follows, so a linked *file* answers
        // false and would fall through to be read — and `read_to_string`
        // follows too, so the bytes would come from wherever it pointed and
        // be written into the repo as pack content. A pack names its own
        // paths; a link is how it names one it does not contain. I008, I009.
        // Failing to answer is not the same as answering no. `read_dir` just
        // named this path, so a failure here is a race or a permission the
        // pack author needs to know about — and stepping over it would leave
        // the pack short of a file it declares with nothing said, which is
        // the shape this slice closes.
        let meta = fs::symlink_metadata(&path).map_err(|e| Error::Pack {
            pack: pack.to_string(),
            message: format!("{}: {e}", path.display()),
        })?;
        if meta.file_type().is_symlink() {
            return Err(link_refusal(pack, &path.display().to_string()));
        }
        if meta.is_dir() {
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
        Owner::Knowledge
    }

    /// A directory and a repository are different sources however alike their
    /// keys read. Since a path key became relative it can spell a repository
    /// exactly, and refusing the pair as duplicates would fail a manifest that
    /// is perfectly valid — a vendored tree beside the repo it mirrors.
    /// ADR-011.
    #[test]
    fn a_directory_spelling_a_repository_is_not_a_duplicate_of_it() {
        let repo = tempfile::tempdir().unwrap();
        let masquerade = "github.com/six5536/superdev";
        write_pack(
            &repo.path().join(masquerade),
            "acme-vendored",
            "# vendored\n",
        );

        let mut manifest = manifest_with(DEFAULT_PACK.source, Some(DEFAULT_PACK.rev));
        manifest.packs.push(PackEntry {
            source: masquerade.into(),
            rev: None,
        });

        let resolved = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest,
            &Lock::default(),
            ResolveMode::Offline,
        )
        .expect("a directory and a repository are two sources, not one named twice");
        assert!(
            resolved
                .content
                .item(knowledge(), ItemKind::Skill, "acme-vendored")
                .is_some(),
            "the vendored pack layered"
        );
    }

    /// A pack names its own file paths, and a symlink lets it name one it
    /// does not contain. The written path stays inside the pack, so nothing
    /// escapes on the way out — what escapes is the content, read from
    /// wherever the link points and written into the repo as pack content.
    /// I008.
    ///
    /// The link is refused rather than skipped, so nothing is written at all:
    /// `resolve` fails before there is a content set for the pipeline to plan
    /// from. Skipping closed the leak but left the pack resolving without an
    /// item it meant to ship, which is the half I009 reopened. ADR-014.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_file_in_a_pack_is_refused_and_leaks_nothing() {
        let repo = tempfile::tempdir().unwrap();
        let outside = repo.path().join("secret.txt");
        fs::write(&outside, "SUPER-SECRET\n").unwrap();
        let pack = repo.path().join("packs/acme");
        write_pack(&pack, "honest", "# honest\n");
        let leak = pack.join("knowledge/skills/leak");
        fs::create_dir_all(&leak).unwrap();
        std::os::unix::fs::symlink(&outside, leak.join("SKILL.md")).unwrap();

        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect_err("a link in the tree stops the pack");

        let message = err.to_string();
        assert!(message.contains("is a symlink"), "{message}");
        assert!(
            message.contains("knowledge/skills/leak/SKILL.md"),
            "the refusal does not name the path: {message}"
        );
        assert!(
            !message.contains("SUPER-SECRET"),
            "the target's bytes were read: {message}"
        );
    }

    /// The manifest decides the format gate, and the walk never sees it — so
    /// a link there would pick the gate with bytes no digest covers, and two
    /// packs declaring different formats would digest alike. Refused, not
    /// skipped: without a manifest there is no pack. I008.
    #[cfg(unix)]
    #[test]
    fn a_linked_pack_manifest_is_refused() {
        let repo = tempfile::tempdir().unwrap();
        let elsewhere = repo.path().join("elsewhere.toml");
        fs::write(
            &elsewhere,
            "format = 1\nname = \"x\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let pack = repo.path().join("packs/acme");
        write_pack(&pack, "honest", "# honest\n");
        fs::remove_file(pack.join(PACK_MANIFEST)).unwrap();
        std::os::unix::fs::symlink(&elsewhere, pack.join(PACK_MANIFEST)).unwrap();

        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect_err("a linked manifest");

        assert!(format!("{err}").contains("is a symlink"), "{err}");
    }

    /// The pack root is the pack. A link there walks a directory somewhere
    /// else entirely and calls all of it content.
    ///
    /// Asserted against `read_pack` rather than through `resolve`, because a
    /// *path* source cannot reach it: `rooted` canonicalises, so a link the
    /// user named is resolved before the pack is read, which is the point of
    /// canonicalising. The root that arrives unresolved is a fetched pack's —
    /// `<checkout>/pack`, whatever the cloned repository put there.
    #[cfg(unix)]
    #[test]
    fn a_linked_pack_root_is_refused() {
        let repo = tempfile::tempdir().unwrap();
        write_pack(&repo.path().join("real"), "honest", "# honest\n");
        let linked = repo.path().join("checkout-pack");
        std::os::unix::fs::symlink(repo.path().join("real"), &linked).unwrap();

        let err = read_pack("acme", &linked).expect_err("a linked root");

        assert!(format!("{err}").contains("is a symlink"), "{err}");
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
        write_pack(&repo.path().join("packs/acme"), "scope", "# Ours\n");
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
            .item(knowledge(), ItemKind::Skill, "scope")
            .expect("scope");
        assert_eq!(item.files[0].1, "# Ours\n");
        // Everything the pack does not carry still comes from the embedded
        // copy.
        assert!(
            resolved
                .content
                .item(knowledge(), ItemKind::Skill, "integrate")
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
                    .item(knowledge(), ItemKind::Skill, "scope")
                    .is_some(),
                "{mode:?}: the embedded pack still supplies the items"
            );
            // Layer 0 is the embedded pack, and naming it again does not
            // make it a pack layer.
            assert_eq!(
                resolved
                    .content
                    .origin(knowledge(), ItemKind::Skill, "scope"),
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
        write_pack(&repo.path().join("packs/acme"), "scope", "# Ours\n");
        let resolved = resolved(repo.path(), &["./packs/acme"]);
        assert_eq!(
            resolved
                .content
                .item(knowledge(), ItemKind::Skill, "scope")
                .expect("scope")
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
    /// reader could act on. It is refused where it is met, so the path the
    /// error names is the link itself. ADR-014.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_directory_inside_a_pack_is_refused() {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path().join("packs/acme");
        write_pack(&dir, "brand-new", "# Brand new\n");
        std::os::unix::fs::symlink(&dir, dir.join("loop")).unwrap();
        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect_err("the cycle is refused, not walked");
        let message = err.to_string();
        assert!(message.contains("is a symlink"), "{message}");
        assert!(message.ends_with("elsewhere"), "walked into it: {message}");
        assert!(
            message.contains("/loop") && !message.contains("loop/loop"),
            "the refusal does not name the link itself: {message}"
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
        assert!(
            record
                .digest
                .as_deref()
                .is_some_and(|d| d.starts_with("sha256:")),
            "{record:?}"
        );
        assert_eq!(record.rev.as_deref(), Some("v1"));
        assert_eq!(record.format, 1);
        // Cached, so the next run needs nothing from outside.
        assert!(fetch::cache_path(repo.path(), record.digest.as_deref().unwrap()).is_dir());
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

    /// A fetched pack is read back from the cache on every later run, and the
    /// cache is a directory on this machine with no index of its own. So the
    /// filesystem check has to hold there too, not only for a path pack —
    /// otherwise a link that appeared in the cache after the fetch that
    /// checked it would be read straight through. ADR-014.
    ///
    /// Planted rather than fetched: what git's index says at fetch time is
    /// slice 3's check, and this is the one that runs afterwards.
    #[cfg(unix)]
    #[test]
    fn a_symlink_in_a_cached_pack_is_refused_when_it_is_read_back() {
        let repo = tempfile::tempdir().unwrap();
        let fixture = tempfile::tempdir().unwrap();
        git_fixture(fixture.path(), "v1", "fixture-skill", "# v1\n");
        let url = format!("file://{}", fixture.path().display());
        let manifest = git_manifest(&url, "v1");

        let first = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &manifest,
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect("the pack fetches clean");
        let lock = Lock {
            packs: first.packs.clone(),
            ..Lock::default()
        };
        let cached = fetch::cache_path(repo.path(), first.packs[0].digest.as_deref().unwrap());

        let outside = repo.path().join("secret.txt");
        fs::write(&outside, "SUPER-SECRET\n").unwrap();
        let planted = cached.join("knowledge/skills/fixture-skill/EXTRA.md");
        std::os::unix::fs::symlink(&outside, &planted).unwrap();

        // Offline: the cache is what is read, and no fetch stands between the
        // plant and the check.
        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest,
            &lock,
            ResolveMode::Offline,
        )
        .expect_err("the cached link stops the run");

        let message = err.to_string();
        assert!(message.contains("is a symlink"), "{message}");
        assert!(message.contains("EXTRA.md"), "{message}");
    }

    /// superdev's own pack must keep resolving, and it is the pack every
    /// third-party author copies — so the day a link appears under `/pack/`
    /// is the day this fails, rather than the day a release does. ADR-014.
    #[test]
    fn superdevs_own_pack_carries_no_symlink() {
        // The crate's `assets` is itself a relative link to `/pack/`, which is
        // what keeps the files inside the published crate — and is exactly
        // what `read_pack` refuses in a pack root, so follow it to the
        // directory the manifest actually pins. Its *contents* are what the
        // pack ships, and none of those may be a link.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .canonicalize()
            .expect("the crate's assets link resolves to /pack");
        let mut links = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).expect("the pack is readable") {
                let path = entry.expect("an entry").path();
                let meta = fs::symlink_metadata(&path).expect("its type");
                if meta.file_type().is_symlink() {
                    links.push(path.display().to_string());
                } else if meta.is_dir() {
                    stack.push(path);
                }
            }
        }
        assert!(
            links.is_empty(),
            "superdev's own pack ships a symlink: {links:?}"
        );
        assert!(
            read_pack("superdev", &root).is_ok(),
            "superdev's own pack no longer resolves"
        );
    }

    /// I004: a path source records no digest.
    ///
    /// There is nothing for one to be checked against — a directory is read
    /// afresh every run — so a recorded value was written by every commit
    /// touching the pack and read by nothing. ADR-016.
    #[test]
    fn a_path_pack_records_no_digest() {
        let repo = tempfile::tempdir().unwrap();
        write_pack(&repo.path().join("packs/acme"), "brand-new", "# new\n");

        let resolved = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect("a path pack resolves");

        assert_eq!(resolved.packs.len(), 1);
        assert_eq!(resolved.packs[0].digest, None);
        // Absent exactly when `rev` is: the two describe pinned bytes, and a
        // directory has neither.
        assert_eq!(resolved.packs[0].rev, None);
    }

    /// The churn this closes. Editing a file under a path pack and resolving
    /// again leaves its lock record byte-identical, so a commit touching the
    /// pack no longer rewrites a line nothing reads. I004.
    #[test]
    fn editing_a_path_pack_leaves_its_lock_record_unchanged() {
        let repo = tempfile::tempdir().unwrap();
        let pack = repo.path().join("packs/acme");
        write_pack(&pack, "brand-new", "# before\n");
        let resolve_once = || {
            resolve(
                repo.path(),
                &FakeRunner::new(),
                &manifest_with("./packs/acme", None),
                &Lock::default(),
                ResolveMode::Fetching,
            )
            .expect("a path pack resolves")
            .packs
        };

        let before = resolve_once();
        fs::write(
            pack.join("knowledge/skills/brand-new/SKILL.md"),
            "# after, and quite different\n",
        )
        .unwrap();
        let after = resolve_once();

        let written = |packs| {
            toml_edit::ser::to_string_pretty(&Lock {
                packs,
                ..Lock::default()
            })
            .expect("the lock serialises")
        };
        assert_eq!(
            written(before),
            written(after),
            "editing the pack rewrote its lock record"
        );
    }

    /// A lock written before this still parses, and loses only that field —
    /// so a repo does not need a `sync` before its next `status` works.
    #[test]
    fn a_lock_written_before_this_still_parses() {
        let written = "[[packs]]\nsource = \"./pack\"\nidentity = \"pack\"\n\
             digest = \"sha256:9f2a\"\nformat = 1\n";
        let lock: Lock = toml_edit::de::from_str(written).expect("an older lock still parses");
        assert_eq!(lock.packs[0].digest.as_deref(), Some("sha256:9f2a"));
        assert_eq!(lock.packs[0].identity, "pack");
    }

    /// End to end: a real clone of a real repository whose pack holds a link.
    /// Refused at fetch, on git's answer rather than the filesystem's, and
    /// before a digest exists to record. ADR-014.
    #[cfg(unix)]
    #[test]
    fn a_fetched_pack_with_a_symlink_is_refused_before_it_is_digested() {
        let repo = tempfile::tempdir().unwrap();
        let fixture = tempfile::tempdir().unwrap();
        git_fixture(fixture.path(), "v1", "fixture-skill", "# v1\n");
        std::os::unix::fs::symlink(
            "../fixture-skill/SKILL.md",
            fixture.path().join("pack/knowledge/skills/LINK.md"),
        )
        .unwrap();
        git_fixture(fixture.path(), "v2", "fixture-skill", "# v2\n");
        let url = format!("file://{}", fixture.path().display());

        let err = resolve(
            repo.path(),
            &crate::runner::SystemRunner,
            &git_manifest(&url, "v2"),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .expect_err("the link stops the fetch");

        let message = err.to_string();
        // The path git reports, repo-relative and with no leading slash —
        // which is what says the *index* refused it. The filesystem check
        // from slice 2 would catch this same link on Linux and name the
        // checkout's absolute path, so asserting only "is a symlink" would
        // pass with this slice removed entirely.
        assert!(
            message.contains(": pack/knowledge/skills/LINK.md is a symlink"),
            "refused by the filesystem rather than the index: {message}"
        );
        // Nothing digested means nothing cached: the refusal lands before
        // there is a digest to name a cache directory with.
        let cache = repo.path().join(".superdev/cache/packs");
        let cached: Vec<_> = fs::read_dir(&cache)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|name| name != ".fetch")
                    .collect()
            })
            .unwrap_or_default();
        assert!(cached.is_empty(), "a refused pack was cached: {cached:?}");
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
        let cached = fetch::cache_path(repo.path(), first.packs[0].digest.as_deref().unwrap());
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
        assert!(
            message.contains(first.packs[0].digest.as_deref().unwrap()),
            "{message}"
        );
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
                .item(knowledge(), ItemKind::Skill, "scope")
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
        fs::write(dir.join("agents/superdev.md"), "not yours to ship\n").unwrap();
        let err = resolve(
            repo.path(),
            &FakeRunner::new(),
            &manifest_with("./packs/acme", None),
            &Lock::default(),
            ResolveMode::Fetching,
        )
        .unwrap_err();
        assert!(err.to_string().contains("agents/superdev.md"), "{err}");
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
