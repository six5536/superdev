# Plans

Implementation plans, numbered in one series across both kinds so a
number names one plan. `plan-<nnn>-feature-<slug>.md` delivers a spec as a
slice list; `plan-<nnn>-adhoc-<slug>.md` covers one-off work outside the
feature workflow.

## Feature plans

* [Flatten the superdev-core API](plan-001-feature-flatten-crate-api.md) - apply the module flatten rule to superdev-core — private submodules, pub use re-exports at lib.rs, callers writing crate::Item.
* [Agent Instructions Layer](plan-002-feature-agent-instructions-layer.md) - deliver S010 — the user-owned AGENTS.md with one ensured import, the fenced superdev.md aggregator, per-capability instruction files, codegraph MCP wiring, and the code-index dogfood.
* [Externally Sourced Content Packs — feature plan](plan-003-feature-content-packs.md) - deliver S014 in eighteen slices — move the content to /pack, reorganise it into pack layout, derive it from that layout, add the manifest and lock schemas, resolve local then git sources, wire ownership, teach init and update, make each release one command, make a committed path pin portable, dogfood it, then close the gaps acceptance found and the one deferred issue small enough to fix.
* [Content pack hardening — feature plan](plan-005-feature-content-pack-hardening.md) - deliver ADR-012 to ADR-016 in seven slices — refuse an unsupported transport, refuse a symlink in a pack and let git decide what one is, give the spawn seam a deadline and an environment, bound the one unprompted request, prove a pin before writing it, and stop recording a digest nothing reads.

## Ad-hoc plans

* [Workflow autonomy — branch, slice dependencies, unattended delivery](plan-004-adhoc-workflow-autonomy.md) - give the workflow a branch at frame, model slice dependencies in the plan, run stages 4-7 unattended on a general superdev run facility with a Stop hook and a new execute-feature-plan skill, and commit at every successful integrate.
* [Fold the superdev-format validator into the Rust validator](plan-006-adhoc-rust-format-validator.md) - the grammar-driven format validator moves from a Node script into superdev-core and merges with the AOKF validator behind one command, one report and one hook, proved against goldens captured from the reference while it still ran.
* [Drop the AOKF conformance ladder](plan-007-adhoc-drop-the-aokf-conformance-ladder.md) - ADR-017 in code — the three-level ladder leaves the spec, the validator, the CLI and the parity goldens, knowledge passes or fails, and no file in the tree names a level but the ADR and this plan.
* [SOKF becomes a core part of superdev](plan-008-adhoc-sokf-becomes-core.md) - AOKF is renamed SOKF and stops being a swappable capability, the two validators merge into one module behind one command, a document's type names the schema that governs it, and the schema layer is enforced for the first time.
* [Drop rtk and the bash-output-filter capability](plan-009-adhoc-drop-the-bash-output-filter.md) - the bash-output-filter slot, its rtk provider, the five things it owns and the flag that disabled it all leave, and a manifest still naming the table gets a guided error.
