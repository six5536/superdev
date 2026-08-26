//! pack/fetch.rs — getting a pinned pack's bytes, and proving they are the
//! bytes that were pinned.
//!
//! A git source is fetched by spawning the user's own `git`, so credentials,
//! ssh agents and forge access are theirs and superdev holds no token
//! (ADR-007). A resolved pack is kept under `.superdev/cache/packs/<digest>/`
//! so a later run needs the network only for bytes this machine does not
//! have (ADR-005).

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::runner::{CommandRunner, Output};

/// Where a resolved pack is kept, under the gitignored machine-state
/// directory that already holds the search index and the backup tree.
const CACHE_DIR: &str = ".superdev/cache/packs";

/// The directory inside a pack repository that holds the pack.
///
/// The sparse checkout takes this and nothing else: a pack repository is
/// usually a whole project, and its history is not content.
const PACK_SUBDIR: &str = "pack";

/// A pack tree's digest: `sha256:` and the hex of a hash over every file's
/// path and bytes, in path order.
///
/// Over paths as well as contents, so moving a file between two names is a
/// different pack — a digest that ignored paths would call the rename
/// identical.
pub fn digest(files: &[(String, String)]) -> String {
    let mut sorted: Vec<&(String, String)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = Sha256::new();
    for (path, body) in sorted {
        // Lengths are hashed too: without them `ab` + `c` and `a` + `bc`
        // would produce the same digest.
        hasher.update(path.len().to_le_bytes());
        hasher.update(path.as_bytes());
        hasher.update(body.len().to_le_bytes());
        hasher.update(body.as_bytes());
    }
    let hex: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

/// Where a pack of this digest is cached.
pub fn cache_path(root: &Path, digest: &str) -> PathBuf {
    // The digest carries a `:` that Windows forbids in a path component.
    root.join(CACHE_DIR).join(digest.replace(':', "-"))
}

/// The `-c` overrides every git command here carries.
///
/// A pack's digest is over the bytes as the pack published them, and the lock
/// recording it is committed and shared. Left to the user's `core.autocrlf`,
/// a checkout on Windows would translate every line ending, so the same rev
/// would digest differently there and a lock written on one platform would
/// fail verification on the other. superdev's clone takes the bytes verbatim
/// whatever the machine is configured to prefer.
///
/// Every git call superdev makes is built from this, `pin.rs`'s query
/// included — the one call that did not was how a manifest still got a
/// command run.
fn overrides() -> Vec<String> {
    vec![
        "-c".into(),
        "core.autocrlf=false".into(),
        "-c".into(),
        "core.eol=lf".into(),
        // An `ext::` URL names a command and git runs it as the connection.
        // Whether it may is `protocol.ext.allow`, which defaults to refusing
        // but is the user's to change — and a manifest is a file that arrives
        // with a repository, so a source is not superdev's to trust. Set for
        // superdev's own calls rather than inherited, so a machine configured
        // to permit the transport does not permit it to a cloned manifest.
        // I007.
        "-c".into(),
        "protocol.ext.allow=never".into(),
    ]
}

/// Fetch a git source into a directory of its own, returning the pack root.
///
/// Shallow, blobless and sparse: the pack directory and not the history.
/// A rev that names a commit takes a second path, because `--branch` accepts
/// a tag or branch and not a sha.
pub fn fetch(
    runner: &dyn CommandRunner,
    pack: &str,
    url: &str,
    rev: &str,
    into: &Path,
) -> Result<PathBuf> {
    fs::create_dir_all(into).map_err(|e| Error::Pack {
        pack: pack.to_string(),
        message: format!("{}: {e}", into.display()),
    })?;
    let checkout = into.join("repo");
    // A previous interrupted attempt would otherwise make `git clone` fail on
    // a non-empty target.
    if checkout.exists() {
        fs::remove_dir_all(&checkout).map_err(|e| Error::Pack {
            pack: pack.to_string(),
            message: format!("{}: {e}", checkout.display()),
        })?;
    }

    let target = checkout.to_string_lossy().into_owned();
    let mut clone: Vec<String> = Vec::new();
    clone.extend([
        "clone".into(),
        "--depth".into(),
        "1".into(),
        "--filter=blob:none".into(),
        "--sparse".into(),
    ]);
    if !looks_like_sha(rev) {
        clone.push("--branch".into());
        clone.push(rev.into());
    }
    // Everything after this is an operand. `parse` already refuses a source
    // or rev beginning with `-`; this is the second lock on that door, so the
    // shape of what a manifest may say stops being load-bearing here.
    clone.push("--".into());
    clone.push(url.into());
    clone.push(target.clone());
    run_git(runner, pack, &clone, into)?;

    if looks_like_sha(rev) {
        // A sha is not reachable by `--branch`, and a blobless shallow clone
        // does not carry it: ask for that one commit, then move onto it.
        let mut args: Vec<String> = Vec::new();
        args.extend([
            "-C".into(),
            target.clone(),
            "fetch".into(),
            "--depth".into(),
            "1".into(),
            "--".into(),
            "origin".into(),
            rev.into(),
        ]);
        run_git(runner, pack, &args, into)?;

        let mut args: Vec<String> = Vec::new();
        args.extend([
            "-C".into(),
            target.clone(),
            "checkout".into(),
            "--detach".into(),
            "FETCH_HEAD".into(),
        ]);
        run_git(runner, pack, &args, into)?;
    }

    let mut args: Vec<String> = Vec::new();
    args.extend([
        "-C".into(),
        target,
        "sparse-checkout".into(),
        "set".into(),
        PACK_SUBDIR.into(),
    ]);
    run_git(runner, pack, &args, into)?;

    let pack_root = checkout.join(PACK_SUBDIR);
    if !pack_root.is_dir() {
        return Err(Error::Pack {
            pack: pack.to_string(),
            message: format!("`{rev}` carries no `{PACK_SUBDIR}/` directory"),
        });
    }
    Ok(pack_root)
}

/// Spawn git with the overrides in front, whatever the caller asked for.
///
/// The only way this crate reaches git. Callers pass the verb and its
/// operands and cannot omit the overrides, because they never assemble the
/// vector: a call that forgot one is what let a manifest run a command
/// through `update`, and a rule the type system keeps is worth more than one
/// a test has to remember to look for.
pub(super) fn git(runner: &dyn CommandRunner, args: &[String], cwd: &Path) -> Result<Output> {
    let mut full = overrides();
    full.extend_from_slice(args);
    runner.run("git", &full, cwd)
}

/// Run one git command, turning a failure into a pack error naming the pack.
///
/// A missing `git` is worth its own message: every superdev repo is a git
/// repo, so a user without it is in an unusual state and the generic
/// "not found" would not say what to install.
fn run_git(runner: &dyn CommandRunner, pack: &str, args: &[String], cwd: &Path) -> Result<()> {
    match git(runner, args, cwd) {
        Ok(out) if out.status == 0 => Ok(()),
        Ok(out) => Err(Error::Pack {
            pack: pack.to_string(),
            message: format!(
                "`git {}` failed (exit {}): {}",
                args.join(" "),
                out.status,
                out.stderr.replace('\n', " ").trim()
            ),
        }),
        Err(Error::Command { stderr, .. }) if stderr == "not found" => Err(Error::Pack {
            pack: pack.to_string(),
            message: "needs `git` on PATH to fetch a pinned pack — install git, \
                      or use a local path source"
                .into(),
        }),
        Err(e) => Err(Error::Pack {
            pack: pack.to_string(),
            message: e.to_string(),
        }),
    }
}

/// Whether a rev looks like a commit sha rather than a tag or branch.
///
/// Git's own rule: at least four hex characters, and abbreviations are
/// accepted. A tag of only hex characters would be misread, which no
/// convention produces.
fn looks_like_sha(rev: &str) -> bool {
    rev.len() >= 4 && rev.len() <= 40 && rev.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::FakeRunner;

    /// git runs a command for an `ext::` URL, and whether it may is the
    /// user's config to set — so superdev sets it for its own calls rather
    /// than inheriting an answer. Without this, a manifest cloned from
    /// anywhere runs whatever it likes on a machine that permits the
    /// transport. I007.
    #[test]
    fn every_git_call_refuses_the_transports_that_run_commands() {
        let runner = FakeRunner::new();
        let dir = tempfile::tempdir().unwrap();

        let _ = fetch(
            &runner,
            "acme",
            "https://example.invalid/acme.git",
            "v1",
            dir.path(),
        );

        let calls = runner.calls();
        assert!(!calls.is_empty(), "the fetch spawned nothing");
        for call in &calls {
            assert!(
                call.contains("protocol.ext.allow=never"),
                "a git call without the override: {call}"
            );
        }
    }

    /// An operand that begins with `-` is an option to git. `parse` refuses
    /// those, and `--` is the second lock on the same door: the two together
    /// are what make an argument vector's shape not worth reasoning about.
    #[test]
    fn a_url_is_passed_as_an_operand_not_an_option() {
        let runner = FakeRunner::new();
        let dir = tempfile::tempdir().unwrap();

        let _ = fetch(
            &runner,
            "acme",
            "https://example.invalid/acme.git",
            "v1",
            dir.path(),
        );

        let clone = runner
            .calls()
            .into_iter()
            .find(|c| c.contains(" clone "))
            .expect("a clone");
        let (before, after) = clone.split_once(" -- ").expect("an end-of-options marker");
        assert!(!before.contains("example.invalid"), "{clone}");
        assert!(after.contains("example.invalid"), "{clone}");
    }

    fn files(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(p, b)| ((*p).to_string(), (*b).to_string()))
            .collect()
    }

    #[test]
    fn a_digest_is_stable_and_order_independent() {
        let one = files(&[("a.md", "alpha"), ("b.md", "beta")]);
        let other = files(&[("b.md", "beta"), ("a.md", "alpha")]);
        assert_eq!(digest(&one), digest(&other));
        assert!(digest(&one).starts_with("sha256:"));
    }

    #[test]
    fn a_changed_byte_changes_the_digest() {
        assert_ne!(
            digest(&files(&[("a.md", "alpha")])),
            digest(&files(&[("a.md", "alphb")]))
        );
    }

    /// Over paths as well as contents: a digest that ignored paths would
    /// call a rename identical, and a rename is a different pack.
    #[test]
    fn a_renamed_file_changes_the_digest() {
        assert_ne!(
            digest(&files(&[("a.md", "alpha")])),
            digest(&files(&[("b.md", "alpha")]))
        );
    }

    /// Lengths are hashed with the bytes, so no two different trees can be
    /// run together into one identical stream.
    #[test]
    fn concatenation_does_not_collide() {
        assert_ne!(
            digest(&files(&[("a", "bc")])),
            digest(&files(&[("ab", "c")]))
        );
    }

    #[test]
    fn a_cache_path_carries_no_colon() {
        let path = cache_path(Path::new("/repo"), "sha256:abc");
        assert!(!path.to_string_lossy().contains(':'), "{}", path.display());
        assert!(path.starts_with("/repo/.superdev/cache/packs"));
    }

    /// The lock's digest is committed and shared. Left to the machine's
    /// `core.autocrlf`, a Windows checkout would translate every line ending,
    /// the same rev would digest differently there, and a lock written on one
    /// platform would fail verification on the other.
    #[test]
    fn every_git_call_takes_the_bytes_verbatim() {
        use crate::runner::{FakeRunner, Output};
        let fake = FakeRunner::new();
        // Every git invocation succeeds, so the whole sequence runs.
        fake.script(
            "git",
            Output {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        let dir = tempfile::tempdir().unwrap();
        // A sha rev takes the longest path: clone, fetch, checkout, sparse.
        let _ = fetch(
            &fake,
            "acme",
            "https://example.com/p.git",
            "9f2a1b3c",
            dir.path(),
        );
        let calls = fake.calls();
        assert!(!calls.is_empty(), "the fetch ran no git at all");
        for call in &calls {
            assert!(
                call.contains("core.autocrlf=false") && call.contains("core.eol=lf"),
                "a git call touching the work tree without the overrides: {call}"
            );
        }
    }

    #[test]
    fn a_sha_is_told_from_a_tag() {
        assert!(looks_like_sha("9f2a1b"));
        assert!(looks_like_sha(&"a".repeat(40)));
        assert!(!looks_like_sha("assets-v1.4.0"));
        assert!(!looks_like_sha("main"));
        assert!(!looks_like_sha("v1"), "too short to be a sha");
        assert!(!looks_like_sha(&"a".repeat(41)), "longer than a sha");
    }
}
