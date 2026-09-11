//! pipeline.rs — the verb pipeline between manifest and engine: one plan
//! entry and one apply entry, shared by init, status, sync and update.
//!
//! The ordering rules live here, by construction: the custom prune runs
//! before planning (an unpruned just-released skill would read as an orphan),
//! and the orphan pass plans last (removals run after every component write).
//! The binary loads, calls, renders and turns facts into exit codes.

use std::io;
use std::path::Path;

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Ctx};
use crate::components::codegraph::CODEGRAPH_INDEX_DIR;
use crate::components::{skillpack, sokf};
use crate::content::{self, ContentSet, ItemKind, Origin, Owner};
use crate::engine::Planned;
use crate::error::{Error, Result};
use crate::lock::{Lock, PackLock};
use crate::manifest::{Manifest, PackEntry};
use crate::orphan::OrphanPlan;
use crate::pack;
use crate::registry::{self, Pinned};
use crate::runner::CommandRunner;
use crate::{components, engine, orphan, report};

/// Provider name for repo-level actions no capability owns.
use crate::engine::REPO_PROVIDER;

/// How the pipeline treats a manifest pinned off the registry default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanMode {
    /// Plan the version this binary can provide and let the behind lines
    /// carry the news — status must report a stale pin, not fail on it.
    Status,
    /// Refuse to plan: sync would have to act on the pin, and substituting
    /// the default silently is worse than stopping.
    Sync,
}

/// One planning pass over the repo: the actions, the orphan outcome, the
/// report lines, and the pruned lock the apply consumes.
pub struct RepoPlan {
    planned: Vec<Planned>,
    /// Everything the components claim, for reconciling the lock against what
    /// is actually there. Every kind, not only whole files: a mise pin and a
    /// JSON key are recorded on the same terms and go stale the same way.
    claims: Vec<Claim>,
    orphans: OrphanPlan,
    behind: Vec<String>,
    custom: Vec<String>,
    content: Vec<String>,
    /// One record per pack that resolved, for the lock. A dropped entry's
    /// record leaves with it, so a pack's files become orphans by the
    /// ordinary rule.
    packs: Vec<PackLock>,
    blueprint: Option<String>,
    /// The loaded lock with custom-released entries pruned in memory.
    lock: Lock,
    /// True when the prune removed anything — the lock needs saving even
    /// when nothing else is planned.
    lock_changed: bool,
}

impl RepoPlan {
    /// The planned entries, for rendering.
    pub fn planned(&self) -> &[Planned] {
        &self.planned
    }

    /// Prepend an init-only entry — the project template — ahead of every
    /// capability's, so its scaffolds exist before any component write. No
    /// other verb plans templates, which is what keeps them write-once.
    pub fn prepend(&mut self, entry: Planned) {
        self.planned.insert(0, entry);
    }

    /// Whether any entry carries an action.
    pub fn has_actions(&self) -> bool {
        self.planned.iter().any(|p| !p.actions.is_empty())
    }

    /// Whether any entry carries drift: a managed file, pin or entry that no
    /// longer matches the blueprint. `Run` actions are excluded because they
    /// provision external state — a code index, an installed tool — that no
    /// checkout carries and the lock never hashes. A run a real change
    /// triggers is planned beside the write that triggered it, so dropping
    /// runs here hides no drift.
    pub fn has_drift(&self) -> bool {
        self.planned
            .iter()
            .any(|p| p.actions.iter().any(|a| !matches!(a, Action::Run { .. })))
    }

    /// One line per enabled capability pinned away from this binary's registry.
    pub fn behind_lines(&self) -> &[String] {
        &self.behind
    }

    /// One line per skill or workflow skill released to the user.
    pub fn custom_lines(&self) -> &[String] {
        &self.custom
    }

    /// Where the content came from: the layer that is layer 0, each layer
    /// above it, and any item one pack hid from another. Informational —
    /// layering is what the manifest asked for, never drift.
    pub fn content_lines(&self) -> &[String] {
        &self.content
    }

