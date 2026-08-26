//! pack/pin.rs — where the pack pin moves on `update`.
//!
//! The default pin is compiled into the binary, so left alone it can never
//! advance past the binary that carries it and a content release would reach
//! only repos whose owner hand-edits the manifest. `update` is the one verb
//! that goes and looks (ADR-009).

use std::path::Path;

use crate::manifest::{Manifest, PackEntry};
use crate::runner::CommandRunner;

use super::source::{DEFAULT_PACK, PACK_TAG_PREFIX, PackSource};

/// A pack release, as `assets-vX.Y.Z` spells it. Ordered by the tuple, which
/// is what makes "newest" mean the version rather than the tag string —
/// `assets-v0.10.0` sorts after `assets-v0.9.0`, which alphabetically it
/// would not.
type Release = (u64, u64, u64);

/// What the default source said when asked for its newest release.
enum Newest {
    /// It answered: the newest release it carries, or none tagged yet.
    Answered(Option<Release>),
    /// It could not be asked — no network, no `git`, no access.
    Unreachable,
}

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
        let Some(current) = release(&rev) else {
            lines.push(format!(
                "packs: {named} stays at {rev} — not a release tag, so nothing says what is newer"
            ));
            continue;
        };
        let newest = asked.get_or_insert_with(|| newest_release(runner, root, &source));
        let (target, unchecked) = choose(current, floor, newest);
        if target > current {
            let moved = tag(target);
            entry.rev = Some(moved.clone());
            lines.push(match unchecked {
                None => format!("packs: {named} moved to {moved}"),
                Some(why) => format!(
                    "packs: {named} moved to {moved} — {why}, so no further than this binary carries"
                ),
            });
        } else {
            lines.push(match unchecked {
                None => format!("packs: {named} is at the newest release {rev}"),
                Some(why) => format!("packs: {named} stays at {rev} — {why}"),
            });
        }
    }
    lines
}

/// Where a pin should end up, and what to say about how it got there.
///
/// Split out from the loop because the floor is whatever this binary happens
/// to carry, and the one case that cannot be reached from a released binary —
/// no floor at all, which is what a candidate build has — is the one most
/// worth holding.
fn choose(current: Release, floor: Option<Release>, newest: &Newest) -> (Release, Option<String>) {
    // Never below what the binary carries, and never below where the pin
    // already is: going to look is meant to bring content forward.
    let base = floor.map_or(current, |floor| current.max(floor));
    match newest {
        // A pin ahead of every release the source carries is a hand-edit or a
        // typo, and the `sync` that follows is about to fail on it. It is
        // reported as what it is rather than as the newest release, which
        // would say the query had confirmed something it did not.
        Newest::Answered(Some(remote)) if *remote < current => (
            base,
            Some(format!("the source's newest release is {}", tag(*remote))),
        ),
        Newest::Answered(Some(remote)) => (base.max(*remote), None),
        Newest::Answered(None) => (base, Some("it carries no release tag".to_string())),
        Newest::Unreachable => (base, Some("could not reach it".to_string())),
    }
}

/// Ask a source for the newest pack release it carries.
///
/// `ls-remote` and not a clone: the answer is one line of refs, and the pin
/// it produces is what the resolver then fetches and verifies as usual.
fn newest_release(runner: &dyn CommandRunner, root: &Path, source: &PackSource) -> Newest {
    let args = vec![
        "ls-remote".to_string(),
        "--tags".to_string(),
        "--refs".to_string(),
        source.clone_url(),
        format!("refs/tags/{PACK_TAG_PREFIX}*"),
    ];
    match runner.run("git", &args, root) {
        Ok(out) if out.status == 0 => Newest::Answered(
            out.stdout
                .lines()
                .filter_map(|line| line.rsplit('/').next())
                .filter_map(release)
                .max(),
        ),
        // Every failure is the same failure here: superdev could not find
        // out, so the pin goes no further than what it already has.
        _ => Newest::Unreachable,
    }
}

