//! components/codegraph.rs — the code-index capability via codegraph.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Component, Ctx};
use crate::error::{Error, Result};
use crate::registry::{self, CODEGRAPH_PLATFORMS};

/// mise `[tools]` key for codegraph.
pub const CODEGRAPH_MISE_TOOL: &str = "http:codegraph";
/// Directory `codegraph init` creates.
pub const CODEGRAPH_INDEX_DIR: &str = ".codegraph";

/// The codegraph provider.
pub struct Codegraph;

/// The `.mise.toml` value for the pinned release: one checksummed bundle per
/// platform, in registry order, so the same version always renders the same
/// fragment.
fn pin_value(version: &str) -> String {
    let platforms: Vec<String> = CODEGRAPH_PLATFORMS
        .iter()
        .map(|(platform, url, checksum)| {
            format!("\"{platform}\" = {{ url = \"{url}\", checksum = \"{checksum}\" }}")
        })
        .collect();
    format!(
        "{{ version = \"{version}\", platforms = {{ {} }}, strip_components = 1 }}",
        platforms.join(", ")
    )
}

impl Component for Codegraph {
    fn capability(&self) -> Capability {
        Capability::CodeIndex
    }

    fn provider(&self) -> &'static str {
        "codegraph"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let config = ctx
            .config(Capability::CodeIndex)
            .expect("planned only when enabled");
        let version = config.version.clone().expect("registry pins codegraph");
        let default = registry::entries()
            .iter()
            .find(|e| e.capability == Capability::CodeIndex)
            .and_then(|e| e.version)
            .expect("registry pins codegraph");
        if version != default {
            return Err(Error::Manifest {
                message: format!(
                    "code-index version must match the registry default {default} — the pinned checksum is the provenance"
                ),
            });
        }
        let mut actions = Vec::new();
        let value = pin_value(&version);
        let current = match std::fs::read_to_string(ctx.root.join(".mise.toml")) {
            Ok(s) => super::mise::current_pin(&s, CODEGRAPH_MISE_TOOL)?,
            Err(_) => None,
        };
        // Round-trip the desired value so layout differences never read as
        // drift: the pin is an inline table, which toml_edit renders its way.
        let desired = super::mise::set_pin("", CODEGRAPH_MISE_TOOL, &value)
            .and_then(|s| super::mise::current_pin(&s, CODEGRAPH_MISE_TOOL))?
            .expect("pin just set");
        if current.as_deref() != Some(desired.as_str()) {
            actions.push(Action::SetMisePin {
                tool: CODEGRAPH_MISE_TOOL.into(),
                value_toml: value,
            });
        }
        if !ctx.root.join(CODEGRAPH_INDEX_DIR).is_dir() {
            // Through mise: the tool is pinned in this repo's `.mise.toml`,
            // and mise install puts it on no PATH the running process can see.
            // Naming the tool keeps mise from installing the repo's whole
            // toolchain first — one unbuildable pin of the user's would
            // otherwise fail this run.
            actions.push(Action::Run {
                program: "mise".into(),
                args: vec![
                    "exec".into(),
                    CODEGRAPH_MISE_TOOL.into(),
                    "--".into(),
                    "codegraph".into(),
                    "init".into(),
                ],
                purpose: "build the code index".into(),
                undo: None,
                optional: false,
            });
        }
        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Component, Ctx};
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::FakeRunner;

    #[test]
    fn fresh_repo_plans_pin_and_init() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let descs: Vec<String> = Codegraph
            .plan(&ctx)
            .unwrap()
            .iter()
            .map(|a| a.describe())
            .collect();
        assert!(descs.iter().any(|d| d.contains(CODEGRAPH_MISE_TOOL)));
        // The pin carries every platform's bundle and checksum, so a repo
        // synced on one machine installs the same bytes on the others.
        let pin = pin_value("1.5.0");
        for (platform, url, checksum) in CODEGRAPH_PLATFORMS {
            assert!(pin.contains(platform), "{pin}");
            assert!(pin.contains(url), "{pin}");
            assert!(pin.contains(checksum), "{pin}");
        }
        assert!(pin.contains("strip_components = 1"), "{pin}");
        // Never bare `codegraph`: it exists only inside the repo's mise env.
        assert!(
            descs
                .iter()
                .any(|d| d.contains("run `mise exec http:codegraph -- codegraph init`")),
            "descs: {descs:?}"
        );
    }

    #[test]
    fn indexed_repo_with_pin_plans_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let version = manifest.capabilities["code-index"].version.clone().unwrap();
        std::fs::write(
            dir.path().join(".mise.toml"),
            crate::components::mise::set_pin("", CODEGRAPH_MISE_TOOL, &pin_value(&version))
                .unwrap(),
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join(CODEGRAPH_INDEX_DIR)).unwrap();
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(Codegraph.plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn a_version_off_the_registry_default_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("code-index").unwrap().version = Some("9.9.9".into());
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        // superdev ships checksums for one release only, so it can install
        // that one only.
        let err = Codegraph.plan(&ctx).unwrap_err().to_string();
        assert!(err.contains("must match the registry default"), "{err}");
    }

    #[test]
    fn reports_its_slot_and_provider() {
        assert_eq!(Codegraph.capability(), Capability::CodeIndex);
        assert_eq!(Codegraph.provider(), "codegraph");
    }
}
