# Plans

Implementation plans, one document per piece of work, numbered in one
series. A plan states its goal, the contract changes it makes and the
work blocks that deliver it; scope writes it and build works it.

## Plans

* [Flatten the superdev-core API][sokf:plan-001-flatten-crate-api] - apply the module flatten rule to superdev-core — private submodules, pub use re-exports at lib.rs, callers writing crate::Item.
* [Agent Instructions Layer][sokf:plan-002-agent-instructions-layer] - deliver S010 — the user-owned AGENTS.md with one ensured import, the fenced superdev.md aggregator, per-capability instruction files, codegraph MCP wiring, and the code-index dogfood.
* [Externally Sourced Content Packs][sokf:plan-003-content-packs] - deliver S014 in eighteen slices — move the content to /pack, reorganise it into pack layout, derive it from that layout, add the manifest and lock schemas, resolve local then git sources, wire ownership, teach init and update, make each release one command, make a committed path pin portable, dogfood it, then close the gaps acceptance found and the one deferred issue small enough to fix.
* [Workflow autonomy — branch, slice dependencies, unattended delivery][sokf:plan-004-workflow-autonomy] - give the workflow a branch at frame, model slice dependencies in the plan, run stages 4-7 unattended on a general superdev run facility with a Stop hook and a new execute-feature-plan skill, and commit at every successful integrate.
* [Content pack hardening][sokf:plan-005-content-pack-hardening] - deliver ADR-012 to ADR-016 in seven slices — refuse an unsupported transport, refuse a symlink in a pack and let git decide what one is, give the spawn seam a deadline and an environment, bound the one unprompted request, prove a pin before writing it, and stop recording a digest nothing reads.
* [Fold the superdev-format validator into the Rust validator][sokf:plan-006-rust-format-validator] - the grammar-driven format validator moves from a Node script into superdev-core and merges with the AOKF validator behind one command, one report and one hook, proved against goldens captured from the reference while it still ran.
* [Drop the AOKF conformance ladder][sokf:plan-007-drop-the-aokf-conformance-ladder] - ADR-017 in code — the three-level ladder leaves the spec, the validator, the CLI and the parity goldens, knowledge passes or fails, and no file in the tree names a level but the ADR and this plan.
* [SOKF becomes a core part of superdev][sokf:plan-008-sokf-becomes-core] - AOKF is renamed SOKF and stops being a swappable capability, the two validators merge into one module behind one command, a document's type names the schema that governs it, and the schema layer is enforced for the first time.
* [Drop rtk and the bash-output-filter capability][sokf:plan-009-drop-the-bash-output-filter] - the bash-output-filter slot, its rtk provider, the five things it owns and the flag that disabled it all leave, and a manifest still naming the table gets a guided error.
* [Links address ids][sokf:plan-010-links-address-ids] - SOKF 0.4 gives a body link an id-addressed form, superdev validate --fix converts the tree to it, and a renamed or moved concept stops breaking the documents that cite it.
* [Documents are filed by lifecycle][sokf:plan-011-filing-by-lifecycle] - one lifecycle field replaces two vocabularies, every document sits in a folder named for its state, and a document left in the base directory is unfiled — an error the fix pass repairs.
* [The workflow becomes contract-driven][sokf:plan-012-contract-driven-workflow] - the seven-phase spec-driven workflow becomes five contract-driven phases — criteria move into the feature-request as EARS sentences, contracts become durable in public/ and internal/, the spec documents are migrated and deleted, and the skills merge to match.
* [Workflow autonomy][sokf:plan-013-workflow-autonomy] - eight slices delivering the unattended workflow — the run state and its verbs, the Stop hook, the managed hook entry, the plan format's dependencies, the branching and commit conventions, the driver skill, and the records.
* [Bring every schema in line with its own rules and the workflow][sokf:plan-014-schema-review-findings] - the schema review's findings land — worked examples satisfy their own schemas, the report schemas gain identity and filing, stale vocabulary leaves, the contract, ADR and idea shapes unify, and the pack mirror stays byte-identical.
* [Integrate runs /code-review once, at the last slice][sokf:plan-015-code-review-at-the-last-slice] - the per-slice /code-review in integrate becomes one feature-wide review at the last slice, over the whole diff, with findings returning to build as today.
* [Schema layer enforcement][sokf:plan-016-schema-layer-enforcement] - three slices making the validator read what the schemas declare — content kinds, the frontmatter contract, and the required-key vocabulary — each landing with the reconciliation it surfaces.
* [Example conformance][sokf:plan-017-example-conformance] - two slices making validate check each schema's example against the schema that declares it — the document check in place, then link form without resolution — each landing with the reconciliation it surfaces.
* [Validate path dispatch][sokf:plan-018-validate-path-dispatch] - two slices making validate check a named file as what it is — the schema half reaches a named path first, then the grammar half stops misreading documents and parity is proved end to end.
* [Contract-design review and the binding-surface standard][sokf:plan-019-contract-design-review] - five slices: the include mechanism, the standard carried into the 15 contract schemas, the skill's explicit go-ahead, and the nine-contract sweep in two passes.
* [Normative shape enforcement][sokf:plan-020-normative-shape-enforcement] - slices delivering the body-pattern vocabulary, the EARS declaration, the contract-kind declarations and the contract sweep.
* [Contracts define their interfaces][sokf:plan-021-contracts-define-their-interface] - slices delivering the definition-block vocabulary, each kind's declared form, the drift tests that bind a contract to its implementation, and the split of the file-format kind.
* [A decidable finding is an error][sokf:plan-022-decidable-findings-are-errors] - slices closing the promised run-state fields, promoting the five findings the repository alone settles, scoping the edit-time hook off the two that span files, and holding the turn open while the knowledge carries an error.
* [A warning is counted by default and listed on request][sokf:plan-023-warnings-are-counted-not-listed] - slices adding the `--warnings` flag the contract promises, carrying both counts into `--json` alongside the findings it lists, and giving the two hooks the same default as the command line.
* [A contract includes its definition][sokf:plan-024-a-contract-includes-its-definition] - slices delivering I049 — the source include, the sixth content kind, schema variants, the one contract schema, the skills' judgement and declaration steps, the migration of nine contracts, and the deletion of fifteen schemas and four copy-comparing tests.
* [A contract's behaviour is written as EARS][sokf:plan-025-a-contracts-behaviour-is-written-as-ears] - slices delivering I037 — the three item declarations in the validator, the sweep of nine contracts to keyed EARS promises, the contract schema in its final form with twelve examples, the tracker schemas' keyed criteria with the c<n> sweep of fifty issues, and the records.
* [Filing an issue without framing it][sokf:plan-026-filing-an-issue-without-framing-it] - slices delivering I030 — a heading declared per variant in the validator, the tracker schemas varying by a four-state lifecycle with the sweep of the issues on file, the /file skill and the workflow entry, /frame framing in place with the three phases' gates, the backlog's retirement, and the records.
* [The workflow is file, scope, build, accept][sokf:plan-027-the-workflow-is-file-scope-build-accept] - slices delivering I052 — the validator's nested items and optional key closing contract-010's five PENDING promises, a contract's nested criteria, one issue schema with the sweep of the issues on file, one plan schema with the sweep of the plans, the scope and contract-design skills, the build, execute-plan and accept skills with the workflow text, and the concepts and records.

<!-- sokf:links -->
[sokf:plan-001-flatten-crate-api]: /knowledge/plans/open/plan-001-flatten-crate-api.md
[sokf:plan-002-agent-instructions-layer]: /knowledge/plans/done/plan-002-agent-instructions-layer.md
[sokf:plan-003-content-packs]: /knowledge/plans/done/plan-003-content-packs.md
[sokf:plan-004-workflow-autonomy]: /knowledge/plans/done/plan-004-workflow-autonomy.md
[sokf:plan-005-content-pack-hardening]: /knowledge/plans/done/plan-005-content-pack-hardening.md
[sokf:plan-006-rust-format-validator]: /knowledge/plans/done/plan-006-rust-format-validator.md
[sokf:plan-007-drop-the-aokf-conformance-ladder]: /knowledge/plans/done/plan-007-drop-the-aokf-conformance-ladder.md
[sokf:plan-008-sokf-becomes-core]: /knowledge/plans/done/plan-008-sokf-becomes-core.md
[sokf:plan-009-drop-the-bash-output-filter]: /knowledge/plans/done/plan-009-drop-the-bash-output-filter.md
[sokf:plan-010-links-address-ids]: /knowledge/plans/done/plan-010-links-address-ids.md
[sokf:plan-011-filing-by-lifecycle]: /knowledge/plans/done/plan-011-filing-by-lifecycle.md
[sokf:plan-012-contract-driven-workflow]: /knowledge/plans/done/plan-012-contract-driven-workflow.md
[sokf:plan-013-workflow-autonomy]: /knowledge/plans/done/plan-013-workflow-autonomy.md
[sokf:plan-014-schema-review-findings]: /knowledge/plans/done/plan-014-schema-review-findings.md
[sokf:plan-015-code-review-at-the-last-slice]: /knowledge/plans/done/plan-015-code-review-at-the-last-slice.md
[sokf:plan-016-schema-layer-enforcement]: /knowledge/plans/done/plan-016-schema-layer-enforcement.md
[sokf:plan-017-example-conformance]: /knowledge/plans/done/plan-017-example-conformance.md
[sokf:plan-018-validate-path-dispatch]: /knowledge/plans/done/plan-018-validate-path-dispatch.md
[sokf:plan-019-contract-design-review]: /knowledge/plans/done/plan-019-contract-design-review.md
[sokf:plan-020-normative-shape-enforcement]: /knowledge/plans/done/plan-020-normative-shape-enforcement.md
[sokf:plan-021-contracts-define-their-interface]: /knowledge/plans/done/plan-021-contracts-define-their-interface.md
[sokf:plan-022-decidable-findings-are-errors]: /knowledge/plans/done/plan-022-decidable-findings-are-errors.md
[sokf:plan-023-warnings-are-counted-not-listed]: /knowledge/plans/done/plan-023-warnings-are-counted-not-listed.md
[sokf:plan-024-a-contract-includes-its-definition]: /knowledge/plans/done/plan-024-a-contract-includes-its-definition.md
[sokf:plan-025-a-contracts-behaviour-is-written-as-ears]: /knowledge/plans/done/plan-025-a-contracts-behaviour-is-written-as-ears.md
[sokf:plan-026-filing-an-issue-without-framing-it]: /knowledge/plans/done/plan-026-filing-an-issue-without-framing-it.md
[sokf:plan-027-the-workflow-is-file-scope-build-accept]: /knowledge/plans/open/plan-027-the-workflow-is-file-scope-build-accept.md
