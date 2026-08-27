# Adhoc Plans

* [Workflow autonomy — branch, slice dependencies, unattended delivery](P004-workflow-autonomy.md) - give the workflow a branch at frame, model slice dependencies in the plan, run stages 4-7 unattended on a general superdev run facility with a Stop hook and a new execute-feature-plan skill, and commit at every successful integrate.
* [Fold the superdev-format validator into the Rust validator](P006-rust-format-validator.md) - the grammar-driven format validator moves from a Node script into superdev-core and merges with the AOKF validator behind one command, one report and one hook, proved against goldens captured from the reference while it still runs.
* [Drop the AOKF conformance ladder](P007-drop-the-aokf-conformance-ladder.md) - ADR-017 in code — the three-level ladder leaves the spec, the validator, the CLI and the parity goldens, a bundle passes or fails, and no file in the tree names a level but the ADR and this plan.
