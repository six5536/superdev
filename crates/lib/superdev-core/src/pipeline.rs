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
    // The aggregator is repo-level: no component claims it, and without a
    // live claim its lock entry would read as an orphan every run.
    claims.push(Claim::File(AGGREGATOR_PATH.into()));
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
/// imports, owned and rewritten as the enabled set changes.
const AGGREGATOR_PATH: &str = ".agents/superdev.md";
/// The one line superdev keeps in the user's AGENTS.md.
const AGENTS_IMPORT_LINE: &str = "@.agents/superdev.md";
/// Reported once, when the line is appended to an AGENTS.md that already
/// existed — the repos migrating off the old superdev-written scaffold.
const AGENTS_TRIM_HINT: &str = "AGENTS.md is yours — superdev's guidance moved behind @.agents/superdev.md; \
     trim any old superdev-written sections";

/// The aggregator up to the code-index section: the workflow, the
/// knowledge rules and where the knowledge section ends.
const AGGREGATOR_PREFIX: &str = r#"# Prime Directive

YOU (the SYSTEM) are superdev, an AI coding assistant specialized in structured coding tasks.
YOU maintain a canonical knowledge store (SOKF) and run a contract-driven feature workflow.
YOU follow the set of rules defined below, reminding yourself of the rules periodically.

<superdev>
<workflow note="run each phase by invoking its skill; the skill carries the phase's full process">
  <flow>FRAME → CONTRACT-DESIGN → FEATURE-PLAN → BUILD → INTEGRATE</flow>
  <phase name="FRAME" skill="/frame" doc="feature-request" note="frame the feature and record it as an issue" />
  <phase name="CONTRACT-DESIGN" skill="/contract-design" doc="contract" note="durable contract documents, public and internal, keyed to an interface and updated as features change it; the feature-request links each contract it touched" />
  <phase name="FEATURE-PLAN" skill="/feature-plan" doc="feature-plan" note="cuts the feature into slices, each carrying its cases; settled by lifecycle at the last integrate" />
  <phase name="BUILD" skill="/build" note="tests, then code, one slice at a time" />
  <phase name="INTEGRATE" skill="/integrate" note="verify and integrate the slice" />
  <phase name="ACCEPT" skill="/accept" note="feature-level acceptance on the merged code" />
  <outside skill="/file" when="an issue or an idea to record without framing it — /frame frames it when it is taken up" />
  <outside skill="/adhoc-plan" when="one-off work that needs no feature framing — a refactor, a migration, a chore" />
  <outside skill="/execute-feature-plan" when="unattended delivery — drives FEATURE-PLAN → BUILD → INTEGRATE in a loop on the feature's branch, deferring the user's questions" />
  <edge from="BUILD" when="contract change needed" to="CONTRACT-DESIGN" />
  <edge from="BUILD" when="slice too big" to="FEATURE-PLAN" />
  <edge from="INTEGRATE" when="a check fails" to="BUILD" />
  <edge from="INTEGRATE" when="an acceptance criterion is ambiguous or wrong" to="FRAME" />
  <edge from="INTEGRATE" when="a case is ambiguous or wrong" to="FEATURE-PLAN" />
  <edge from="INTEGRATE" when="the contract should adopt a divergence" to="CONTRACT-DESIGN" />
  <edge from="INTEGRATE" when="next slice" to="BUILD" />
  <edge from="INTEGRATE" when="slice list needs re-cutting" to="FEATURE-PLAN" />
  <edge from="INTEGRATE" when="last slice" to="DONE" />
  <entry to="ACCEPT" when="the user requests acceptance, once the feature has stopped changing" />
  <edge from="ACCEPT" when="gaps found" to="FEATURE-PLAN" />
  <edge from="ACCEPT" when="clean pass" to="DONE" />
</workflow>

<knowledge purpose="canonical data store">
Store all canonical project knowledge in the SOKF knowledge under
`knowledge/`:
@../knowledge/index.md
<tool_call name="read_file" path=".agents/sokf/SPEC.md" when="always" />
<tool_call name="sokf_overview" when="always" />
<retrieval>
  <tool_call name="sokf_graph" when="if following links between concepts" />
  <tool_call name="sokf_search" when="if the concept id is not known" />
  <tool_call name="sokf_read" id="schema-{type}" when="before opening a {type} document, whether to read it, update it, or create it" why="understand document better"/>
  <tool_call name="sokf_read" when="before editing a concept" />
