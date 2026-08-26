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
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::runner::{CommandRunner, Output, RunOptions};

use super::manifest::link_refusal;
use super::source::SUPPORTED_SCHEMES;

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

/// The transports refused by name.
///
/// Named and not left to the blanket, because git resolves
/// `protocol.<name>.allow` ahead of `protocol.allow` whatever their sources:
/// a machine whose own config says `protocol.ext.allow = always` outranks a
/// blanket on superdev's command line, and only a named refusal on that same
/// command line outranks the machine. `ext` runs a program as its connection;
/// `git` and `http` authenticate nothing, so anyone on the path can answer.
/// ADR-012.
const REFUSED_TRANSPORTS: &[&str] = &["ext", "git", "http"];

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
    let mut args = vec![
        "-c".to_string(),
        "core.autocrlf=false".into(),
        "-c".into(),
        "core.eol=lf".into(),
        // Everything superdev did not admit, refused. A manifest is a file
        // that arrives with a repository, so a source is not superdev's to
        // trust; set on superdev's own calls rather than inherited, so a
        // machine configured to permit a transport does not permit it to a
        // cloned manifest. I007.
        "-c".into(),
        "protocol.allow=never".into(),
    ];
    // Read off the same constant `PackSource::parse` refuses against, so the
    // two halves of the allowlist cannot drift apart.
    for scheme in SUPPORTED_SCHEMES {
        args.push("-c".into());
        args.push(format!("protocol.{scheme}.allow=always"));
    }
    // The blanket alone does not hold against a machine that has named one of
    // these, and `insteadOf` can rewrite a source `parse` approved into one of
    // them after superdev has handed it over. This is the half that sees that.
    for refused in REFUSED_TRANSPORTS {
        args.push("-c".into());
        args.push(format!("protocol.{refused}.allow=never"));
    }
    args
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

    // Before anything under the pack is read or digested, and before the
    // directory check below, so what stops the run is the entry rather than
    // whatever shape it left on disk.
    refuse_linked_entries(runner, pack, &checkout)?;

    let pack_root = checkout.join(PACK_SUBDIR);
    if !pack_root.is_dir() {
        return Err(Error::Pack {
            pack: pack.to_string(),
            message: format!("`{rev}` carries no `{PACK_SUBDIR}/` directory"),
        });
    }
    Ok(pack_root)
}

/// A checked-out entry git records as a symlink.
const INDEX_SYMLINK: &str = "120000";

/// A checked-out entry git records as a gitlink — a submodule.
const INDEX_GITLINK: &str = "160000";

