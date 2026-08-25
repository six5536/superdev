//! pack/resolve.rs — the phase that turns pack entries into content.
//!
//! Runs before anything plans, so `Component::plan` stays side-effect free
//! and `status` provably never fetches (ADR-002).

use std::fs;
use std::path::{Path, PathBuf};

use crate::content::{ContentSet, Item, Origin, items_from, snapshot_items};
use crate::error::{Error, Result};
use crate::lock::Lock;
use crate::manifest::{Manifest, PackEntry};

use super::manifest::{PACK_MANIFEST, PackManifest, check_path};
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
}

/// What one entry resolved to.
enum Resolved {
    /// Its own items, as a layer above the embedded pack.
    Layer(Vec<Item>),
    /// The embedded pack itself — the pin names exactly what is compiled in,
    /// so there is nothing to fetch.
    Embedded,
    /// Not satisfiable without reaching out, which `Offline` may not do.
    Pending,
}

/// Resolve the manifest's packs over the embedded pack.
///
/// Reads the local paths; the only phase in a run that reads anything
/// outside the repo. An entry pinning exactly what this binary embeds
/// resolves from it and makes no request.
pub fn resolve(
    root: &Path,
    manifest: &Manifest,
    lock: &Lock,
    mode: ResolveMode,
) -> Result<Resolution> {
    // The digests the lock records are what a fetched pack is verified
    // against. Nothing here fetches, so nothing is verified against them yet.
    let _ = lock;

    let mut layers = vec![(snapshot_items(), Origin::Snapshot)];
    let mut pending = Vec::new();
    for (index, entry) in manifest.packs.iter().enumerate() {
        let source = PackSource::parse(entry)?;
        match resolve_one(root, entry, &source, mode)? {
            Resolved::Layer(items) => layers.push((
                items,
                Origin::Pack {
                    index,
                    name: entry.source.clone(),
                },
            )),
            // Already layer 0; naming it again would shadow it with itself.
            Resolved::Embedded => {}
            Resolved::Pending => pending.push(entry.clone()),
        }
    }
    Ok(Resolution {
        content: ContentSet::from_layers(layers),
        pending,
    })
}

/// Resolve one entry.
fn resolve_one(
    root: &Path,
    entry: &PackEntry,
    source: &PackSource,
    mode: ResolveMode,
) -> Result<Resolved> {
    match source {
        PackSource::Path { path } => Ok(Resolved::Layer(read_pack(
            &entry.source,
            &pack_dir(root, path),
        )?)),
        PackSource::Git { rev, .. } => {
            // A pin naming exactly what this binary carries is the default
            // path written out, and must cost no request.
            let default = PackSource::Git {
                url: DEFAULT_PACK.source.to_string(),
                rev: DEFAULT_PACK.rev.to_string(),
            };
            if source.identity() == default.identity() && rev == DEFAULT_PACK.rev {
                return Ok(Resolved::Embedded);
            }
            match mode {
                ResolveMode::Offline => Ok(Resolved::Pending),
                ResolveMode::Fetching => Err(Error::Pack {
                    pack: entry.source.clone(),
                    message: format!("cannot be resolved at `{rev}`"),
                }),
            }
        }
    }
}

/// Every item one pack directory provides.
///
/// `pack.toml` is read first: the format decides whether the rest means what
/// this binary thinks. Every path is checked before any of them becomes an
/// item, so a pack carrying a refused file contributes nothing rather than
/// contributing most of itself.
fn read_pack(pack: &str, dir: &Path) -> Result<Vec<Item>> {
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
    Ok(items_from(
        files
            .iter()
            .map(|(path, body)| (path.as_str(), body.as_str())),
    ))
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

/// Where a path source's directory sits.
///
/// A relative path is the repo's, not the working directory's: the manifest
/// is committed, so it has to mean the same thing wherever the command runs
/// from.
fn pack_dir(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;
    use crate::content::{ItemKind, Owner};

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
                &manifest_with(source, Some(DEFAULT_PACK.rev)),
                &Lock::default(),
                ResolveMode::Fetching,
            )
            .unwrap();
            assert!(resolved.pending.is_empty(), "{source}");
        }
    }

    #[test]
    fn an_unresolvable_git_pin_is_pending_offline_and_an_error_when_fetching() {
        let repo = tempfile::tempdir().unwrap();
        let manifest = manifest_with("github:someone/other", Some("v9"));
        let offline = resolve(
            repo.path(),
            &manifest,
            &Lock::default(),
            ResolveMode::Offline,
        )
        .unwrap();
        assert_eq!(offline.pending.len(), 1);
        let err = resolve(
            repo.path(),
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

    #[test]
    fn a_missing_pack_directory_names_the_pack_and_the_path() {
        let repo = tempfile::tempdir().unwrap();
        let err = resolve(
            repo.path(),
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
        assert_eq!(
            pack_dir(repo.path(), Path::new("./packs/acme")),
            repo.path().join("./packs/acme")
        );
        let elsewhere = tempfile::tempdir().unwrap();
        // Resolving with a different root finds nothing, which is what makes
        // the path the repo's rather than the process's.
        assert!(
            resolve(
                elsewhere.path(),
                &manifest_with("./packs/acme", None),
                &Lock::default(),
                ResolveMode::Fetching,
            )
            .is_err()
        );
    }
}
