//! pack/source.rs — where a pack comes from, and the key that decides whether
//! it replaces the embedded pack or layers over it.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::manifest::PackEntry;

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
    /// has no revision to pin.
    pub fn parse(entry: &PackEntry) -> Result<PackSource> {
        let source = entry.source.trim();
        if is_git(source) {
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

    /// The same source with a relative path settled against the repo root.
    ///
    /// A path source's identity is a location on this machine, so where it
    /// points has to be settled before anything compares it: `./packs/acme`
    /// and `packs/acme` are one directory and must be one pack. Relative to
    /// the repo, not the process: the manifest is committed, so it means the
    /// same thing wherever the command runs from.
    pub fn rooted(&self, root: &Path) -> PackSource {
        let PackSource::Path { path } = self else {
            return self.clone();
        };
        let joined = if path.is_absolute() {
            path.clone()
        } else {
            root.join(path)
        };
        // `canonicalize` settles `.`, `..` and symlinks, and needs the
        // directory to exist. A missing one keeps the joined path, so the
        // error that follows names where superdev actually looked.
        PackSource::Path {
            path: joined.canonicalize().unwrap_or(joined),
        }
    }

    /// The comparison key every spelling of one source shares.
    ///
    /// Scheme, userinfo, port, a `.git` suffix and any trailing slash are
    /// removed and host and path lowercased, so `github:six5536/superdev`,
    /// `https://github.com/six5536/superdev.git` and the ssh form are one
    /// source. A path source's key is its canonicalised absolute path, which
    /// [`PackSource::rooted`] settles. ADR-004.
    pub fn identity(&self) -> String {
        match self {
            PackSource::Git { url, .. } => git_identity(url),
            PackSource::Path { path } => path.to_string_lossy().trim_end_matches('/').to_string(),
        }
    }

    /// Whether this names the repository the embedded pack is a copy of.
    ///
    /// Compared on the normalised identity, so every spelling of that
    /// repository is the default: comparing the strings would treat three of
    /// the four common forms as a stranger's pack. ADR-004.
    pub fn is_default(&self) -> bool {
        matches!(self, PackSource::Git { .. })
            && self.identity() == git_identity(DEFAULT_PACK.source)
    }

    /// The URL to hand `git`.
    ///
    /// `github:owner/repo` is superdev's own shorthand and git does not know
    /// it — git reads a colon with no dot before it as the scp form and goes
    /// looking for a host called `github`. Until now the default pin always
    /// resolved from the binary, so nothing ever handed it to git; `update`
    /// moving that pin ahead is what makes the expansion load-bearing. Only
    /// the named forges are expanded; every other spelling, an ssh alias
    /// included, is git's own and passes through untouched.
    pub fn clone_url(&self) -> String {
        match self {
            PackSource::Git { url, .. } => match shorthand(url) {
                Some((host, path)) => format!("https://{host}.com/{path}"),
                None => url.clone(),
            },
            PackSource::Path { path } => path.to_string_lossy().into_owned(),
        }
    }
}

/// The forges `<name>:owner/repo` is shorthand for, each at `<name>.com`.
///
/// An allowlist and not "any bare word before a colon": a bare word is an ssh
/// alias or an `insteadOf` prefix at least as often as it is a forge, and
/// those belong to the user's git config, which superdev cannot see. Naming
/// the forges is also what keeps the shorthand a documented spelling rather
/// than a guess a reader has to run to confirm.
const SHORTHAND_FORGES: &[&str] = &["github", "gitlab"];

/// Split superdev's `forge:owner/repo` shorthand into its host and path.
///
/// The forge name is matched whole, so the scp form the same forge is also
/// written as — `github.com:o/r`, `git@github.com:o/r` — is not matched and
/// not expanded a second time into `github.com.com`.
fn shorthand(url: &str) -> Option<(&str, &str)> {
    if url.contains("://") {
        return None;
    }
    let (before, after) = url.split_once(':')?;
    let forge = SHORTHAND_FORGES
        .iter()
        .any(|f| before.eq_ignore_ascii_case(f));
    forge.then_some((before, after))
}

/// Whether a source names a git repository rather than a directory.
///
/// Git is what has to be recognised: anything unrecognised is treated as a
/// path, so a mistyped directory fails saying the directory is missing rather
/// than failing inside a clone.
fn is_git(source: &str) -> bool {
    if source.contains("://") {
        return true;
    }
    // `host:owner/repo` shorthand, and the scp form `user@host:path`. A
    // Windows drive letter is one character before the colon, which no host
    // or shorthand is.
    match source.split_once(':') {
        Some((before, _)) => before.len() > 1 && !before.contains('/') && !before.starts_with('.'),
        None => false,
    }
}

/// Normalise a git URL to `host/path`.
fn git_identity(url: &str) -> String {
    let rest = match url.split_once("://") {
        Some((_, rest)) => rest.to_string(),
        // Shorthand: `github:owner/repo` means github.com. Anything else
        // with a colon is the scp form, whose authority is what precedes it.
        None => match shorthand(url) {
            Some((host, path)) => format!("{host}.com/{path}"),
            None => match url.split_once(':') {
                Some((authority, path)) => format!("{authority}/{path}"),
                None => url.to_string(),
            },
        },
    };
    // Userinfo and a port belong to the authority, which ends at the first
    // `/`. Stripping either across the whole string would let an `@` or a
    // `:` in the path decide the identity — and a source whose *path*
    // contained `@github.com/six5536/superdev` would normalise to the
    // default pack's key and be treated as the base.
    let (authority, path) = rest.split_once('/').unwrap_or((rest.as_str(), ""));
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = host.split_once(':').map_or(host, |(h, _)| h);
    // A leading slash is the scp form's absolute path (`host:/srv/x`) and
    // the doubled slash of `ssh://host//srv/x`: one repository either way.
    let path = path
        .trim_start_matches('/')
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .trim_end_matches('/');
    if path.is_empty() {
        host.to_ascii_lowercase()
    } else {
        format!("{host}/{path}").to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(source: &str, rev: Option<&str>) -> PackEntry {
        PackEntry {
            source: source.into(),
            rev: rev.map(Into::into),
        }
    }

    fn identity_of(source: &str) -> String {
        PackSource::parse(&entry(source, Some("v1")))
            .expect("a git source")
            .identity()
    }

    /// ADR-004's equivalence class: every spelling of one repository is one
    /// source, or a rev that drops an item stops removing it with nothing on
    /// screen to say why.
    #[test]
    fn every_spelling_of_one_repository_shares_an_identity() {
        let expected = "github.com/six5536/superdev";
        for source in [
            "github:six5536/superdev",
            "https://github.com/six5536/superdev",
            "https://github.com/six5536/superdev.git",
            "https://github.com/six5536/superdev.git/",
            "git@github.com:six5536/superdev.git",
            "ssh://git@github.com/six5536/superdev.git",
            "https://GitHub.com/SIX5536/superdev.git",
        ] {
            assert_eq!(identity_of(source), expected, "{source}");
        }
    }

    /// The scp form without a user is a host, not superdev's shorthand.
    /// Expanding it again would key it as `github.com.com/...`, so the same
    /// repository written that way would not be recognised as the base.
    #[test]
    fn the_scp_form_is_not_expanded_as_shorthand() {
        assert_eq!(
            identity_of("github.com:six5536/superdev"),
            "github.com/six5536/superdev"
        );
    }

    /// git does not know `github:owner/repo` — it reads the colon as the scp
    /// form and looks for a host called `github`. Every other spelling is
    /// git's own and must reach it unchanged, or a user's ssh config, their
    /// mirror and their `insteadOf` rules all stop applying.
    #[test]
    fn only_the_shorthand_is_expanded_for_git() {
        let url_of = |source: &str| {
            PackSource::parse(&entry(source, Some("v1")))
                .expect("a git source")
                .clone_url()
        };
        assert_eq!(
            url_of("github:six5536/superdev"),
            "https://github.com/six5536/superdev"
        );
        assert_eq!(
            url_of("gitlab:six5536/superdev"),
            "https://gitlab.com/six5536/superdev"
        );
        for untouched in [
            "https://github.com/six5536/superdev.git",
            "git@github.com:six5536/superdev.git",
            "ssh://git@github.com/six5536/superdev.git",
            "github.com:six5536/superdev",
            // A bare word before the colon is an ssh alias or an `insteadOf`
            // prefix as often as it is a forge name, and rewriting one into
            // `https://gh.com/...` breaks a clone that worked. Only the
            // forges superdev names are expanded; the rest are git's.
            "gh:acme/packs",
            "work:acme/packs",
        ] {
            assert_eq!(url_of(untouched), untouched, "{untouched}");
        }
    }

    /// An alias is not a forge, so it is not keyed as one either: `gh:` and
    /// `github:` are different sources until the user's git says otherwise,
    /// and guessing they are the same would let an alias match the base pack.
    #[test]
    fn an_alias_is_not_keyed_as_a_forge() {
        assert_eq!(identity_of("gh:six5536/superdev"), "gh/six5536/superdev");
        assert_ne!(
            identity_of("gh:six5536/superdev"),
            identity_of("github:six5536/superdev")
        );
    }

    /// The default source is recognised however it is spelled, and nothing
    /// else is: `update` moves this pin and no other, so a false positive
    /// here would move a stranger's pin and a false negative would strand
    /// superdev's own.
    #[test]
    fn the_default_source_is_recognised_by_identity_and_nothing_else_is() {
        for spelling in [
            "github:six5536/superdev",
            "https://github.com/six5536/superdev.git",
            "git@github.com:six5536/superdev.git",
        ] {
            let source = PackSource::parse(&entry(spelling, Some("v1"))).unwrap();
            assert!(source.is_default(), "{spelling}");
        }
        for other in [
            "github:six5536/superdev-forks",
            "https://evil.example/x@github.com/six5536/superdev",
        ] {
            let source = PackSource::parse(&entry(other, Some("v1"))).unwrap();
            assert!(!source.is_default(), "{other}");
        }
        let path = PackSource::parse(&entry("./packs/acme", None)).unwrap();
        assert!(
            !path.is_default(),
            "a directory is never the default source"
        );
    }

    #[test]
    fn distinct_repositories_do_not_share_an_identity() {
        let mine = identity_of("github:six5536/superdev");
        for other in [
            "github:six5536/superdev-forks",
            "github:someone/superdev",
            "gitlab:six5536/superdev",
            "https://example.com/six5536/superdev.git",
        ] {
            assert_ne!(identity_of(other), mine, "{other}");
        }
    }

    /// Userinfo and a port belong to the authority. An `@` or a `:` further
    /// along is part of the path and must not decide the identity — a source
    /// whose path ended `@github.com/six5536/superdev` would otherwise
    /// normalise to the default pack's key and be treated as the base,
    /// replacing the embedded content wholesale.
    #[test]
    fn only_the_authority_is_stripped_so_a_foreign_source_cannot_impersonate() {
        let default = identity_of("github:six5536/superdev");
        for impostor in [
            "https://evil.example/x@github.com/six5536/superdev",
            "https://evil.example/six5536/superdev@github.com",
        ] {
            assert_ne!(identity_of(impostor), default, "{impostor}");
        }
        // An `@` in the path stays in the path.
        assert_eq!(
            identity_of("https://github.com/six5536/super@dev"),
            "github.com/six5536/super@dev"
        );
        // Two repositories differing only after an `@` stay distinct.
        assert_ne!(
            identity_of("https://github.com/a/b@c"),
            identity_of("https://example.com/x/y@c")
        );
    }

    /// A port is not part of which repository a URL names, so the same repo
    /// with one written out is the same source.
    #[test]
    fn an_explicit_port_does_not_split_the_equivalence_class() {
        assert_eq!(
            identity_of("ssh://git@github.com:22/six5536/superdev.git"),
            identity_of("github:six5536/superdev")
        );
    }

    /// A path source's identity is a location, so two spellings of one
    /// directory must be one pack — otherwise the same pack layers twice and
    /// is reported as shadowing itself.
    #[test]
    fn two_spellings_of_one_directory_share_an_identity() {
        let repo = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(repo.path().join("packs/acme")).unwrap();
        // `..` resolves through a directory that exists, not a lexical one.
        std::fs::create_dir_all(repo.path().join("packs/other")).unwrap();
        let of = |source: &str| {
            PackSource::parse(&entry(source, None))
                .expect("a path source")
                .rooted(repo.path())
                .identity()
        };
        let expected = of("./packs/acme");
        for spelling in [
            "packs/acme",
            "./packs/acme",
            "packs/./acme",
            "packs/other/../acme",
        ] {
            assert_eq!(of(spelling), expected, "{spelling}");
        }
        assert_ne!(of("packs/elsewhere"), expected);
        // Settled against the repo, not the process: absolute either way.
        assert!(std::path::Path::new(&expected).is_absolute(), "{expected}");
    }

    /// A directory that does not exist cannot be canonicalised; the joined
    /// path keeps the error naming where superdev actually looked.
    #[test]
    fn a_missing_directory_still_roots_against_the_repo() {
        let repo = tempfile::tempdir().unwrap();
        let rooted = PackSource::parse(&entry("./packs/absent", None))
            .unwrap()
            .rooted(repo.path());
        let PackSource::Path { path } = rooted else {
            panic!("a path source");
        };
        assert!(path.starts_with(repo.path()), "{}", path.display());
    }

    #[test]
    fn a_git_source_without_a_rev_is_refused() {
        let err = PackSource::parse(&entry("github:six5536/superdev", None)).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("github:six5536/superdev"), "{message}");
        assert!(message.contains("rev"), "{message}");
    }

    #[test]
    fn a_path_source_with_a_rev_is_refused() {
        let err = PackSource::parse(&entry("./packs/acme", Some("v1"))).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("./packs/acme"), "{message}");
        assert!(message.contains("no `rev`"), "{message}");
    }

    #[test]
    fn a_directory_is_a_path_source_and_a_url_is_not() {
        assert!(matches!(
            PackSource::parse(&entry("./packs/acme", None)).unwrap(),
            PackSource::Path { .. }
        ));
        assert!(matches!(
            PackSource::parse(&entry("/srv/packs/acme", None)).unwrap(),
            PackSource::Path { .. }
        ));
        // A bare relative path is a path, not a mistyped URL.
        assert!(matches!(
            PackSource::parse(&entry("../acme", None)).unwrap(),
            PackSource::Path { .. }
        ));
        assert!(matches!(
            PackSource::parse(&entry("github:a/b", Some("v1"))).unwrap(),
            PackSource::Git { .. }
        ));
    }

    /// The embedded pack and the pin that claims to describe it are set by
    /// one release command; this is what catches them drifting apart if it
    /// ever sets only one.
    #[test]
    fn the_default_pin_names_the_embedded_packs_version() {
        let pack_toml = crate::content::pack_manifest_source();
        let version = pack_toml
            .lines()
            .find_map(|line| line.strip_prefix("version"))
            .and_then(|rest| rest.split('"').nth(1))
            .expect("pack.toml carries a version");
        assert_eq!(
            DEFAULT_PACK.rev,
            format!("{PACK_TAG_PREFIX}{version}"),
            "DEFAULT_PACK.rev must name the tag /pack/pack.toml's version cuts"
        );
    }
}
