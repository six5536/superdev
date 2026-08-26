# Plans

* [Flatten the superdev-core API](P001-flatten-crate-api.md) - apply the module flatten rule to superdev-core — private submodules, pub use re-exports at lib.rs, callers writing crate::Item.
* [Agent Instructions Layer](P002-agent-instructions-layer.md) - deliver S010 — the user-owned AGENTS.md with one ensured import, the fenced superdev.md aggregator, per-capability instruction files, codegraph MCP wiring, and the code-index dogfood.
* [Externally Sourced Content Packs — feature plan](P003-content-packs.md) - deliver S014 in fourteen slices — move the content to /pack, reorganise it into pack layout, derive it from that layout, add the manifest and lock schemas, resolve local then git sources, wire ownership, teach init and update, make each release one command, make a committed path pin portable, and dogfood it.
* [Workflow autonomy — branch, slice dependencies, unattended delivery](P004-workflow-autonomy.md) - give the workflow a branch at frame, model slice dependencies in the plan, run stages 4-7 unattended behind a Stop hook and a new execute-feature-plan skill, and commit at every successful integrate.