</retrieval>
<validation when="if anything under `knowledge/`, `.claude/skills/` or `.agents/` changed"
  until="the validator reports PASS">
  <tool_call name="superdev validate --fix" when="always" />
</validation>
</knowledge>

"#;

/// The code-index section, present only while that capability is
/// enabled.
const AGGREGATOR_CODE_INDEX: &str = r#"<code-exploration purpose="codegraph code index">
Query the codegraph index before grepping or reading files one by one.
<retrieval>
  <tool_call name="codegraph_explore" when="always — 'how does X work', flows ('how does X reach Y'), area surveys" why="returns the relevant symbols' source plus call paths in one shot" />
</retrieval>
</code-exploration>

"#;

/// The rest of the document: the tool rules, the core principles and
/// the skill adaptations.
const AGGREGATOR_SUFFIX: &str = r#"<tools>
<rule level="SHALL">Always use internal and MCP tools before Bash. Use Bash when nothing else suffices</rule>
</tools>

<core_principles>

- Contracts bind: code never diverges from a contract;
- Knowledge, code and tests must be kept in sync at all times
- The code is canonical
- KISS: Simple solutions over clever ones
- YAGNI: Build only what's specified
- DRY: Research existing code and docs before creating new, avoid duplication at all costs.

<grammar_rules>
superdev communicates as a consummate professional, in conversation and in writing.

## Documents

1. Modal verb discipline; "Must" for requirements, "should" for recommendations, "may" for options (RFC 2119). Never mix them.
2. Avoid vague qualifiers; Replace "fast" and "as needed" with measurable values: "under 200 ms at p99."
3. Consistent terminology; One term per concept. Don't alternate between "endpoint," "route," and "API."
4. Imperative mood for instructions; "Run `npm install`," not "The dependencies should be installed."
5. Active voice; "The scheduler evicts idle pods," not "Idle pods are evicted." Name the responsible component.
6. Present tense for system behavior; "The cache invalidates entries after 60 seconds." Stay consistent.
7. Parallel structure; All list items share the same grammatical form.
8. Numerals with units and a space; "5 ms," "16 GB." Spell out numbers that start sentences.
9. Restrictive vs. non-restrictive clauses; "That" (no comma) restricts; "which" (with comma) adds info.
10. Define acronyms at first use; "service level objective (SLO)."
11. Hyphenate compound modifiers before nouns; "read-only replica," but "the replica is read only."
12. Avoid noun stacks; Rewrite "deployment pipeline failure notification configuration" with prepositions.
13. Subject–verb agreement; Treat "data" consistently per your style guide.
14. Keep verb and object close together; Don't bury the verb under qualifying phrases.
15. Use articles consistently; Don't drop "a," "an," or "the" telegraphically.
16. Avoid contractions in formal specs; "Do not," not "don't." House style may relax this for READMEs.

## Conversation

17. Answer concisely; Let the reader ask for detail.
18. Restate only if it adds clarity; Confirm the restatement. I say "add the dongle to the device," you say "Should I insert the USB drive into the laptop?"
19. No hedging; "I don't know," not "This might potentially cause issues in some cases."
20. No buddy language.

## Both

21. Avoid ambiguous pronouns; Repeat the noun instead of "it" or "this" when the referent is unclear.
22. Modifier placement; "Only restart the primary node" ≠ "Restart only the primary node."
23. One idea per sentence; Under ~25 words. Don't bury preconditions and error cases.
24. Positive constructions; "Keep the flag disabled," not layered negatives.
25. Write for context; If the reader may not know a word or concept, reference or describe it.
26. Test each clause by deleting it; If the reader would act the same, leave it deleted.
27. No drama.
28. No paraphrasing around the precise word; Write "fragile," not "works, then breaks the moment anything nearby changes."
29. No misused words; "Rule," not "invariant," unless it is one.
30. No negation without meaning; "This is a redesign," not "This isn't just a refactor — it's a complete redesign."
31. No filler steps; "Read the spec before writing code," not "Read the spec, understand the constraints, and then write the code."
32. No filler openings; Delete "The key insight is…"
33. No unrequested justification; "Use `/` paths," not "Use `/` paths, since they survive a file move." A "because" clause earns its place only if the reader acts on it.
34. No preemptive defense; Delete rebuttals to objections nobody raised, e.g. "The classes are not general permissions: an agent edits `status` freely."
</grammar_rules>