    /// One line per orphan released because the user edited it.
    pub fn released_lines(&self) -> Vec<String> {
        self.orphans.released_lines()
    }

    /// The blueprint-version report: informational, never the exit code. A
    /// settled repo under a newer binary is not drift.
    pub fn blueprint_line(&self) -> Option<&str> {
        self.blueprint.as_deref()
    }
}

/// The result of an apply: the rendered report and what the run did.
pub struct ApplyOutcome {
    /// The rendered apply report; empty when nothing needed applying.
    pub report: String,
    /// False when the engine failed and unwound.
    pub ok: bool,
}

/// Plan the whole repo: prune the custom-released lock entries in memory,
/// plan the repo-level lines and every component, and put the orphan pass
/// last so removals run after every component write — a rename whose write
/// fails rolls back before anything is deleted.
pub fn plan_repo(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    lock: &Lock,
    mode: PlanMode,
) -> Result<RepoPlan> {
    // The report lines describe the manifest as written; Status planning
    // alone runs against the plannable copy below.
    let behind = behind_pins(manifest);
    let blueprint = blueprint_line(manifest);
    let plannable_manifest;
    let manifest = match mode {
        PlanMode::Status => {
            plannable_manifest = plannable(manifest);
            &plannable_manifest
        }
        PlanMode::Sync => {
            if let Some((capability, pinned, default)) = locked_pin_mismatch(manifest) {
                return Err(Error::Manifest {
                    message: format!(
                        "{} is pinned {pinned} but this superdev only supports {default} — run `superdev update`",
                        capability.as_str()
                    ),
                });
            }
            manifest
        }
    };
    let mut lock = lock.clone();
    // Before planning: a skill or workflow just marked custom still has its
    // lock entry, and unpruned an unmodified one would read as an orphan and
    // be deleted — the opposite of what marking it custom asked for.
    // Resolved before planning, so `Component::plan` stays side-effect free
    // and every component reads one content set (ADR-002). `status` resolves
    // offline, which is what makes it provably free of fetching.
    let resolve_mode = match mode {
        PlanMode::Status => pack::ResolveMode::Offline,
        PlanMode::Sync => pack::ResolveMode::Fetching,
    };
    let resolution = pack::resolve(root, runner, manifest, &lock, resolve_mode)?;
    let content = resolution.content;
    // Named against the resolved set, not the embedded one: a custom name
    // guards whatever is shipped now, which a pack may have added to.
    let custom = custom_lines(manifest, &content);
    let content_report = content_lines(manifest, &content, &resolution.pending);
    let lock_changed = prune_custom(manifest, &content, &mut lock);
    let components = components::enabled(manifest)?;
    let ctx = Ctx {
        root,
        runner,
        manifest,
        lock: &lock,
        content: &content,
    };
    let mut planned = Vec::new();
    planned.extend(repo_entry(root, manifest, &content)?);
    planned.extend(engine::plan(&components, &ctx)?);
    let claims_by_component: Vec<(Option<Capability>, String, Vec<Claim>)> = components
        .iter()
        .map(|c| (c.capability(), c.provider().to_string(), c.owned(&ctx)))
        .collect();
    claim_collision(&claims_by_component)?;
    let mut claims: Vec<Claim> = claims_by_component
        .into_iter()
        .flat_map(|(_, _, claims)| claims)
        .collect();
    // Repo-owned files need explicit claims because they do not belong to a
    // component. Without these, each lock entry would look orphaned.
    claims.push(Claim::File(AGGREGATOR_PATH.into()));
    for (kind, directory) in [
        (ItemKind::PiExtension, "extensions"),
        (ItemKind::PiSkill, "skills"),
    ] {
        for item in content.items_of(Owner::Repo, kind) {
            for (relative, _) in &item.files {
                claims.push(Claim::File(format!(
                    ".pi/{directory}/{}/{}",
                    item.name, relative
                )));
            }
        }
    }
    let orphans = orphan::plan(root, &lock, &claims)?;
    if !orphans.actions.is_empty() {
        planned.push(Planned {
            capability: None,
            provider: REPO_PROVIDER.into(),
            actions: orphans.actions.clone(),
        });
    }
    Ok(RepoPlan {
        behind,
        custom,
        content: content_report,
        packs: resolution.packs,
        blueprint,
        planned,
        orphans,
        claims,
        lock,
        lock_changed,
    })
}

