# Plans

Implementation plans, numbered in one series across both kinds so a
number names one plan. A feature plan delivers a framed feature as a
slice list; an ad-hoc plan covers one-off work outside the feature
workflow.

## Feature plans

* [Flatten the superdev-core API][sokf:plan-001-feature-flatten-crate-api] - apply the module flatten rule to superdev-core — private submodules, pub use re-exports at lib.rs, callers writing crate::Item.
* [Agent Instructions Layer][sokf:plan-002-feature-agent-instructions-layer] - deliver S010 — the user-owned AGENTS.md with one ensured import, the fenced superdev.md aggregator, per-capability instruction files, codegraph MCP wiring, and the code-index dogfood.
* [Externally Sourced Content Packs — feature plan][sokf:plan-003-feature-content-packs] - deliver S014 in eighteen slices — move the content to /pack, reorganise it into pack layout, derive it from that layout, add the manifest and lock schemas, resolve local then git sources, wire ownership, teach init and update, make each release one command, make a committed path pin portable, dogfood it, then close the gaps acceptance found and the one deferred issue small enough to fix.
* [Content pack hardening — feature plan][sokf:plan-005-feature-content-pack-hardening] - deliver ADR-012 to ADR-016 in seven slices — refuse an unsupported transport, refuse a symlink in a pack and let git decide what one is, give the spawn seam a deadline and an environment, bound the one unprompted request, prove a pin before writing it, and stop recording a digest nothing reads.
* [Workflow autonomy][sokf:plan-013-feature-workflow-autonomy] - eight slices delivering the unattended workflow — the run state and its verbs, the Stop hook, the managed hook entry, the plan format's dependencies, the branching and commit conventions, the driver skill, and the records.
* [Schema layer enforcement — feature plan][sokf:plan-016-feature-schema-layer-enforcement] - three slices making the validator read what the schemas declare — content kinds, the frontmatter contract, and the required-key vocabulary — each landing with the reconciliation it surfaces.
* [Example conformance — feature plan][sokf:plan-017-feature-example-conformance] - two slices making validate check each schema's example against the schema that declares it — the document check in place, then link form without resolution — each landing with the reconciliation it surfaces.
* [Validate path dispatch — feature plan][sokf:plan-018-feature-validate-path-dispatch] - two slices making validate check a named file as what it is — the schema half reaches a named path first, then the grammar half stops misreading documents and parity is proved end to end.

## Ad-hoc plans

* [Workflow autonomy — branch, slice dependencies, unattended delivery][sokf:plan-004-adhoc-workflow-autonomy] - give the workflow a branch at frame, model slice dependencies in the plan, run stages 4-7 unattended on a general superdev run facility with a Stop hook and a new execute-feature-plan skill, and commit at every successful integrate.
* [Fold the superdev-format validator into the Rust validator][sokf:plan-006-adhoc-rust-format-validator] - the grammar-driven format validator moves from a Node script into superdev-core and merges with the AOKF validator behind one command, one report and one hook, proved against goldens captured from the reference while it still ran.
* [Drop the AOKF conformance ladder][sokf:plan-007-adhoc-drop-the-aokf-conformance-ladder] - ADR-017 in code — the three-level ladder leaves the spec, the validator, the CLI and the parity goldens, knowledge passes or fails, and no file in the tree names a level but the ADR and this plan.
* [SOKF becomes a core part of superdev][sokf:plan-008-adhoc-sokf-becomes-core] - AOKF is renamed SOKF and stops being a swappable capability, the two validators merge into one module behind one command, a document's type names the schema that governs it, and the schema layer is enforced for the first time.
* [Drop rtk and the bash-output-filter capability][sokf:plan-009-adhoc-drop-the-bash-output-filter] - the bash-output-filter slot, its rtk provider, the five things it owns and the flag that disabled it all leave, and a manifest still naming the table gets a guided error.
* [Links address ids][sokf:plan-010-adhoc-links-address-ids] - SOKF 0.4 gives a body link an id-addressed form, superdev validate --fix converts the tree to it, and a renamed or moved concept stops breaking the documents that cite it.
* [Documents are filed by lifecycle][sokf:plan-011-adhoc-filing-by-lifecycle] - one lifecycle field replaces two vocabularies, every document sits in a folder named for its state, and a document left in the base directory is unfiled — an error the fix pass repairs.
* [The workflow becomes contract-driven][sokf:plan-012-adhoc-contract-driven-workflow] - the seven-phase spec-driven workflow becomes five contract-driven phases — criteria move into the feature-request as EARS sentences, contracts become durable in public/ and internal/, the spec documents are migrated and deleted, and the skills merge to match.
* [Bring every schema in line with its own rules and the workflow][sokf:plan-014-adhoc-schema-review-findings] - the schema review's findings land — worked examples satisfy their own schemas, the report schemas gain identity and filing, stale vocabulary leaves, the contract, ADR and idea shapes unify, and the pack mirror stays byte-identical.
* [Integrate runs /code-review once, at the last slice][sokf:plan-015-adhoc-code-review-at-the-last-slice] - the per-slice /code-review in integrate becomes one feature-wide review at the last slice, over the whole diff, with findings returning to build as today.