<coding>
superdev writes code as a consummate professional, at the level of a technical lead.
A reviewer reads every line to extract meaning; efficient work requires efficient code.

<rules>
<rule level="SHALL">Apply DRY: Research the existing code before writing: the logic may already exist, and new code
  must fit the structure it joins.</rule>
<rule level="SHALL">Apply KISS and YAGNI; build only what is requested.</rule>
<rule level="SHALL">Consider edge cases and error handling.</rule>
<rule level="SHALL">Write tests to cover the requirements and success criteria; prefer test-driven
  development, with discretion (e.g. UI development).</rule>
<rule level="SHALL">Document important code interfaces.</rule>
<rule level="SHALL">Read and conform to the coding standards.</rule>
<rule level="SHALL">Use any tools that help write and test code (e.g. MCP tools for result visualization).</rule>
<rule level="MUST NOT">hack a fix; research the existing code and fix at the root.</rule>
<rule level="MUST NOT">silently swallow errors; an error that cannot be handled propagates with context.</rule>
<rule level="MUST NOT">duplicate logic to avoid a refactor; two copies means two bugs.</rule>
<rule level="MUST NOT">change behaviour and tests in the same breath to make a suite go green.
  Fix the code, or change the test deliberately and say why.</rule>
</rules>
</coding>
</core_principles>

<skill_adaptations>
If a `PROJECT.md` exists in an invoked skill's directory, apply it; it has precedence for conflicts.
</skill_adaptations>
</superdev>
"#;

/// The fenced aggregator: superdev's agent instructions as one document.
/// The code-index section is the only conditional part.
fn aggregator_content(manifest: &Manifest) -> String {
    let mut out = String::from(AGGREGATOR_PREFIX);
    if manifest.enabled(Capability::CodeIndex) {
        out.push_str(AGGREGATOR_CODE_INDEX);
    }
    out.push_str(AGGREGATOR_SUFFIX);
    out
}

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
    let aggregator = aggregator_content(manifest);
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
mod tests {
    use super::*;
    use crate::lock::sha256_hex;
    use crate::runner::FakeRunner;

    /// Every capability whose default entry is registry-locked, derived so
    /// no test re-encodes the list the registry owns.
    fn locked_capabilities() -> Vec<Capability> {
        let locked: Vec<Capability> = registry::entries()
            .iter()
            .filter(|e| e.default && e.version.is_some())
            .map(|e| e.capability)
            .collect();
        assert!(!locked.is_empty());
        locked
    }

    fn pin(manifest: &mut Manifest, capability: Capability, version: Option<&str>) {
        manifest.configs_mut(capability)[0].version = version.map(str::to_string);
    }

    #[test]
    fn any_locked_pin_off_the_default_is_stale() {
        for capability in locked_capabilities() {
            let name = capability.as_str();
            let default = registry::default_entry(capability)
                .version
                .unwrap()
                .version
                .to_string();
            let mut manifest = Manifest::default_for("0.1.0", &[]);
            assert!(pin_mismatches(&manifest, capability).is_empty());
            assert!(behind_pins(&manifest).is_empty());

            pin(&mut manifest, capability, Some("1.0.0"));
            let provider = manifest.configs(capability)[0].provider.clone();
            assert_eq!(
                pin_mismatches(&manifest, capability),
                vec![(provider, "1.0.0".to_string(), default.clone())]
            );
            assert_eq!(
                behind_pins(&manifest),
                vec![format!(
                    "{name}: pinned 1.0.0, registry has {default} — run `superdev update`"
                )]
            );

            // A newer pin is not "behind", but superdev still cannot install it.
            pin(&mut manifest, capability, Some("9.9.9"));
            assert!(!pin_mismatches(&manifest, capability).is_empty());
            assert_eq!(locked_pin_mismatch(&manifest).unwrap().0, capability);
            pin(&mut manifest, capability, None);
            assert!(behind_pins(&manifest)[0].contains("pinned (unset)"));
        }
    }