/// Apply a plan: reconcile the lock's released, gone and disabled keys, then
/// either save the changed lock and stamp the blueprint (nothing planned) or
/// run the engine, keep the lock only on success, and stamp. The lock and
/// stamp settle before the caller prints `report`: a reader that closes
/// stdout early (`sync | head`) must not leave applied changes without
/// their lock entries. The cost is narrow: a stamp failure after a
/// successful apply surfaces as the error and drops the rendered report.
pub fn apply_repo(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    plan: RepoPlan,
) -> Result<ApplyOutcome> {
    let RepoPlan {
        planned,
        orphans,
        packs,
        claims,
        mut lock,
        mut lock_changed,
        ..
    } = plan;
    // What resolved is what the next run must get again: the digest is the
    // whole of that proof, and a dropped entry's record goes with it so its
    // files orphan by the ordinary rule.
    if lock.packs != packs {
        lock.packs = packs;
        lock_changed = true;
    }
    // Released and gone orphans leave the lock without an action, and a
    // disabled capability's applied record goes with its files.
    for key in orphans.released.iter().chain(orphans.gone.iter()) {
        lock_changed |= lock.files.remove(key).is_some();
    }
    // Nothing writes or reads `owners` any more: clear whatever attribution a
    // pre-removal binary left behind, in one stroke, rather than waiting for
    // each file's rewrite — an entry on an up-to-date file would never see one.
    if !lock.owners.is_empty() {
        lock.owners.clear();
        lock_changed = true;
    }
    let disabled: Vec<String> = lock
        .components
        .keys()
        .filter(|name| !manifest.capabilities.contains_key(*name))
        .cloned()
        .collect();
    for name in disabled {
        lock.components.remove(&name);
        lock_changed = true;
    }
    // A pack removed from a many slot goes the same way as a disabled
    // capability: its record leaves with its files.
    for (name, records) in lock.components.iter_mut() {
        let Some(capability) = Capability::parse(name) else {
            continue;
        };
        let before = records.len();
        records.retain(|r| manifest.config_of(capability, &r.provider).is_some());
        lock_changed |= records.len() != before;
    }
    lock.components.retain(|_, records| !records.is_empty());
    if planned.iter().all(|p| p.actions.is_empty()) {
        if reconcile_lock(root, &claims, &mut lock) {
            lock_changed = true;
        }
        if lock_changed {
            lock.save(root)?;
        }
        stamp_blueprint(root, manifest)?;
        return Ok(ApplyOutcome {
            report: String::new(),
            ok: true,
        });
    }
    let result = engine::apply(root, runner, manifest, &planned, &mut lock);
    if result.ok {
        // After the engine, never before it: a file the user edited differs
        // from the lock *and* from what superdev writes, and reconciling
        // first would bless the edit as the recorded hash — so the write that
        // follows would find them equal and report a plain write, saying
        // nothing about the edit it just overwrote. Everything the engine
        // wrote is already recorded, so what is left here is what it had no
        // reason to touch.
        reconcile_lock(root, &claims, &mut lock);
        lock.save(root)?;
        stamp_blueprint(root, manifest)?;
    }
    Ok(ApplyOutcome {
        report: report::render_apply(&result),
        ok: result.ok,
    })
}