/// The release a tag names, or `None` for anything that is not one.
///
/// Exactly three numeric parts: a pre-release like `assets-v1.0.0-rc1` is
/// not a release `update` may move a pin to, and neither is a branch.
fn release(tag: &str) -> Option<Release> {
    let version = tag.trim().strip_prefix(PACK_TAG_PREFIX)?;
    let [major, minor, patch] = version.split('.').collect::<Vec<_>>()[..] else {
        return None;
    };
    let number = |part: &str| part.parse::<u64>().ok().filter(|_| !part.starts_with('+'));
    Some((number(major)?, number(minor)?, number(patch)?))
}

/// The tag a release is written as.
fn tag((major, minor, patch): Release) -> String {
    format!("{PACK_TAG_PREFIX}{major}.{minor}.{patch}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{FakeRunner, Output};

    /// A manifest with the given entries and nothing else that matters here.
    fn manifest(packs: &[(&str, Option<&str>)]) -> Manifest {
        let mut manifest = Manifest::default_for("0.0.0", &[]);
        manifest.packs = packs
            .iter()
            .map(|(source, rev)| PackEntry {
                source: (*source).to_string(),
                rev: rev.map(Into::into),
            })
            .collect();
        manifest
    }

    /// A runner whose `ls-remote` answers with these tags.
    fn source_carrying(tags: &[&str]) -> FakeRunner {
        let runner = FakeRunner::new();
        let refs: String = tags
            .iter()
            .map(|tag| format!("0123456789abcdef\trefs/tags/{tag}\n"))
            .collect();
        runner.script(
            "git ls-remote",
            Output {
                status: 0,
                stdout: refs,
                stderr: String::new(),
            },
        );
        runner
    }

    fn revs(manifest: &Manifest) -> Vec<Option<&str>> {
        manifest.packs.iter().map(|p| p.rev.as_deref()).collect()
    }

    /// Test plan case 16: the pin moves to the source's newest release, ahead
    /// of what this binary embeds. Without this the whole feature is limited
    /// to repos whose owner edits the manifest by hand.
    #[test]
    fn a_default_source_pin_moves_to_the_newest_release() {
        let runner = source_carrying(&["assets-v0.1.0", "assets-v0.9.0", "assets-v0.10.0"]);
        let mut manifest = manifest(&[(DEFAULT_PACK.source, Some(DEFAULT_PACK.rev))]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [Some("assets-v0.10.0")]);
        assert!(
            lines.iter().any(|l| l.contains("moved to assets-v0.10.0")),
            "{lines:?}"
        );
        // Newest by version, not by tag string: `0.9.0` sorts after `0.10.0`
        // alphabetically, and picking it would move the pin backwards.
        assert!(
            runner.calls().iter().any(|c| c.contains("ls-remote")),
            "{:?}",
            runner.calls()
        );
    }

    /// Test plan case 17: a pin naming another source is reported and left
    /// alone. Moving it would pull content nobody reviewed on a command
    /// typed for superdev's own updates.
    #[test]
    fn a_third_party_pin_is_reported_and_left_alone() {
        let runner = source_carrying(&["assets-v9.9.9"]);
        let mut manifest = manifest(&[("github:acme/packs", Some("assets-v0.1.0"))]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [Some("assets-v0.1.0")]);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("github:acme/packs") && l.contains("stays at assets-v0.1.0")),
            "{lines:?}"
        );
        assert!(
            runner.calls().is_empty(),
            "an update naming no source of superdev's makes no request: {:?}",
            runner.calls()
        );
    }

    /// Test plan case 20: with the source unreachable the pin moves no
    /// further than what this binary carries, and says it could not check.
    /// Erroring instead would make `update` unusable offline.
    #[test]
    fn an_unreachable_source_stops_at_what_the_binary_carries() {
        let runner = FakeRunner::new();
        runner.missing("git");
        let mut manifest = manifest(&[(DEFAULT_PACK.source, Some("assets-v0.0.1"))]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [Some(DEFAULT_PACK.rev)]);
        let line = lines.join("\n");
        assert!(line.contains("could not reach it"), "{line}");
        assert!(line.contains(DEFAULT_PACK.rev), "{line}");
    }

    /// A failed `ls-remote` is the same as no `git` at all: a private repo,
    /// an expired credential and a blocked network all leave superdev
    /// without an answer, and none of them is a reason to move the pin.
    #[test]
    fn a_failed_query_is_treated_as_unreachable() {
        let runner = FakeRunner::new();
        runner.script(
            "git ls-remote",
            Output {
                status: 128,
                stdout: String::new(),
                stderr: "fatal: could not read Username".into(),
            },
        );
        let mut manifest = manifest(&[(DEFAULT_PACK.source, Some(DEFAULT_PACK.rev))]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [Some(DEFAULT_PACK.rev)]);
        assert!(lines.join("\n").contains("could not reach it"), "{lines:?}");
    }

    /// Test plan case 22: a manifest an earlier binary wrote gains the
    /// default entry, which is how a pre-pack repo starts tracking content
    /// releases at all.
    #[test]
    fn a_manifest_without_an_entry_gains_the_default_one() {
        let runner = source_carrying(&["assets-v0.4.0"]);
        let mut manifest = manifest(&[]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(manifest.packs.len(), 1);
        assert_eq!(manifest.packs[0].source, DEFAULT_PACK.source);
        // Written at the binary's own rev, then moved by the same run: the
        // entry appears already current, not a release behind.
        assert_eq!(revs(&manifest), [Some("assets-v0.4.0")]);
        assert!(
            lines.iter().any(|l| l.contains("wrote the default entry")),
            "{lines:?}"
        );
    }

    /// Going to look brings content forward. A source that has lost its
    /// newest tag, or a pin a user put ahead of the releases, must not have
    /// content quietly taken away by a routine `update`.
    #[test]
    fn a_pin_never_moves_backwards() {
        let runner = source_carrying(&["assets-v0.1.0"]);
        let mut manifest = manifest(&[(DEFAULT_PACK.source, Some("assets-v9.9.9"))]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [Some("assets-v9.9.9")]);
        // Not "is at the newest release": the source does not carry this rev
        // at all, and the `sync` that follows is about to fail on it. Saying
        // it is current would send the reader looking anywhere but here.
        assert_eq!(
            lines,
            ["packs: github:six5536/superdev stays at assets-v9.9.9 \
              — the source's newest release is assets-v0.1.0"]
        );
    }

    /// A pin the source does carry, and carries as its newest, is the one
    /// case that may say so.
    #[test]
    fn a_pin_at_the_sources_newest_is_reported_as_current() {
        let runner = source_carrying(&["assets-v0.1.0"]);
        let mut manifest = manifest(&[(DEFAULT_PACK.source, Some(DEFAULT_PACK.rev))]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(
            lines,
            ["packs: github:six5536/superdev is at the newest release assets-v0.1.0"]
        );
    }

    /// A pin on a branch or a sha is a deliberate choice — someone testing
    /// against unreleased content — and nothing here knows what is newer
    /// than it. Report it and leave it.
    #[test]
    fn a_pin_that_is_not_a_release_tag_is_left_alone() {
        let runner = source_carrying(&["assets-v9.9.9"]);
        let mut manifest = manifest(&[
            (DEFAULT_PACK.source, Some("main")),
            (
                "https://github.com/six5536/superdev.git",
                Some("assets-v1.0.0-rc1"),
            ),
        ]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [Some("main"), Some("assets-v1.0.0-rc1")]);
        assert_eq!(
            lines
                .iter()
                .filter(|l| l.contains("not a release tag"))
                .count(),
            2,
            "{lines:?}"
        );
    }

    /// A path source has no revision to pin, so there is nothing to move and
    /// nothing to say: it is read afresh on every run by design.
    #[test]
    fn a_path_source_has_no_pin_to_move() {
        let runner = FakeRunner::new();
        let mut manifest = manifest(&[("./packs/acme", None)]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [None]);
        assert!(lines.is_empty(), "{lines:?}");
        assert!(runner.calls().is_empty(), "{:?}", runner.calls());
    }

    /// A source with no release tagged yet answers, and the answer is "none":
    /// distinct from unreachable, and reported as what it is.
    #[test]
    fn a_source_carrying_no_release_says_so() {
        let runner = source_carrying(&[]);
        let mut manifest = manifest(&[(DEFAULT_PACK.source, Some(DEFAULT_PACK.rev))]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [Some(DEFAULT_PACK.rev)]);
        assert!(
            lines.join("\n").contains("carries no release tag"),
            "{lines:?}"
        );
    }

    /// The query is built from the source, not from a hardcoded URL, and
    /// carries the tag pattern so a repository of a thousand tags answers
    /// with the handful that are pack releases.
    #[test]
    fn the_query_asks_the_source_for_its_release_tags() {
        let runner = source_carrying(&["assets-v0.2.0"]);
        let mut manifest = manifest(&[(DEFAULT_PACK.source, Some(DEFAULT_PACK.rev))]);

        update_pins(&runner, Path::new("."), &mut manifest);

        let calls = runner.calls();
        assert_eq!(calls.len(), 1, "asked once: {calls:?}");
        assert_eq!(
            calls[0],
            "git ls-remote --tags --refs https://github.com/six5536/superdev refs/tags/assets-v*"
        );
    }

    /// A candidate's content tag is on the source like any other, and must
    /// not be what a stable pin moves to: the release script cuts one at every
    /// binary rc, so this is the ordinary state of the repository, not an edge.
    #[test]
    fn a_candidate_tag_on_the_source_is_not_a_release_to_move_to() {
        let runner = source_carrying(&["assets-v0.1.0", "assets-v0.2.0-rc.1"]);
        let mut manifest = manifest(&[(DEFAULT_PACK.source, Some("assets-v0.1.0"))]);

        let lines = update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(revs(&manifest), [Some("assets-v0.1.0")]);
        assert!(
            lines.join("\n").contains("is at the newest release"),
            "{lines:?}"
        );
    }

    /// A candidate binary pins `assets-vX.Y.Z-rc.N`, which is no release, so
    /// this binary contributes no floor. Nothing may panic on that, and a pin
    /// must not be dragged up to a version the binary cannot vouch for.
    #[test]
    fn a_binary_carrying_no_release_is_no_floor() {
        let current = (0, 1, 0);

        assert_eq!(
            choose(current, None, &Newest::Unreachable),
            (current, Some("could not reach it".to_string()))
        );
        assert_eq!(
            choose(current, None, &Newest::Answered(Some((0, 4, 0)))),
            ((0, 4, 0), None),
            "the source still moves it; only the binary's own floor is absent"
        );
    }

    /// Two spellings of the default source are one source: the query goes out
    /// once, and both entries land on the same release.
    #[test]
    fn the_source_is_asked_once_however_many_entries_spell_it() {
        let runner = source_carrying(&["assets-v0.3.0"]);
        let mut manifest = manifest(&[
            (DEFAULT_PACK.source, Some(DEFAULT_PACK.rev)),
            ("git@github.com:six5536/superdev.git", Some("assets-v0.0.1")),
        ]);

        update_pins(&runner, Path::new("."), &mut manifest);

        assert_eq!(
            revs(&manifest),
            [Some("assets-v0.3.0"), Some("assets-v0.3.0")]
        );
        assert_eq!(runner.calls().len(), 1, "{:?}", runner.calls());
    }
}