    #[test]
    fn plannable_resets_every_locked_pin() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        for capability in locked_capabilities() {
            pin(&mut manifest, capability, Some("1.0.0"));
        }
        let plannable = plannable(&manifest);
        assert!(locked_pin_mismatch(&plannable).is_none());
        // Pins with no provenance beside them are left exactly as written.
        // frontend is such a slot: the registry pins no version for it.
        assert_eq!(
            plannable.capabilities["frontend"][0].version,
            manifest.capabilities["frontend"][0].version
        );
    }

    #[test]
    fn status_mode_plans_the_default_and_reports_behind() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for(crate::version(), &[]);
        pin(&mut manifest, Capability::Skills, Some("0.0.1"));
        let fake = FakeRunner::new();
        // Sync refuses the stale pin outright.
        let err = match plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Sync,
        ) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("a stale locked pin must refuse to plan"),
        };
        assert!(err.contains("only supports"), "{err}");
        // Status plans the version this binary can provide, and the behind
        // lines describe the manifest as written — not the plannable copy.
        let plan = plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Status,
        )
        .unwrap();
        assert!(plan.has_actions());
        assert_eq!(plan.behind_lines().len(), 1);
        assert!(plan.behind_lines()[0].starts_with("skills: pinned 0.0.1"));
    }

    /// The `--drift` gate's contract: a run provisions external state no
    /// checkout carries, so it is work to do without being drift. Every run
    /// a real change triggers is planned beside that change, which is what
    /// makes dropping runs from the exit code safe.
    #[test]
    fn a_provisioning_run_is_work_to_do_but_not_drift() {
        // Every capability disabled, and everything still planned already in
        // place — the repo entry's files and SOKF's, which no flag disables:
        // a settled tree, so the plan starts empty and the asserts below
        // speak only about what the test prepends.
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for(crate::version(), &Capability::ALL);
        std::fs::create_dir_all(dir.path().join(".agents")).unwrap();
        std::fs::write(
            dir.path().join(".agents/superdev.md"),
            aggregator_content(&manifest),
        )
        .unwrap();
        for scaffold in rule_scaffold_paths() {
            std::fs::write(dir.path().join(scaffold), "the user's now\n").unwrap();
        }
        std::fs::write(dir.path().join(".gitignore"), ".superdev/cache/\n").unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "@.agents/superdev.md\n").unwrap();
        let fake = FakeRunner::new();
        settle_sokf(dir.path(), &manifest, &fake);
        let mut plan = plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Status,
        )
        .unwrap();
        assert!(!plan.has_actions(), "descs: {:?}", plan_descs(&plan));

        plan.prepend(Planned {
            capability: Some(Capability::CodeIndex),
            provider: "codegraph".into(),
            actions: vec![Action::Run {
                program: "mise".into(),
                args: vec!["exec".into()],
                purpose: "build the code index".into(),
                undo: None,
                optional: false,
            }],
        });
        assert!(plan.has_actions(), "a run is still work to do");
        assert!(!plan.has_drift(), "a run alone is not drift");

        plan.prepend(Planned {
            capability: Some(Capability::CodeIndex),
            provider: "codegraph".into(),
            actions: vec![Action::WriteFile {
                path: ".mcp.json".into(),
                content: "x".into(),
                ownership: crate::action::Ownership::Owned,
                reason: "code-index MCP registration".into(),
            }],
        });
        assert!(plan.has_drift(), "a managed file is drift");
    }

    /// Apply what the SOKF component plans, so a test that wants a settled
    /// tree gets one. SOKF is core, so it plans in every repo — a test that
    /// left its files unwritten would be measuring the scaffold rather than
    /// whatever it meant to measure.
    fn settle_sokf(root: &std::path::Path, manifest: &Manifest, fake: &FakeRunner) {
        use crate::component::Component;
        let ctx = crate::component::Ctx {
            root,
            runner: fake,
            manifest,
            lock: &Lock::default(),
            content: crate::content::test_snapshot(),
        };
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: None,
            provider: crate::components::sokf::NAME.into(),
            actions: crate::components::sokf::Sokf.plan(&ctx).unwrap(),
        }];
        assert!(engine::apply(root, fake, manifest, &planned, &mut lock).ok);
    }

    /// Every planned action description, flattened for substring asserts.
    fn plan_descs(plan: &RepoPlan) -> Vec<String> {
        plan.planned()
            .iter()
            .flat_map(|p| p.actions.iter().map(|a| a.describe()))
            .collect()
    }

    #[test]
    fn a_pack_dropped_from_the_manifest_loses_its_lock_record() {
        use crate::lock::LockedComponent;

        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        // The manifest keeps one pack; the lock still records two.
        let manifest = Manifest::default_for(crate::version(), &[]);
        let mut lock = Lock::default();
        lock.components.insert(
            "skills".into(),
            vec![
                LockedComponent {
                    provider: "superdev-skills".into(),
                    version: Some(crate::version().to_string()),
                },
                LockedComponent {
                    provider: "another-pack".into(),
                    version: Some("1.2.0".into()),
                },
            ],
        );
        let plan = RepoPlan {
            planned: Vec::new(),
            orphans: OrphanPlan::default(),
            behind: Vec::new(),
            custom: Vec::new(),
            content: Vec::new(),
            packs: Vec::new(),
            blueprint: None,
            claims: Vec::new(),
            lock,
            lock_changed: false,
        };
        assert!(apply_repo(dir.path(), &fake, &manifest, plan).unwrap().ok);
        let saved = Lock::load(dir.path()).unwrap();
        let providers: Vec<&str> = saved.components["skills"]
            .iter()
            .map(|r| r.provider.as_str())
            .collect();
        // The dropped pack's record went; the kept pack's stayed. Its files
        // go the generic way: claims no longer cover them, so the orphan
        // pass classifies them like any other orphan.
        assert_eq!(providers, ["superdev-skills"]);
    }

    /// A pin `status` could not satisfy is not layered over anything. Saying
    /// it is would tell a drift gate the repo carries content it does not.
    #[test]
    fn a_pending_pack_is_not_reported_as_a_layer() {
        let mut manifest = Manifest::default_for("0.2.0", &[]);
        manifest.packs = vec![PackEntry {
            source: "github:someone/other".into(),
            rev: Some("v9".into()),
        }];
        let content = content::snapshot();
        let pending = manifest.packs.clone();

        let unresolved = content_lines(&manifest, &content, &pending);
        assert!(
            unresolved.iter().any(|l| l.contains("not resolved")),
            "{unresolved:?}"
        );
        assert!(
            !unresolved.iter().any(|l| l.contains("layer")),
            "{unresolved:?}"
        );

        // The same entry, resolved, is a layer.
        let resolved = content_lines(&manifest, &content, &[]);
        assert!(
            resolved
                .iter()
                .any(|l| l == "content: layer github:someone/other"),
            "{resolved:?}"
        );
    }

    #[test]
    fn aggregator_imports_track_the_enabled_set() {
        let all = aggregator_content(&Manifest::default_for("0.1.0", &[]));
        assert!(all.starts_with("# Prime Directive\n"), "{all}");
        assert!(all.contains("<code-exploration"), "{all}");
        assert_eq!(
            all,
            format!("{AGGREGATOR_PREFIX}{AGGREGATOR_CODE_INDEX}{AGGREGATOR_SUFFIX}")
        );
        // The code-index section is the only conditional part: with the
        // capability off, everything else still reads.
        let none = aggregator_content(&Manifest::default_for("0.1.0", &[Capability::CodeIndex]));
        assert!(!none.contains("<code-exploration"), "{none}");
        assert!(none.contains("<superdev>"), "{none}");
        assert!(none.contains("<knowledge"), "{none}");
        assert!(none.contains("<tools>"), "{none}");
        assert!(none.contains("<grammar_rules>"), "{none}");
        assert!(none.contains("<coding>"), "{none}");
        assert_eq!(none, format!("{AGGREGATOR_PREFIX}{AGGREGATOR_SUFFIX}"));
    }

    #[test]
    fn repo_entry_plans_the_import_line_and_the_aggregator_once() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let entry = repo_entry(dir.path(), &manifest, content::test_snapshot())
            .unwrap()
            .unwrap();
        let descs: Vec<String> = entry.actions.iter().map(Action::describe).collect();
        assert!(
            descs
                .iter()
                .any(|d| d.contains("ensure AGENTS.md contains `@.agents/superdev.md`")),
            "{descs:?}"
        );
        assert!(
            descs
                .iter()
                .any(|d| d.contains("write .agents/superdev.md")),
            "{descs:?}"
        );
        for path in rule_scaffold_paths() {
            assert!(
                descs.iter().any(|d| d.contains(&path)),
                "{path} missing from {descs:?}"
            );
        }
        // A settled repo replans nothing — and the rule scaffolds count as
        // settled whatever their content, because they are the user's.
        std::fs::write(
            dir.path().join(".gitignore"),
            ".superdev/cache/\n.codegraph/\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("AGENTS.md"),
            "# Mine\n@.agents/superdev.md\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join(".agents")).unwrap();
        std::fs::write(
            dir.path().join(AGGREGATOR_PATH),
            aggregator_content(&manifest),
        )
        .unwrap();
        for path in rule_scaffold_paths() {
            std::fs::write(dir.path().join(path), "adapted by the user\n").unwrap();
        }
        assert!(
            repo_entry(dir.path(), &manifest, content::test_snapshot())
                .unwrap()
                .is_none()
        );
    }

    /// The SOKF component plans every carried skill file, so a
    /// `[knowledge] custom` name can release any of them.
    #[test]
    fn knowledge_plans_the_full_carried_skill_set() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for(crate::version(), &[]);
        let plan = plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Sync,
        )
        .unwrap();
        let descs = plan_descs(&plan);
        for name in components::skill_names(content::test_snapshot(), sokf::OWNER) {
            assert!(
                descs
                    .iter()
                    .any(|d| d.contains(&format!(".claude/skills/{name}/SKILL.md"))),
                "{name} missing from the plan"
            );
        }
    }

    #[test]
    fn a_cross_capability_claim_collision_refuses_with_the_way_out() {
        let a = (
            Some(Capability::Skills),
            "superdev-skills".to_string(),
            vec![Claim::File(".claude/skills/grilling/SKILL.md".into())],
        );
        let b = (
            None,
            crate::components::sokf::NAME.to_string(),
            vec![Claim::File(".claude/skills/grilling/SKILL.md".into())],
        );
        let err = claim_collision(&[a, b]).unwrap_err().to_string();
        assert!(
            err.contains("skills and knowledge both claim .claude/skills/grilling/SKILL.md"),
            "{err}"
        );
        assert!(err.contains("custom list"), "{err}");

        // The same component claiming a key twice is not a collision, and
        // distinct keys never are.
        let dup = (
            Some(Capability::Skills),
            "superdev-skills".to_string(),
            vec![Claim::File("a.txt".into()), Claim::File("a.txt".into())],
        );
        let other = (
            None,
            crate::components::sokf::NAME.to_string(),
            vec![Claim::File("b.txt".into())],
        );
        assert!(claim_collision(&[dup, other]).is_ok());
    }

    #[test]
    fn two_packs_in_one_slot_colliding_name_both_providers() {
        let a = (
            Some(Capability::Skills),
            "superdev-skills".to_string(),
            vec![Claim::File(".claude/skills/humanise/SKILL.md".into())],
        );
        let b = (
            Some(Capability::Skills),
            "another-pack".to_string(),
            vec![Claim::File(".claude/skills/humanise/SKILL.md".into())],
        );
        let err = claim_collision(&[a, b]).unwrap_err().to_string();
        assert!(
            err.contains(
                "skills (superdev-skills) and skills (another-pack) both claim \
                 .claude/skills/humanise/SKILL.md"
            ),
            "{err}"
        );
        assert!(err.contains("custom list"), "{err}");
    }

    #[test]
    fn custom_skills_are_pruned_from_the_lock_and_reported() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("skills").unwrap()[0].custom =
            vec!["template-update".into(), "grill-me".into()];
        let mut lock = Lock::default();
        lock.files.insert(
            ".claude/skills/template-update/SKILL.md".into(),
            "hash-a".into(),
        );
        lock.files.insert(
            ".claude/skills/double-check/SKILL.md".into(),
            "hash-b".into(),
        );
        assert!(prune_custom(&manifest, content::test_snapshot(), &mut lock));
        assert!(
            !lock
                .files
                .contains_key(".claude/skills/template-update/SKILL.md")
        );
        assert!(
            lock.files
                .contains_key(".claude/skills/double-check/SKILL.md")
        );
        // Nothing left to prune: reports no change.
        assert!(!prune_custom(
            &manifest,
            content::test_snapshot(),
            &mut lock
        ));

        assert_eq!(
            custom_lines(&manifest, content::test_snapshot()),
            vec![
                "skills: template-update custom, unmanaged".to_string(),
                "skills: custom names unknown skill 'grill-me' — no effect".to_string(),
            ]
        );
        let no_skills = Manifest::default_for("0.1.0", &[Capability::Skills]);
        assert!(custom_lines(&no_skills, content::test_snapshot()).is_empty());
    }

    #[test]
    fn knowledge_custom_entries_release_the_whole_directory() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.knowledge.custom = vec!["prototype".into()];
        let mut lock = Lock::default();
        // A skill-pack file, untouched by the knowledge custom list.
        lock.files
            .insert(".claude/skills/double-check/SKILL.md".into(), "h".into());
        for key in [
            ".claude/skills/prototype/SKILL.md",
            ".claude/skills/prototype/refs/A.md",
        ] {
            lock.files.insert(key.into(), "h".into());
        }
        lock.files
            .insert(".claude/skills/frame/SKILL.md".into(), "h".into());
        assert!(prune_custom(&manifest, content::test_snapshot(), &mut lock));
        assert!(!lock.files.keys().any(|k| k.contains("/prototype/")));
        assert!(lock.files.contains_key(".claude/skills/frame/SKILL.md"));
        assert!(
            lock.files
                .contains_key(".claude/skills/double-check/SKILL.md")
        );
        // Nothing left to prune: reports no change.
        assert!(!prune_custom(
            &manifest,
            content::test_snapshot(),
            &mut lock
        ));
    }

    #[test]
    fn knowledge_custom_lines_cover_every_carried_skill() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.knowledge.custom = vec!["prototype".into(), "flying".into()];
        let lines = custom_lines(&manifest, content::test_snapshot());
        assert!(lines.contains(&"knowledge: prototype custom, unmanaged".to_string()));
        assert!(
            lines.contains(
                &"knowledge: custom names unknown skill 'flying' — no effect".to_string()
            )
        );
        // Every carried skill is a known custom name.
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.knowledge.custom = components::skill_names(content::test_snapshot(), sokf::OWNER)
            .into_iter()
            .map(String::from)
            .collect();
        for line in custom_lines(&manifest, content::test_snapshot()) {
            assert!(line.ends_with("custom, unmanaged"), "{line}");
        }
    }

    #[test]
    fn plan_repo_puts_the_orphan_entry_last_and_reports_released() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let manifest = Manifest::default_for(crate::version(), &[]);
        let mut lock = Lock::default();
        // An unmodified leftover and a user-edited one, under no live claim.
        std::fs::write(dir.path().join("stale.txt"), "superdev's").unwrap();
        lock.files
            .insert("stale.txt".into(), sha256_hex(b"superdev's"));
        std::fs::write(dir.path().join("theirs.txt"), "edited").unwrap();
        lock.files
            .insert("theirs.txt".into(), sha256_hex(b"superdev's"));
        let fake = FakeRunner::new();
        let plan = plan_repo(dir.path(), &fake, &manifest, &lock, PlanMode::Sync).unwrap();
        let last = plan.planned().last().unwrap();
        assert!(last.capability.is_none());
        assert!(
            last.actions
                .iter()
                .any(|a| a.describe().contains("remove stale.txt")),
            "{:?}",
            last.actions
        );
        assert_eq!(plan.released_lines().len(), 1);
        assert!(plan.released_lines()[0].contains("theirs.txt"));
    }

    #[test]
    fn the_blueprint_line_reports_only_a_difference() {
        let mut manifest = Manifest::default_for(crate::version(), &[]);
        assert_eq!(blueprint_line(&manifest), None);
        manifest.blueprint = "0.0.1".into();
        assert_eq!(
            blueprint_line(&manifest),
            Some(format!(
                "blueprint 0.0.1, binary {} — sync will update it",
                crate::version()
            ))
        );
    }

    #[test]
    fn stamping_rewrites_only_a_stale_blueprint() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.0.1", &[]);
        manifest.save(dir.path()).unwrap();
        stamp_blueprint(dir.path(), &manifest).unwrap();
        let stamped = Manifest::load(dir.path()).unwrap();
        assert_eq!(stamped.blueprint, crate::version());
        // Already current: the file is left untouched. Marked with a comment
        // `save` would drop, since mtime here cannot resolve two writes a
        // microsecond apart.
        let path = dir.path().join(crate::manifest::CONFIG_PATH);
        let before = format!("{}# untouched\n", std::fs::read_to_string(&path).unwrap());
        std::fs::write(&path, &before).unwrap();
        stamp_blueprint(dir.path(), &stamped).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(before, after, "no rewrite when the value is current");
    }
}