/// Bring the lock's hashes up to what is actually there.
///
/// Every claim kind, read the way the orphan pass reads one: a mise pin and a
/// JSON key are values inside a shared file, and their hashes go stale on the
/// same terms as a whole file's. A stale one costs more, in fact — the orphan
/// pass compares against it to decide whether an entry is superdev's to
/// remove or the user's to keep, so a stale hash leaves superdev's own
/// registration in a shared file for good, over a line saying the user
/// changed it.
///
/// Refreshes what the lock already holds and never adds a key. Adoption
/// leaves a repo's own copy of a shipped file unclaimed when it already
/// matches, deliberately, and it is claimed all the same so it does not read
/// as an orphan — inserting here would quietly take ownership of every one of
/// them on the next run.
///
/// Runs after the engine, never before. A file the user edited differs from
/// the lock and from what superdev writes; reconciling first would record the
/// edit as the hash the engine then compares against, and an overwrite nobody
/// was told about is the failure the hash exists to prevent.
fn reconcile_lock(root: &Path, claims: &[Claim], lock: &mut Lock) -> bool {
    let mut changed = false;
    for claim in claims {
        let key = claim.lock_key();
        let Some(recorded) = lock.files.get(&key) else {
            continue;
        };
        // Absent is not stale: a claim whose file or key is gone is the
        // orphan pass's business, not this one. An unreadable one leaves the
        // recorded hash alone — guessing at content is what the engine
        // refuses to do everywhere else, and this pass runs after a
        // successful apply, too late to turn a repair into a failure.
        let Ok(Some(value)) = claim.read_current(root) else {
            continue;
        };
        let actual = crate::lock::sha256_hex(value.as_bytes());
        if *recorded != actual {
            lock.files.insert(key, actual);
            changed = true;
        }
    }
    changed
}

/// Mark as custom, at init time, everything the repo already carries under a
/// name superdev would manage. Returns the lines to print.
pub fn adopt_existing(root: &Path, manifest: &mut Manifest) -> Vec<String> {
    let content = content::snapshot();
    let mut lines = skillpack::adopt_existing(root, &content, manifest);
    lines.extend(sokf::adopt_existing(root, &content, manifest));
    lines
}

/// The pin for the provider the manifest names, falling back to the default
/// entry when the capability is not enabled or names a provider the registry
/// lacks. None means the version floats. Callers pass single slots only, so
/// the first entry is the whole set; a many slot has no one selected pin.
pub fn selected_pin(manifest: &Manifest, capability: Capability) -> Option<Pinned> {
    manifest
        .configs(capability)
        .first()
        .and_then(|c| registry::entry_for(capability, &c.provider))
        .unwrap_or_else(|| registry::default_entry(capability))
        .version
}

/// The registry version for `provider` in `capability`, when the registry
/// carries that pair.
pub fn registry_version_of(capability: Capability, provider: &str) -> Option<String> {
    registry::entry_for(capability, provider)?
        .version
        .map(|p| p.version.to_string())
}

/// Refuse when two enabled components claim the same lock key — across
/// capabilities or between two packs in one many slot. Deliberate overrides
/// are intra-component, so a cross-component collision is always an
/// accident — silently picking a winner would oscillate across syncs. The
/// message carries the way out; providers are named only when the
/// capability alone cannot tell the two sides apart.
/// What to call a claimant in a collision message: its slot, or the SOKF
/// knowledge for the core component that fills none.
fn slot_name(capability: Option<Capability>) -> &'static str {
    capability.map_or("knowledge", Capability::as_str)
}

fn claim_collision(claims_by_component: &[(Option<Capability>, String, Vec<Claim>)]) -> Result<()> {
    let mut seen: std::collections::BTreeMap<String, (Option<Capability>, String)> =
        std::collections::BTreeMap::new();
    for (capability, provider, claims) in claims_by_component {
        for claim in claims {
            let key = claim.lock_key();
            if let Some((first_cap, first_provider)) = seen.get(&key)
                && !(first_cap == capability && first_provider == provider)
            {
                let (first, second) = if first_cap == capability {
                    (
                        format!("{} ({first_provider})", slot_name(*first_cap)),
                        format!("{} ({provider})", slot_name(*capability)),
                    )
                } else {
                    (
                        slot_name(*first_cap).to_string(),
                        slot_name(*capability).to_string(),
                    )
                };
                return Err(Error::Manifest {
                    message: format!(
                        "{first} and {second} both claim {key} — add its skill to one side's custom list, or upgrade superdev",
                    ),
                });
            }
            seen.insert(key, (*capability, provider.clone()));
        }
    }
    Ok(())
}

