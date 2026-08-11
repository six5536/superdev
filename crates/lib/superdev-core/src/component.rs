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

/// One capability provider.
pub trait Component {
    /// Slot this provider fills.
    fn capability(&self) -> Capability;
    /// Provider id, e.g. `"codegraph"`.
    fn provider(&self) -> &'static str;
    /// Observe current state, compare with the manifest, return the diff as
    /// actions. Empty means in sync. Must not change anything.
    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>>;
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
