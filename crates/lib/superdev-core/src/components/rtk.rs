//! components/rtk.rs — the bash-output-filter capability via rtk.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::{Error, Result};
use crate::registry::RTK_PLATFORMS;

use super::item::{ManagedItem, claims, plan_items};

/// mise `[tools]` key for rtk, in the platform config files.
pub const RTK_MISE_TOOL: &str = "http:rtk";

/// The committed early-init mise config that activates the platform config
/// files. Owned by this capability until a second consumer of `auto_env`
/// appears; it sweeps with the capability.
const MISERC_PATH: &str = ".miserc.toml";
const MISERC_CONTENT: &str = "# superdev-owned: activates mise's platform config files (mise.<env>.toml).\nauto_env = true\n";

/// The platform-scoped pin files: rtk publishes no windows-arm64 artefact,
/// and an unlisted platform in a plain `.mise.toml` platforms table would
/// hard-fail `mise install` there. With `auto_env`, a windows-arm64 machine
/// loads neither file and skips the tool silently.
const UNIX_PINS_PATH: &str = "mise.unix.toml";
const WINDOWS_PINS_PATH: &str = "mise.windows-x64.toml";

/// The instruction file telling agents output is auto-filtered and how to
/// get raw output, imported by the `.agents/superdev.md` aggregator.
const INSTRUCTIONS_PATH: &str = ".agents/rtk.md";
const INSTRUCTIONS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/rtk/rtk.md"));