/// Where superdev's agent instructions live: the aggregator AGENTS.md
/// imports, owned and rewritten when the canonical source changes.
const AGGREGATOR_PATH: &str = ".agents/superdev.md";
/// The one line superdev keeps in the user's AGENTS.md.
const AGENTS_IMPORT_LINE: &str = "@.agents/superdev.md";
/// Reported once, when the line is appended to an AGENTS.md that already
/// existed — the repos migrating off the old superdev-written scaffold.
const AGENTS_TRIM_HINT: &str = "AGENTS.md is yours — superdev's guidance moved behind @.agents/superdev.md; \
     trim any old superdev-written sections";

/// Canonical agent instructions are copied verbatim, independent of the
/// enabled capabilities. The source no longer carries expansion markers.
const AGGREGATOR_TEMPLATE: &str = include_str!("agent-instructions.md");

/// One file's content, absent as an empty string; other errors propagate.
fn read_or_empty(path: std::path::PathBuf) -> Result<String> {
    match std::fs::read_to_string(&path) {
        Ok(existing) => Ok(existing),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(source) => Err(Error::Io { path, source }),
    }
}

/// The repo-level entry no capability owns: the `.gitignore` lines, the
/// ensured AGENTS.md import, and the instructions aggregator it points at.
fn repo_entry(root: &Path, manifest: &Manifest, content: &ContentSet) -> Result<Option<Planned>> {
    let gitignore = read_or_empty(root.join(".gitignore"))?;
    let mut wanted = vec![(".superdev/cache/".to_string(), "ignore machine state")];
    if manifest.enabled(Capability::CodeIndex) {
        wanted.push((format!("{CODEGRAPH_INDEX_DIR}/"), "ignore the code index"));
    }
    let mut actions: Vec<Action> = wanted
        .into_iter()
        .filter(|(line, _)| !crate::fsutil::has_line(&gitignore, line))
        .map(|(line, reason)| Action::EnsureLine {
            path: ".gitignore".into(),
            line,
            reason: reason.to_string(),
            append_note: None,
        })
        .collect();
    if !crate::fsutil::has_line(&read_or_empty(root.join("AGENTS.md"))?, AGENTS_IMPORT_LINE) {
        actions.push(Action::EnsureLine {
            path: "AGENTS.md".into(),
            line: AGENTS_IMPORT_LINE.into(),
            reason: "make agents read superdev's instructions".into(),
            append_note: Some(AGENTS_TRIM_HINT.into()),
        });
    }
    let aggregator = AGGREGATOR_TEMPLATE.to_string();
    if read_or_empty(root.join(AGGREGATOR_PATH))? != aggregator {
        actions.push(Action::WriteFile {
            path: AGGREGATOR_PATH.into(),
            content: aggregator,
            ownership: crate::action::Ownership::Owned,
            reason: "superdev's agent instructions".into(),
        });
    }
    for item in content.items_of(Owner::Repo, ItemKind::AgentScaffold) {
        let path = format!(".agents/{}.md", item.name);
        // Write-once: the rules are the user's to adapt from the moment they
        // exist, so only an absent file is planned.
        if !root.join(&path).is_file() {
            actions.push(Action::WriteFile {
                path,
                content: item.files[0].1.clone(),
                ownership: crate::action::Ownership::Scaffold,
                reason: "general agent rules".into(),
            });
        }
    }
    for (kind, directory, reason) in [
        (ItemKind::PiExtension, "extensions", "Pi workflow extension"),
        (ItemKind::PiSkill, "skills", "Pi skill"),
    ] {
        for item in content.items_of(Owner::Repo, kind) {
            for (relative, content) in &item.files {
                let path = format!(".pi/{directory}/{}/{}", item.name, relative);
                if read_or_empty(root.join(&path))? != *content {
                    actions.push(Action::WriteFile {
                        path,
                        content: content.clone(),
                        ownership: crate::action::Ownership::Owned,
                        reason: reason.into(),
                    });
                }
            }
        }
    }
    if actions.is_empty() {
        return Ok(None);
    }
    Ok(Some(Planned {
        capability: None,
        provider: REPO_PROVIDER.into(),
        actions,
    }))
}

