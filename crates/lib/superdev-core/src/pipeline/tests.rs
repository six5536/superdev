use super::*;
use crate::{lock::sha256_hex, runner::FakeRunner};

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

fn materialize_pi_repo_assets(root: &Path) {
    for (kind, directory) in [
        (ItemKind::PiExtension, ".pi/extensions"),
        (ItemKind::PiSkill, ".pi/skills"),
    ] {
        for item in content::test_snapshot().items_of(Owner::Repo, kind) {
            for (relative, body) in &item.files {
                let path = root.join(directory).join(&item.name).join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, body).unwrap();
            }
        }
    }
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
    std::fs::write(dir.path().join(".agents/superdev.md"), AGGREGATOR_TEMPLATE).unwrap();
    for scaffold in rule_scaffold_paths() {
        std::fs::write(dir.path().join(scaffold), "the user's now\n").unwrap();
    }
    std::fs::write(
        dir.path().join(".gitignore"),
        ".superdev/cache/\n.superdev/workflows/\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("AGENTS.md"),
        crate::agent_file::render("", AGGREGATOR_TEMPLATE).unwrap(),
    )
    .unwrap();
    materialize_pi_repo_assets(dir.path());
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
fn agent_instructions_match_canonical_source_for_every_enabled_set() {
    for disabled in [vec![], vec![Capability::CodeIndex]] {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &disabled);
        let entry = repo_entry(dir.path(), &manifest, content::test_snapshot())
            .unwrap()
            .unwrap();
        let written = entry
            .actions
            .iter()
            .find_map(|action| match action {
                Action::WriteFile { path, content, .. } if path == AGGREGATOR_PATH => Some(content),
                _ => None,
            })
            .expect("the canonical instructions are planned");
        assert_eq!(written.as_bytes(), AGGREGATOR_TEMPLATE.as_bytes());
    }
}

#[test]
fn repo_entry_plans_the_instruction_prefix_and_the_aggregator_once() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let entry = repo_entry(dir.path(), &manifest, content::test_snapshot())
        .unwrap()
        .unwrap();
    let descs: Vec<String> = entry.actions.iter().map(Action::describe).collect();
    assert!(
        descs
            .iter()
            .any(|d| d == "ensure AGENTS.md starts with superdev's instructions"),
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
        ".superdev/cache/\n.superdev/workflows/\n.codegraph/\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("AGENTS.md"),
        crate::agent_file::render("# Mine\n", AGGREGATOR_TEMPLATE).unwrap(),
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join(".agents")).unwrap();
    std::fs::write(dir.path().join(AGGREGATOR_PATH), AGGREGATOR_TEMPLATE).unwrap();
    for path in rule_scaffold_paths() {
        std::fs::write(dir.path().join(path), "adapted by the user\n").unwrap();
    }
    materialize_pi_repo_assets(dir.path());
    assert!(
        repo_entry(dir.path(), &manifest, content::test_snapshot())
            .unwrap()
            .is_none()
    );
}

#[test]
fn repo_entry_plans_the_complete_pi_extension() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Manifest::default_for(crate::version(), &[]);
    let entry = repo_entry(dir.path(), &manifest, content::test_snapshot())
        .unwrap()
        .unwrap();
    let descs: Vec<String> = entry.actions.iter().map(Action::describe).collect();
    let extension = content::test_snapshot()
        .item(Owner::Repo, ItemKind::PiExtension, "superdev")
        .unwrap();
    for (relative, _) in &extension.files {
        let path = format!(".pi/extensions/superdev/{relative}");
        assert!(descs.iter().any(|line| line.contains(&path)), "{path}");
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