/// rtk's rewrite hook in the shared `.claude/settings.json`, launched
/// through mise because the pinned binary is on no PATH the hook can
/// assume. Fail-open by the harness's own rule: only exit code 2 blocks a
/// command, and rtk's hook exits 0 on every failure path.
const HOOK_POINTER: &str = "hooks.PreToolUse";
const HOOK_COMMAND: &str = "mise exec http:rtk -- rtk hook claude";
/// The registration itself: rewrite Bash commands through rtk. Built from
/// [`HOOK_COMMAND`] so the marker always matches the element it registers.
fn hook_element() -> String {
    format!(r#"{{"matcher":"Bash","hooks":[{{"type":"command","command":"{HOOK_COMMAND}"}}]}}"#)
}

/// The oldest mise whose `auto_env` setting exists. Older mise ignores
/// `.miserc.toml` and the platform files, leaving rtk pinned nowhere.
const MISE_FLOOR: (u64, u64) = (2026, 8);

/// One platform file's whole content: a `[tools]` table holding only the
/// rtk pin for `platforms`, rendered deterministically so the same version
/// always hashes the same.
fn pins_file_content(version: &str, platforms: &[(&str, &str, &str)]) -> String {
    let entries: Vec<String> = platforms
        .iter()
        .map(|(platform, url, checksum)| {
            format!("\"{platform}\" = {{ url = \"{url}\", checksum = \"{checksum}\" }}")
        })
        .collect();
    format!(
        "# superdev-owned: the rtk pin for these platforms (see .miserc.toml).\n\
         [tools]\n\
         \"{RTK_MISE_TOOL}\" = {{ version = \"{version}\", platforms = {{ {} }} }}\n",
        entries.join(", ")
    )
}

/// Everything the capability keeps in the repo.
fn items(version: &str) -> Vec<ManagedItem> {
    let (unix, windows): (Vec<_>, Vec<_>) = RTK_PLATFORMS
        .iter()
        .copied()
        .partition(|(platform, ..)| !platform.starts_with("windows-"));
    vec![
        ManagedItem::OwnedFile {
            path: MISERC_PATH.into(),
            content: MISERC_CONTENT.into(),
            reason: "activate mise platform config files".into(),
        },
        ManagedItem::OwnedFile {
            path: UNIX_PINS_PATH.into(),
            content: pins_file_content(version, &unix),
            reason: "pin rtk on unix platforms".into(),
        },
        ManagedItem::OwnedFile {
            path: WINDOWS_PINS_PATH.into(),
            content: pins_file_content(version, &windows),
            reason: "pin rtk on windows-x64".into(),
        },
        ManagedItem::OwnedFile {
            path: INSTRUCTIONS_PATH.into(),
            content: INSTRUCTIONS.into(),
            reason: "output-filter instructions".into(),
        },
        ManagedItem::JsonEntry {
            path: ".claude/settings.json".into(),
            pointer: HOOK_POINTER.into(),
            marker: Some(HOOK_COMMAND.into()),
            value_json: hook_element(),
        },
    ]
}

/// The observed mise version as (year, minor), when it can be read at all.
/// A missing or unparseable mise stays `None`: the floor is guidance for a
/// machine that has mise, not a gate on machines the rest of the plan will
/// already fail loudly on.
fn mise_version(ctx: &Ctx<'_>) -> Option<(u64, u64)> {
    let out = ctx
        .runner
        .run("mise", &["--version".into()], ctx.root)
        .ok()
        .filter(|out| out.status == 0)?;
    let version = out.stdout.split_whitespace().next()?;
    let mut parts = version.split('.');
    let year = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((year, minor))
}

/// The rtk provider.
pub struct Rtk;

impl Component for Rtk {
    fn capability(&self) -> Capability {
        Capability::BashOutputFilter
    }

    fn provider(&self) -> &'static str {
        "rtk"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let version =
            super::pin::require_registry_default(ctx, Capability::BashOutputFilter, "rtk")?;
        if let Some(observed) = mise_version(ctx)
            && observed < MISE_FLOOR
        {
            return Err(Error::Manifest {
                message: format!(
                    "bash-output-filter needs mise {}.{} or newer (its `auto_env` setting \
                     scopes the rtk pin per platform) — found {}.{}; upgrade mise, or run \
                     `superdev init --no-bash-output-filter` / delete the [bash-output-filter] \
                     table to disable",
                    MISE_FLOOR.0, MISE_FLOOR.1, observed.0, observed.1
                ),
            });
        }
        let mut actions = plan_items(ctx.root, &items(version));
        // A planned write to any mise config means the pin may be new here:
        // trust the config and install the tool by name, optional because a
        // platform rtk does not publish for has nothing to install.
        if actions
            .iter()
            .any(|a| matches!(a, Action::WriteFile { path, .. } if path.ends_with(".toml")))
        {
            actions.push(Action::Run {
                program: "mise".into(),
                args: vec!["trust".into()],
                purpose: "trust the mise config files".into(),
                undo: None,
                optional: true,
            });
            actions.push(Action::Run {
                program: "mise".into(),
                args: vec!["install".into(), RTK_MISE_TOOL.into()],
                purpose: "install the pinned rtk".into(),
                undo: None,
                optional: true,
            });
        }
        Ok(actions)
    }

    fn owned(&self, _ctx: &Ctx<'_>) -> Vec<Claim> {
        let version = crate::registry::entry_for(Capability::BashOutputFilter, "rtk")
            .and_then(|e| e.version)
            .expect("registry pins rtk")
            .version;
        claims(&items(version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Ctx;
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::{FakeRunner, Output};

    fn ctx<'a>(
        root: &'a std::path::Path,
        runner: &'a FakeRunner,
        manifest: &'a Manifest,
        lock: &'a Lock,
    ) -> Ctx<'a> {
        Ctx {
            root,
            runner,
            manifest,
            lock,
            content: crate::content::test_snapshot(),
        }
    }

    fn mise_version_output(version: &str) -> Output {
        Output {
            status: 0,
            stdout: format!("{version} linux-arm64\n"),
            stderr: String::new(),
        }
    }

    #[test]
    fn a_fresh_repo_plans_the_files_the_hook_and_the_install() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        fake.script("mise --version", mise_version_output("2026.8.8"));
        let actions = Rtk.plan(&ctx(dir.path(), &fake, &manifest, &lock)).unwrap();
        let descs: Vec<String> = actions.iter().map(Action::describe).collect();
        for expected in [
            ".miserc.toml",
            "mise.unix.toml",
            "mise.windows-x64.toml",
            ".agents/rtk.md",
            "hooks.PreToolUse",
            "mise trust",
            "mise install http:rtk",
        ] {
            assert!(
                descs.iter().any(|d| d.contains(expected)),
                "{expected} missing: {descs:?}"
            );
        }
        // The pin files split rtk's published platforms at the os boundary.
        let unix = actions
            .iter()
            .find_map(|a| match a {
                Action::WriteFile { path, content, .. } if path == "mise.unix.toml" => {
                    Some(content.clone())
                }
                _ => None,
            })
            .unwrap();
        assert!(
            unix.contains("linux-arm64") && unix.contains("macos-x64"),
            "{unix}"
        );
        assert!(!unix.contains("windows"), "{unix}");
        let windows = actions
            .iter()
            .find_map(|a| match a {
                Action::WriteFile { path, content, .. } if path == "mise.windows-x64.toml" => {
                    Some(content.clone())
                }
                _ => None,
            })
            .unwrap();
        assert!(windows.contains("windows-x64"), "{windows}");
        assert!(!windows.contains("linux"), "{windows}");
    }

    #[test]
    fn a_converged_repo_plans_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        fake.script("mise --version", mise_version_output("2026.8.8"));
        let version = crate::registry::entry_for(Capability::BashOutputFilter, "rtk")
            .unwrap()
            .version
            .unwrap()
            .version;
        for item in items(version) {
            if let ManagedItem::OwnedFile { path, content, .. } = item {
                let target = dir.path().join(&path);
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::write(target, content).unwrap();
            }
        }
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(
            dir.path().join(".claude/settings.json"),
            format!(r#"{{"hooks":{{"PreToolUse":[{}]}}}}"#, hook_element()),
        )
        .unwrap();
        let actions = Rtk.plan(&ctx(dir.path(), &fake, &manifest, &lock)).unwrap();
        assert!(actions.is_empty(), "{actions:?}");
    }

    #[test]
    fn an_old_mise_gets_the_guided_floor_error() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        fake.script("mise --version", mise_version_output("2026.5.11"));
        let err = Rtk
            .plan(&ctx(dir.path(), &fake, &manifest, &lock))
            .unwrap_err()
            .to_string();
        assert!(err.contains("needs mise 2026.8 or newer"), "{err}");
        assert!(err.contains("found 2026.5"), "{err}");
        assert!(err.contains("--no-bash-output-filter"), "{err}");
    }

    #[test]
    fn an_unreadable_mise_version_is_not_a_gate() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        // mise missing entirely: the floor stays silent and the plan stands.
        let fake = FakeRunner::new();
        fake.missing("mise");
        assert!(Rtk.plan(&ctx(dir.path(), &fake, &manifest, &lock)).is_ok());
        // Garbage output: same.
        let fake = FakeRunner::new();
        fake.script(
            "mise --version",
            Output {
                status: 0,
                stdout: "not a version".into(),
                stderr: String::new(),
            },
        );
        assert!(Rtk.plan(&ctx(dir.path(), &fake, &manifest, &lock)).is_ok());
    }

    #[test]
    fn a_foreign_version_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest
            .config_of_mut(Capability::BashOutputFilter, "rtk")
            .unwrap()
            .version = Some("9.9.9".into());
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let err = Rtk
            .plan(&ctx(dir.path(), &fake, &manifest, &lock))
            .unwrap_err()
            .to_string();
        assert!(err.contains("must match the registry default"), "{err}");
        assert!(err.contains("superdev update bash-output-filter"), "{err}");
    }

    #[test]
    fn owned_claims_cover_every_item() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let keys: Vec<String> = Rtk
            .owned(&ctx(dir.path(), &fake, &manifest, &lock))
            .iter()
            .map(Claim::lock_key)
            .collect();
        assert_eq!(
            keys,
            vec![
                ".miserc.toml".to_string(),
                "mise.unix.toml".to_string(),
                "mise.windows-x64.toml".to_string(),
                ".agents/rtk.md".to_string(),
                format!(".claude/settings.json:hooks.PreToolUse[{HOOK_COMMAND}]"),
            ]
        );
    }
}
