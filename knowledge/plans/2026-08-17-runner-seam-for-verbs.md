---
type: Plan
id: plan-runner-seam
title: Runner Seam for the Verbs
description: Orchestration tests move to core through the pipeline's CommandRunner seam; the unix-only shell-fake sandbox shrinks to four smoke journeys.
status: draft
---

From the 2026-08-17 architecture review, candidate 5 (Strong). Completes
the verb track after the
[verb pipeline](2026-08-17-verb-pipeline-in-core.md).

# Friction

Core has exactly the right seam — `CommandRunner` (`runner.rs:21`) with a
`FakeRunner` adapter — but the fake is `#[cfg(test)] pub(crate)`
(`runner.rs:60`) and the verbs hardwire `SystemRunner`
(`manage.rs:122/151/190`). So orchestration behaviour (trust-before-
install ordering, targeted install lists, plugin vs materialise flow) is
tested past the interface: a 190-line `Sandbox`
(`tests/manage.rs:13–204`) writes fake `mise`/`claude`/`codegraph` shell
scripts with a four-env-var protocol, `#![cfg(unix)]` — Windows loses
the whole suite — and `manage.rs:725` re-invents a `QuietRunner` because
the existing adapter is unreachable. One adapter is a hypothetical seam;
this makes it two, and real.

# Design (settled by grilling, 2026-08-17)

Narrower than first drafted. The app's e2e tests spawn the real binary,
so no in-process runner can reach them — the seam pays off only in
core, which the [verb pipeline](2026-08-17-verb-pipeline-in-core.md)
delivers by taking `&dyn CommandRunner`. No `test-util` export:
`FakeRunner` stays crate-internal (it already answers `mise where`
with a scripted fixture path); revisit only if an app-side in-process
test ever materialises.

# Tasks

1. Port the Sandbox tests that assert orchestration (call ordering,
   targeted install lists, plugin vs materialise flow, provider
   switching) to core pipeline tests against the crate-internal
   `FakeRunner` — e.g. `sync_installs_committed_pins_on_a_fresh_clone`
   (`tests/manage.rs:421`) already has a 40-line twin at
   `engine.rs:1288`; keep one of each pair. (`QuietRunner` is already
   gone — the [verb pipeline](2026-08-17-verb-pipeline-in-core.md)
   deletes it when its tests move.)
2. Shrink `tests/manage.rs` to four smoke journeys of the real
   `SystemRunner` wiring (cap 5, file stays `#![cfg(unix)]`):
   init happy path with hints; fresh-clone sync installs committed
   pins; workflows provider switch e2e; orphan sweep e2e.

# Done

Orchestration tests run on Windows. `tests/manage.rs` keeps ≤5 sandbox
tests. No test greps `FAKE_LOG` for behaviour a `FakeRunner` test also
asserts. `npm test` passes on the CI matrix.

# Sequencing

Strictly after the
[verb pipeline](2026-08-17-verb-pipeline-in-core.md), which introduces
the runner-parameterised entries this plan tests through. Delete this
file in the commit that completes the work.