/// Refuse a fetched pack whose index says it carries a link or a submodule.
///
/// Git is asked rather than the filesystem because only git knows. On Windows
/// without `core.symlinks` a link is checked out as a plain file holding the
/// target's path, which `symlink_metadata` cannot tell from content — so the
/// bytes would enter the digest there and not here, and one rev would verify
/// on Linux and fail on Windows naming nothing a reader could act on. The
/// index says `120000` on both. ADR-014.
///
/// A submodule is refused with it: a shallow sparse clone leaves its
/// directory empty, so the pack would ship an empty item and say nothing —
/// the same failure in a different costume.
///
/// The filesystem check in `resolve` stays, and is not redundant: this answer
/// is taken once against the checkout, and the cache read on every later run
/// has no index of its own.
fn refuse_linked_entries(runner: &dyn CommandRunner, pack: &str, checkout: &Path) -> Result<()> {
    let args: Vec<String> = vec![
        "-C".into(),
        checkout.to_string_lossy().into_owned(),
        "ls-files".into(),
        "--stage".into(),
        // NUL-separated, so a path git would otherwise quote — a newline or a
        // non-ASCII byte in a filename — arrives whole and unescaped.
        "-z".into(),
        "--".into(),
        PACK_SUBDIR.into(),
    ];
    let out = git(runner, &args, checkout)?;
    if out.status != 0 {
        return Err(Error::Pack {
            pack: pack.to_string(),
            message: format!(
                "could not read `{PACK_SUBDIR}/`'s entries from the checkout: {}",
                out.stderr.trim()
            ),
        });
    }
    for entry in out.stdout.split('\0').filter(|entry| !entry.is_empty()) {
        // `<mode> <object> <stage>\t<path>`. The path holds no tab: git
        // records one as `\t` in a quoted path, and `-z` paths are never
        // quoted, so a real tab cannot reach here to be split on.
        let Some((meta, path)) = entry.split_once('\t') else {
            continue;
        };
        match meta.split_whitespace().next() {
            Some(INDEX_SYMLINK) => return Err(link_refusal(pack, path)),
            Some(INDEX_GITLINK) => {
                return Err(Error::Pack {
                    pack: pack.to_string(),
                    message: format!(
                        "{path} is a submodule — a shallow sparse clone leaves one \
                         empty, so the pack would ship an item with nothing in it"
                    ),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

/// Spawn git with the overrides in front, whatever the caller asked for.
///
/// The only way this crate reaches git. Callers pass the verb and its
/// operands and cannot omit the overrides, because they never assemble the
/// vector: a call that forgot one is what let a manifest run a command
/// through `update`, and a rule the type system keeps is worth more than one
/// a test has to remember to look for.
pub(super) fn git(runner: &dyn CommandRunner, args: &[String], cwd: &Path) -> Result<Output> {
    git_within(runner, args, cwd, None)
}

/// The same, giving git `timeout` to answer in.
///
/// `None` waits. A clone happens because the user pinned a pack and asked for
/// it, and a repository on a slow link is a legitimately long wait superdev
/// has no business ending; only what superdev does on its own initiative is
/// bounded. ADR-015.
pub(super) fn git_within(
    runner: &dyn CommandRunner,
    args: &[String],
    cwd: &Path,
    timeout: Option<Duration>,
) -> Result<Output> {
    let mut full = overrides();
    full.extend_from_slice(args);
    runner.run_with(
        "git",
        &full,
        cwd,
        &RunOptions {
            timeout,
            env: environment(),
        },
    )
}

/// The environment every git call carries.
///
/// `GIT_TERMINAL_PROMPT=0` on all of them, not only the ones that reach the
/// network: a prompt superdev cannot answer is a stall whatever produced it.
/// Stdin is already null, which makes most prompts fail on EOF — but git asks
/// the terminal directly where it can, and this is what closes that.
fn environment() -> Vec<(String, String)> {
    vec![("GIT_TERMINAL_PROMPT".to_string(), "0".to_string())]
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

    /// A git repository carrying a pack, for the questions only a real index
    /// can answer. Returns its directory; `git` runs inside it.
    #[cfg(unix)]
    fn fixture_repo(dir: &Path) -> impl Fn(&[&str]) -> String + use<'_> {
        let git = move |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(dir)
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .output()
                .expect("git runs");
            assert!(out.status.success(), "git {args:?}: {out:?}");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        fs::create_dir_all(dir.join("pack/knowledge/skills/honest")).unwrap();
        fs::write(
            dir.join("pack/pack.toml"),
            "format = 1\nname = \"fixture\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        fs::write(
            dir.join("pack/knowledge/skills/honest/SKILL.md"),
            "# honest\n",
        )
        .unwrap();
        git(&["init", "-q", "-b", "main"]);
        git(&["config", "user.email", "fixture@example.com"]);
        git(&["config", "user.name", "fixture"]);
        git(&["config", "commit.gpgsign", "false"]);
        git
    }

    /// The clone is the user's own request — they pinned the pack and asked
    /// for it — so nothing here ends it early. Only what superdev does on its
    /// own initiative is bounded. ADR-015.
    #[test]
    fn a_fetch_carries_no_deadline_and_never_prompts() {
        let runner = FakeRunner::new();
        let dir = tempfile::tempdir().unwrap();

        let _ = fetch(
            &runner,
            "acme",
            "https://example.invalid/acme.git",
            "v1",
            dir.path(),
        );

        let options = runner.options();
        assert!(!options.is_empty(), "the fetch spawned nothing");
        for (call, opts) in runner.calls().iter().zip(&options) {
            assert!(
                opts.timeout.is_none(),
                "a fetch call carries a deadline: {call}"
            );
            assert!(
                opts.env
                    .contains(&("GIT_TERMINAL_PROMPT".to_string(), "0".to_string())),
                "a git call that could stop for a prompt: {call}"
            );
        }
    }

    /// A pack with neither a link nor a submodule passes, so the check costs
    /// a clean pack one git call and nothing else.
    #[cfg(unix)]
    #[test]
    fn a_pack_the_index_calls_ordinary_is_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let git = fixture_repo(dir.path());
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "clean"]);

        refuse_linked_entries(&crate::runner::SystemRunner, "acme", dir.path())
            .expect("a pack with no link resolves");
    }

    /// Git's index decides, not the filesystem — and here they agree.
    #[cfg(unix)]
    #[test]
    fn a_symlink_the_index_records_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let git = fixture_repo(dir.path());
        std::os::unix::fs::symlink(
            "../honest/SKILL.md",
            dir.path().join("pack/knowledge/skills/honest/LINK.md"),
        )
        .unwrap();
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "linked"]);

        let err = refuse_linked_entries(&crate::runner::SystemRunner, "acme", dir.path())
            .expect_err("the index says 120000");
        let message = err.to_string();
        assert!(message.contains("is a symlink"), "{message}");
        assert!(
            message.contains("pack/knowledge/skills/honest/LINK.md"),
            "the refusal does not name the path: {message}"
        );
    }

    /// The case the filesystem cannot see, and the reason this asks git at
    /// all. Without `core.symlinks` git writes the entry as a plain file
    /// holding the target's path — which is what a Windows checkout does —
    /// so `symlink_metadata` sees ordinary content and the bytes enter the
    /// digest. A lock written on Linux would then fail on Windows naming
    /// nothing a reader could act on. ADR-014.
    ///
    /// Reproduced on Linux by setting `core.symlinks=false` in the
    /// repository's *own* config and forcing a re-checkout: passing
    /// `-c core.symlinks=false` to `clone` does not work, because clone
    /// probes the filesystem and writes its own value.
    #[cfg(unix)]
    #[test]
    fn a_link_checked_out_as_a_plain_file_is_still_refused() {
        let dir = tempfile::tempdir().unwrap();
        let git = fixture_repo(dir.path());
        let link = dir.path().join("pack/knowledge/skills/honest/LINK.md");
        std::os::unix::fs::symlink("../honest/SKILL.md", &link).unwrap();
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "linked"]);

        git(&["config", "core.symlinks", "false"]);
        fs::remove_file(&link).unwrap();
        git(&["checkout", "--", "pack"]);

        // The premise: the working tree now holds content, not a link.
        let meta = fs::symlink_metadata(&link).expect("the entry exists");
        assert!(
            !meta.file_type().is_symlink(),
            "the checkout still made a real link — the premise did not hold"
        );
        assert_eq!(
            fs::read_to_string(&link).unwrap(),
            "../honest/SKILL.md",
            "the plain file holds the target's path"
        );

        let err = refuse_linked_entries(&crate::runner::SystemRunner, "acme", dir.path())
            .expect_err("the index still says 120000");
        assert!(err.to_string().contains("is a symlink"), "{err}");
    }

    /// A submodule is the same failure in a different costume: a shallow
    /// sparse clone leaves the directory empty, so the pack would ship an
    /// empty item and say nothing. ADR-014.
    #[cfg(unix)]
    #[test]
    fn a_submodule_under_the_pack_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let git = fixture_repo(dir.path());
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "clean"]);
        let sha = git(&["rev-parse", "HEAD"]);
        // The index entry a submodule is, without the machinery around it.
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{sha},pack/vendor"),
        ]);

        let err = refuse_linked_entries(&crate::runner::SystemRunner, "acme", dir.path())
            .expect_err("a gitlink under the pack");
        let message = err.to_string();
        assert!(message.contains("submodule"), "{message}");
        assert!(message.contains("pack/vendor"), "{message}");
    }

    /// git runs a command for an `ext::` URL, and whether it may is the
    /// user's config to set — so superdev sets it for its own calls rather
    /// than inheriting an answer. Without this, a manifest cloned from
    /// anywhere runs whatever it likes on a machine that permits the
    /// transport. I007.
    ///
    /// Every setting is asserted, not just the one that started this: the
    /// blanket alone does not hold against a machine that has named a
    /// transport, so each named refusal is load-bearing on its own. ADR-012.
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
            for setting in [
                "protocol.allow=never",
                "protocol.https.allow=always",
                "protocol.ssh.allow=always",
                "protocol.file.allow=always",
                "protocol.ext.allow=never",
                "protocol.git.allow=never",
                "protocol.http.allow=never",
            ] {
                assert!(
                    call.contains(setting),
                    "a git call without `{setting}`: {call}"
                );
            }
        }
    }

    /// The overrides admit exactly what `parse` admits, because they are read
    /// off the same constant. A scheme added to one and not the other would be
    /// refused by half of the pair and accepted by the other.
    #[test]
    fn the_overrides_admit_exactly_the_supported_schemes() {
        let args = overrides().join(" ");
        for scheme in SUPPORTED_SCHEMES {
            assert!(
                args.contains(&format!("protocol.{scheme}.allow=always")),
                "`{scheme}` is supported but not admitted: {args}"
            );
        }
        for refused in REFUSED_TRANSPORTS {
            assert!(
                !SUPPORTED_SCHEMES.contains(refused),
                "`{refused}` is both refused and supported"
            );
        }
    }

    /// The case that proves the overrides are not decoration.
    ///
    /// `url.<base>.insteadOf` rewrites a URL *after* superdev has handed it
    /// over, so a plain `https://` source that `parse` approved becomes an
    /// `ext::` command under a config that asks for it — which `parse` cannot
    /// see. Only a *named* refusal stops it: git resolves
    /// `protocol.<name>.allow` ahead of `protocol.allow` whatever their
    /// sources, so the machine's `protocol.ext.allow = always` outranks
    /// superdev's blanket and is outranked in turn by superdev's named line.
    ///
    /// Spawned directly rather than through `CommandRunner`, because the
    /// hostile config reaches git as an environment variable and `run` passes
    /// no environment. The argument vector is the product's own. Unix only:
    /// the rewritten URL runs its command through `sh`.
    #[cfg(unix)]
    #[test]
    fn a_rewritten_url_still_runs_no_command() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("marker");
        let config = dir.path().join("hostile.gitconfig");
        std::fs::write(
            &config,
            format!(
                "[protocol \"ext\"]\n\tallow = always\n\
                 [url \"ext::touch {} \"]\n\tinsteadOf = https://superdev.invalid/\n",
                marker.display()
            ),
        )
        .unwrap();

        let mut args = overrides();
        args.extend([
            "ls-remote".to_string(),
            "--".into(),
            "https://superdev.invalid/pack".into(),
        ]);
        let out = std::process::Command::new("git")
            .args(&args)
            .env("GIT_CONFIG_GLOBAL", &config)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .current_dir(dir.path())
            .output()
            .expect("git runs");

        assert!(
            !marker.exists(),
            "the rewritten URL ran its command: {}",
            String::from_utf8_lossy(&out.stderr)
        );
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