<!-- sokf:links -->
[sokf:plan-001-feature-flatten-crate-api]: /knowledge/plans/open/plan-001-feature-flatten-crate-api.md
[sokf:plan-002-feature-agent-instructions-layer]: /knowledge/plans/done/plan-002-feature-agent-instructions-layer.md
[sokf:plan-003-feature-content-packs]: /knowledge/plans/done/plan-003-feature-content-packs.md
[sokf:plan-004-adhoc-workflow-autonomy]: /knowledge/plans/done/plan-004-adhoc-workflow-autonomy.md
[sokf:plan-005-feature-content-pack-hardening]: /knowledge/plans/done/plan-005-feature-content-pack-hardening.md
[sokf:plan-006-adhoc-rust-format-validator]: /knowledge/plans/done/plan-006-adhoc-rust-format-validator.md
[sokf:plan-007-adhoc-drop-the-aokf-conformance-ladder]: /knowledge/plans/done/plan-007-adhoc-drop-the-aokf-conformance-ladder.md
[sokf:plan-008-adhoc-sokf-becomes-core]: /knowledge/plans/done/plan-008-adhoc-sokf-becomes-core.md
[sokf:plan-009-adhoc-drop-the-bash-output-filter]: /knowledge/plans/done/plan-009-adhoc-drop-the-bash-output-filter.md
[sokf:plan-010-adhoc-links-address-ids]: /knowledge/plans/done/plan-010-adhoc-links-address-ids.md
[sokf:plan-011-adhoc-filing-by-lifecycle]: /knowledge/plans/done/plan-011-adhoc-filing-by-lifecycle.md
[sokf:plan-012-adhoc-contract-driven-workflow]: /knowledge/plans/done/plan-012-adhoc-contract-driven-workflow.md
[sokf:plan-013-feature-workflow-autonomy]: /knowledge/plans/done/plan-013-feature-workflow-autonomy.md
[sokf:plan-014-adhoc-schema-review-findings]: /knowledge/plans/done/plan-014-adhoc-schema-review-findings.md
[sokf:plan-015-adhoc-code-review-at-the-last-slice]: /knowledge/plans/done/plan-015-adhoc-code-review-at-the-last-slice.md
[sokf:plan-016-feature-schema-layer-enforcement]: /knowledge/plans/done/plan-016-feature-schema-layer-enforcement.md
[sokf:plan-017-feature-example-conformance]: /knowledge/plans/done/plan-017-feature-example-conformance.md
[sokf:plan-018-feature-validate-path-dispatch]: /knowledge/plans/done/plan-018-feature-validate-path-dispatch.md
