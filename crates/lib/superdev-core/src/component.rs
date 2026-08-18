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

    /// A lock key, parsed back into the claim shape its format encodes —
    /// the inverse of [`Claim::lock_key`], kept beside it so encode and
    /// decode meet at one seam. superdev never writes a file path containing
    /// `:`, so a colon means a managed entry in a shared file.
    pub fn parse_key(key: &str) -> Claim {
        if let Some(tool) = key.strip_prefix(crate::components::mise::PIN_LOCK_PREFIX) {
            return Claim::MisePin(tool.to_string());
        }
        match key.split_once(':') {
            Some((path, pointer)) => Claim::JsonKey {
                path: path.to_string(),
                pointer: pointer.to_string(),
            },
            None => Claim::File(key.to_string()),
        }
    }

    /// The file this claim lives in — the claimed file itself for the
    /// whole-file shape, the shared file for the others.
    pub fn file_path(&self) -> &str {
        match self {
            Claim::File(path) => path,
            Claim::MisePin(_) => ".mise.toml",
            Claim::JsonKey { path, .. } => path,
        }
    }

    /// The claimed value inside `content` — the text the lock's hash was
    /// taken over. `None` when the entry is absent from the file.
    pub fn value_in(&self, content: &str) -> Result<Option<String>> {
        match self {
            Claim::File(_) => Ok(Some(content.to_string())),
            Claim::MisePin(tool) => crate::components::mise::current_pin(content, tool),
            Claim::JsonKey { path, pointer } => {
                crate::json_edit::json_value_at(path, content, pointer)
            }
        }
    }

    /// What the repo currently holds for this claim — the text its lock hash
    /// was taken over. `None` when the file or the entry is gone.
    pub fn read_current(&self, root: &Path) -> Result<Option<String>> {
        match crate::fsutil::read_text(&root.join(self.file_path()))? {
            None => Ok(None),
            Some(content) => self.value_in(&content),
        }
    }

    /// The file content with this claim's entry removed. `None` means the
    /// claim is the whole file, so removal is deletion, not a rewrite.
    /// Callers verify presence first (via [`Claim::value_in`]).
    pub fn removed_from(&self, content: &str) -> Result<Option<String>> {
        match self {
            Claim::File(_) => Ok(None),
            Claim::MisePin(tool) => Ok(Some(
                crate::components::mise::remove_pin(content, tool)?.expect("the pin is present"),
            )),
            Claim::JsonKey { path, pointer } => Ok(Some(
                crate::json_edit::remove_json_pointer(path, content, pointer)?
                    .expect("the entry is present")
                    .0,
            )),
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
    fn claim_keys_round_trip_through_parse() {
        let claims = [
            Claim::File(".claude/skills/tdd/SKILL.md".into()),
            Claim::MisePin("http:codegraph".into()),
            Claim::JsonKey {
                path: ".mcp.json".into(),
                pointer: "mcpServers.superdev-aokf".into(),
            },
        ];
        for claim in claims {
            assert_eq!(Claim::parse_key(&claim.lock_key()), claim);
        }
        // The colon invariant: a shared-file key parses by its first colon,
        // and the mise prefix wins over the generic split.
        assert_eq!(
            Claim::parse_key(".mise.toml:http:codegraph"),
            Claim::MisePin("http:codegraph".into())
        );
    }

    #[test]
    fn each_shape_reads_and_removes_its_own_entry() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("owned.txt"), "whole file").unwrap();
        let mise_toml =
            crate::components::mise::set_pin("", "http:codegraph", "\"1.5.0\"").unwrap();
        std::fs::write(dir.path().join(".mise.toml"), &mise_toml).unwrap();
        std::fs::write(
            dir.path().join(".mcp.json"),
            r#"{"mcpServers":{"superdev-aokf":{"command":"superdev"},"theirs":{}}}"#,
        )
        .unwrap();

        let file = Claim::File("owned.txt".into());
        assert_eq!(
            file.read_current(dir.path()).unwrap().as_deref(),
            Some("whole file")
        );
        // The whole file is the claim: removal is deletion, not a rewrite.
        assert_eq!(file.removed_from("whole file").unwrap(), None);

        let pin = Claim::MisePin("http:codegraph".into());
        let value = pin.read_current(dir.path()).unwrap().unwrap();
        assert!(value.contains("1.5.0"), "{value}");
        let next = pin.removed_from(&mise_toml).unwrap().unwrap();
        assert!(!next.contains("http:codegraph"), "{next}");

        let key = Claim::JsonKey {
            path: ".mcp.json".into(),
            pointer: "mcpServers.superdev-aokf".into(),
        };
        let value = key.read_current(dir.path()).unwrap().unwrap();
        assert!(value.contains("superdev"), "{value}");
        let next = key
            .removed_from(&std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
            .unwrap()
            .unwrap();
        assert!(!next.contains("superdev-aokf"), "{next}");
        assert!(next.contains("theirs"), "{next}");

        // Absent entries read as None; a gone file reads as None.
        assert_eq!(
            Claim::MisePin("http:absent".into())
                .read_current(dir.path())
                .unwrap(),
            None
        );
        assert_eq!(
            Claim::File("missing.txt".into())
                .read_current(dir.path())
                .unwrap(),
            None
        );
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
