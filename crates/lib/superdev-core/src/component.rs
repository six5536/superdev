//! component.rs — the provider contract: observe, compare, emit actions.

use std::path::Path;

use crate::action::Action;
use crate::capability::Capability;
use crate::error::Result;
use crate::lock::Lock;
use crate::manifest::{CapabilityConfig, Manifest};
use crate::runner::CommandRunner;

/// Everything a component may look at while planning. Read-only.
pub struct Ctx<'a> {
    /// Target repo root.
    pub root: &'a Path,
    /// Process seam for observation commands.
    pub runner: &'a dyn CommandRunner,
    /// Desired state.
    pub manifest: &'a Manifest,
    /// Last-applied state.
    pub lock: &'a Lock,
}

impl Ctx<'_> {
    /// The manifest entry for `capability`, when enabled.
    pub fn config(&self, capability: Capability) -> Option<&CapabilityConfig> {
        self.manifest.capabilities.get(capability.as_str())
    }
}

/// One thing a component owns in a managed repo: the typed form of a lock
/// `files` key. The orphan pass subtracts these from the lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Claim {
    /// A whole superdev-owned file, by repo-relative path.
    File(String),
    /// A managed `[tools]` key in `.mise.toml`.
    MisePin(String),
    /// A managed key in a shared JSON file. `pointer` is dotted, with an
    /// optional trailing `[marker]` naming an array element.
    JsonKey {
        /// Repo-relative file path.
        path: String,
        /// Dotted key path, e.g. `mcpServers.superdev-aokf`.
        pointer: String,
    },
}

impl Claim {
    /// The lock `files` key this claim covers.
    pub fn lock_key(&self) -> String {
        match self {
            Claim::File(path) => path.clone(),
            Claim::MisePin(tool) => crate::components::mise::pin_lock_key(tool),
            Claim::JsonKey { path, pointer } => format!("{path}:{pointer}"),
        }
    }
}

/// One capability provider.
pub trait Component {
    /// Slot this provider fills.
    fn capability(&self) -> Capability;
    /// Provider id, e.g. `"codegraph"`.
    fn provider(&self) -> &'static str;
    /// Observe current state, compare with the manifest, return the diff as
    /// actions. Empty means in sync. Must not change anything.
    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>>;
    /// Everything this component owns in a managed repo, whether or not it
    /// needs changing. Derived from the same constants `plan` writes from —
    /// a converged repo plans nothing, so `plan` output cannot answer this.
    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::FakeRunner;

    struct Nop;
    impl Component for Nop {
        fn capability(&self) -> Capability {
            Capability::Knowledge
        }
        fn provider(&self) -> &'static str {
            "nop"
        }
        fn plan(&self, _ctx: &Ctx<'_>) -> crate::error::Result<Vec<crate::action::Action>> {
            Ok(Vec::new())
        }
        fn owned(&self, _ctx: &Ctx<'_>) -> Vec<Claim> {
            Vec::new()
        }
    }

    #[test]
    fn ctx_exposes_capability_config() {
        let manifest = Manifest::default_for("0.1.0", &[Capability::CodeIndex]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: std::path::Path::new("."),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(ctx.config(Capability::Workflows).is_some());
        assert!(ctx.config(Capability::CodeIndex).is_none());
        assert_eq!(Nop.capability(), Capability::Knowledge);
        assert_eq!(Nop.provider(), "nop");
        assert!(Nop.plan(&ctx).unwrap().is_empty());
    }
}
