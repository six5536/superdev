//! components/pin.rs — the shared plan step for registry-locked versions:
//! refuse a manifest version off the registry default, and diff a checksummed
//! `.mise.toml` pin against what the repo has.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::Ctx;
use crate::error::{Error, Result};
use crate::registry::{self, Pinned};

/// The one refusal wording for a version this binary cannot vouch for.
pub fn refusal_message(capability: Capability, pinned: Pinned) -> String {
    format!(
        "{name} version must match the registry default {default} — the {provenance} is the provenance — run `superdev update {name}`",
        name = capability.as_str(),
        default = pinned.version,
        provenance = pinned.provenance.describe(),
    )
}

/// Refuse a manifest version off the registry default; returns the default.
/// Callers are pinned providers, so the registry entry must carry a version.
pub fn require_registry_default(
    ctx: &Ctx<'_>,
    capability: Capability,
    provider: &str,
) -> Result<&'static str> {
    let pinned = registry::entry_for(capability, provider)
        .and_then(|e| e.version)
        .expect("caller is a pinned provider");
    let config = ctx.config(capability).expect("planned only when enabled");
    if config.version.as_deref() != Some(pinned.version) {
        return Err(Error::Manifest {
            message: refusal_message(capability, pinned),
        });
    }
    Ok(pinned.version)
}

/// The pin action for a checksummed tool, when the repo's `.mise.toml` does
/// not already carry it. Refuses a foreign manifest version first.
pub fn planned_pin(
    ctx: &Ctx<'_>,
    capability: Capability,
    provider: &str,
    tool: &str,
    value_toml: &str,
) -> Result<Option<Action>> {
    require_registry_default(ctx, capability, provider)?;
    let current = match std::fs::read_to_string(ctx.root.join(".mise.toml")) {
        Ok(s) => super::mise::current_pin(&s, tool)?,
        Err(_) => None,
    };
    // Round-trip the desired value so layout differences never read as
    // drift: the pin is an inline table, which toml_edit renders its way.
    let desired = super::mise::set_pin("", tool, value_toml)
        .and_then(|s| super::mise::current_pin(&s, tool))?
        .expect("pin just set");
    if current.as_deref() == Some(desired.as_str()) {
        return Ok(None);
    }
    Ok(Some(Action::SetMisePin {
        tool: tool.to_string(),
        value_toml: value_toml.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Ctx;
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::registry::Provenance;
    use crate::runner::FakeRunner;

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
        }
    }

    #[test]
    fn the_refusal_names_the_provenance_and_the_way_forward() {
        let checksum = refusal_message(
            Capability::CodeIndex,
            Pinned {
                version: "1.5.0",
                provenance: Provenance::Checksum,
            },
        );
        assert_eq!(
            checksum,
            "code-index version must match the registry default 1.5.0 — the pinned checksum is the provenance — run `superdev update code-index`"
        );
        let embedded = refusal_message(
            Capability::Skills,
            Pinned {
                version: "0.1.0",
                provenance: Provenance::Embedded,
            },
        );
        assert_eq!(
            embedded,
            "skills version must match the registry default 0.1.0 — the embedded content is the provenance — run `superdev update skills`"
        );
    }

    #[test]
    fn a_foreign_or_missing_version_is_refused_and_the_default_returned() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let fake = FakeRunner::new();
        let lock = Lock::default();

        let default = require_registry_default(
            &ctx(dir.path(), &fake, &manifest, &lock),
            Capability::CodeIndex,
            "codegraph",
        )
        .unwrap();
        assert_eq!(default, "1.5.0");

        manifest.capabilities.get_mut("code-index").unwrap().version = Some("9.9.9".into());
        let err = require_registry_default(
            &ctx(dir.path(), &fake, &manifest, &lock),
            Capability::CodeIndex,
            "codegraph",
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("must match the registry default 1.5.0"),
            "{err}"
        );
        assert!(err.contains("run `superdev update code-index`"), "{err}");

        manifest.capabilities.get_mut("code-index").unwrap().version = None;
        assert!(
            require_registry_default(
                &ctx(dir.path(), &fake, &manifest, &lock),
                Capability::CodeIndex,
                "codegraph"
            )
            .is_err()
        );
    }

    #[test]
    fn planned_pin_diffs_and_normalises_against_the_repo() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let fake = FakeRunner::new();
        let lock = Lock::default();
        let value = "{ version = \"1.5.0\", url = \"https://example.invalid/x.tar.gz\", checksum = \"sha256:00\", strip_components = 1 }";

        // No .mise.toml: the pin is planned.
        let action = planned_pin(
            &ctx(dir.path(), &fake, &manifest, &lock),
            Capability::CodeIndex,
            "codegraph",
            "http:codegraph",
            value,
        )
        .unwrap();
        assert!(
            matches!(action, Some(Action::SetMisePin { .. })),
            "{action:?}"
        );

        // The same pin in a different layout reads as converged, not drift.
        let written = crate::components::mise::set_pin("", "http:codegraph", value).unwrap();
        let relaid = written.replace(", url", ", # pinned\n  url");
        std::fs::write(dir.path().join(".mise.toml"), relaid).unwrap();
        let action = planned_pin(
            &ctx(dir.path(), &fake, &manifest, &lock),
            Capability::CodeIndex,
            "codegraph",
            "http:codegraph",
            value,
        )
        .unwrap();
        assert!(action.is_none(), "{action:?}");
    }
}