/// Remove released skills' and workflows' hashes from the lock: a custom
/// name is the user's file, and a stale hash would misread their next edit
/// as drift against superdev content. True when anything was removed.
/// The two `custom` lists and the skills each governs: the skills
/// capability's entry, and the SOKF table. Name-guarded, because both write
/// into `.claude/skills/` and a name in one list must never release the
/// other's file.
fn custom_lists<'a>(
    manifest: &'a Manifest,
    content: &'a ContentSet,
) -> Vec<(&'static str, &'a [String], Vec<&'a str>)> {
    let mut lists: Vec<(&'static str, &[String], Vec<&str>)> = Vec::new();
    if let Some(config) = manifest.config_of(Capability::Skills, "superdev-skills") {
        lists.push((
            Capability::Skills.as_str(),
            config.custom.as_slice(),
            components::skill_names(content, skillpack::OWNER),
        ));
    }
    lists.push((
        sokf::NAME,
        manifest.knowledge.custom.as_slice(),
        components::skill_names(content, sokf::OWNER),
    ));
    lists
}

fn prune_custom(manifest: &Manifest, content: &ContentSet, lock: &mut Lock) -> bool {
    let mut pruned = false;
    for (_, custom, shipped) in custom_lists(manifest, content) {
        for name in custom {
            if !shipped.contains(&name.as_str()) {
                continue;
            }
            // Release the whole skill directory: a knowledge skill is its
            // directory, and the pack's directories hold only SKILL.md.
            let prefix = format!(".claude/skills/{name}/");
            let keys: Vec<String> = lock
                .files
                .keys()
                .filter(|key| key.starts_with(&prefix))
                .cloned()
                .collect();
            for key in keys {
                pruned |= lock.files.remove(&key).is_some();
            }
        }
    }
    pruned
}

/// One line per skill or workflow skill released to the user, so custom
/// state stays visible without reading the manifest. Flags a custom name
/// that names no shipped skill, since marking it custom has no effect.
/// Where the content came from, and what one pack hid from another.
///
/// Which entry superdev treated as the base is inferred from the source, so
/// a wrong match would otherwise be invisible — printing it turns a silent
/// mismatch into one the next command shows. ADR-004.
fn content_lines(manifest: &Manifest, content: &ContentSet, pending: &[PackEntry]) -> Vec<String> {
    let mut lines = Vec::new();
    let base = content.base();
    match base {
        None => lines.push(format!("content: embedded pack {}", embedded_version())),
        Some(Origin::Pack { index, name }) => {
            let rev = manifest
                .packs
                .get(*index)
                .and_then(|entry| entry.rev.as_deref())
                .unwrap_or("no rev");
            lines.push(format!("content: base {name} at {rev}"));
        }
        // The base is always an entry when there is one; the embedded pack
        // is reported by the `None` arm above.
        Some(Origin::Snapshot) => {}
    }
    for (index, entry) in manifest.packs.iter().enumerate() {
        let is_base = matches!(base, Some(Origin::Pack { index: base, .. }) if *base == index);
        if is_base {
            continue;
        }
        // A pin `status` could not satisfy is not layered over anything, and
        // saying it is would tell a drift gate the repo carries content it
        // does not.
        if pending.contains(entry) {
            lines.push(format!(
                "content: {} not resolved — `superdev sync` fetches it",
                entry.source
            ));
        } else {
            lines.push(format!("content: layer {}", entry.source));
        }
    }
    for hidden in content.shadowed() {
        let (Origin::Pack { name: winner, .. }, Origin::Pack { name: loser, .. }) =
            (&hidden.winner, &hidden.loser)
        else {
            continue;
        };
        lines.push(format!(
            "content: {winner} supersedes {loser}'s {}",
            hidden.name
        ));
    }
    lines
}

/// The embedded pack's own version, for the line that names it.
fn embedded_version() -> String {
    content::pack_manifest_source()
        .lines()
        .find_map(|line| line.strip_prefix("version"))
        .and_then(|rest| rest.split('"').nth(1))
        .unwrap_or("unknown")
        .to_string()
}

fn custom_lines(manifest: &Manifest, content: &ContentSet) -> Vec<String> {
    let mut lines = Vec::new();
    for (cap, custom, shipped) in custom_lists(manifest, content) {
        for name in custom {
            lines.push(if shipped.contains(&name.as_str()) {
                format!("{cap}: {name} custom, unmanaged")
            } else {
                format!("{cap}: custom names unknown skill '{name}' — no effect")
            });
        }
    }
    lines
}

/// One line per enabled entry pinned away from this binary's registry. The
/// provider is named only when the slot holds more than one entry — a
/// single-entry line reads as before.
fn behind_pins(manifest: &Manifest) -> Vec<String> {
    let mut lines = Vec::new();
    for capability in Capability::ALL {
        let many = manifest.configs(capability).len() > 1;
        for (provider, pinned, default) in pin_mismatches(manifest, capability) {
            let label = if many {
                format!("{} ({provider})", capability.as_str())
            } else {
                capability.as_str().to_string()
            };
            lines.push(format!(
                "{label}: pinned {pinned}, registry has {default} — run `superdev update`"
            ));
        }
    }
    lines
}

/// Per entry: the provider, its pin and this binary's default, for every
/// registry-locked entry pinned off that default. Every registry-pinned
/// version is locked to the default, so stale means mismatched — there is no
/// is-it-older question to ask; only the default has provenance, so any
/// other pin, newer included, is one superdev cannot install.
fn pin_mismatches(manifest: &Manifest, capability: Capability) -> Vec<(String, String, String)> {
    manifest
        .configs(capability)
        .iter()
        .filter_map(|config| {
            let default = registry_version_of(capability, &config.provider)?;
            let pinned = config.version.clone();
            (pinned.as_deref() != Some(default.as_str())).then(|| {
                (
                    config.provider.clone(),
                    pinned.unwrap_or_else(|| "(unset)".into()),
                    default,
                )
            })
        })
        .collect()
}

/// The first registry-locked entry pinned off this binary's default.
fn locked_pin_mismatch(manifest: &Manifest) -> Option<(Capability, String, String)> {
    Capability::ALL.into_iter().find_map(|capability| {
        pin_mismatches(manifest, capability)
            .into_iter()
            .next()
            .map(|(_, pinned, default)| (capability, pinned, default))
    })
}

/// A copy of the manifest that can be planned: every registry-locked
/// capability back at the default. Unpinned capabilities are left alone —
/// components accept those as given.
fn plannable(manifest: &Manifest) -> Manifest {
    let mut plannable = manifest.clone();
    for capability in Capability::ALL {
        for config in plannable.configs_mut(capability) {
            // No entry means an unknown provider; leave the pin and let the
            // resolution error say so.
            if let Some(version) = registry_version_of(capability, &config.provider) {
                config.version = Some(version);
            }
        }
    }
    plannable
}

/// The blueprint-version report line, when the manifest is stale.
fn blueprint_line(manifest: &Manifest) -> Option<String> {
    (manifest.blueprint != crate::version()).then(|| {
        format!(
            "blueprint {}, binary {} — sync will update it",
            manifest.blueprint,
            crate::version()
        )
    })
}

/// Record this binary's version as the blueprint last applied. Rewrites
/// config.toml only when the value changes.
fn stamp_blueprint(root: &Path, manifest: &Manifest) -> Result<()> {
    if manifest.blueprint != crate::version() {
        let mut manifest = manifest.clone();
        manifest.blueprint = crate::version().to_string();
        manifest.save(root)?;
    }
    Ok(())
}

#[cfg(test)]
/// Where the general-rules scaffolds land, for the tests that converge a repo.
fn rule_scaffold_paths() -> Vec<String> {
    content::test_snapshot()
        .items_of(Owner::Repo, ItemKind::AgentScaffold)
        .map(|item| format!(".agents/{}.md", item.name))
        .collect()
}

#[cfg(test)]
mod tests;
