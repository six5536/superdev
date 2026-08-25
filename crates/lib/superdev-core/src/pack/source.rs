//! pack/source.rs — where a pack comes from, and the key that decides whether
//! it replaces the embedded pack or layers over it.

use std::path::PathBuf;

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

    /// The comparison key every spelling of one source shares.
    ///
    /// Scheme, userinfo, a `.git` suffix and any trailing slash are removed
    /// and host and path lowercased, so `github:six5536/superdev`,
    /// `https://github.com/six5536/superdev.git` and the ssh form are one
    /// source. A path source's key is the path it holds, which the resolver
    /// has made absolute. ADR-004.
    pub fn identity(&self) -> String {
        match self {
            PackSource::Git { url, .. } => git_identity(url),
            PackSource::Path { path } => path.to_string_lossy().trim_end_matches('/').to_string(),
        }
    }
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
    // Shorthand: `github:owner/repo` means github.com.
    let rest = match url.split_once("://") {
        Some((_, rest)) => rest.to_string(),
        None => match url.split_once(':') {
            // scp form carries userinfo, shorthand does not.
            Some((before, after)) if before.contains('@') => {
                let host = before.split_once('@').map_or(before, |(_, h)| h);
                format!("{host}/{after}")
            }
            Some((shorthand, after)) => format!("{}.com/{after}", shorthand.to_ascii_lowercase()),
            None => url.to_string(),
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
    let path = path
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
            format!("assets-v{version}"),
            "DEFAULT_PACK.rev must name the tag /pack/pack.toml's version cuts"
        );
    }
}
