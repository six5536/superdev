---
type: Plan
id: plan-runner-seam
title: Runner Seam for the Verbs
description: Verbs take a CommandRunner; FakeRunner is exported and the unix-only shell-fake sandbox shrinks to smoke tests.
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

# Tasks

1. `plan_repo`/`apply_repo` (or the verbs, if landed before the
   pipeline) take `&dyn CommandRunner`; `main.rs` passes `SystemRunner`.
2. Export `FakeRunner` behind a `test-util` cargo feature on
   `superdev-core`; the app crate enables it as a dev-dependency
   feature. Delete `QuietRunner`.
3. Port the Sandbox tests that assert orchestration (call ordering,
   install lists, provider switching) to cross-platform unit tests
   against `FakeRunner` — e.g. `sync_installs_committed_pins_on_a_fresh_
   clone` (`tests/manage.rs:421`) already has a 40-line `FakeRunner`
   twin at `engine.rs:1288`; keep one of the pair.
4. Shrink `tests/manage.rs` to a handful of true end-to-end smoke tests
   of the real `SystemRunner` wiring (these stay unix-only; everything
   else runs everywhere, including the materialise fixture flow, whose
   `mise where` answer comes from the fake).

# Done

Verb orchestration tests run on Windows. `tests/manage.rs` keeps ≤5
sandbox tests. No test greps `FAKE_LOG` for behaviour a `FakeRunner`
test also asserts. `npm test` passes on the CI matrix.

# Sequencing

After the [verb pipeline](2026-08-17-verb-pipeline-in-core.md); the
seam-threading is trivial once the pipeline exists. Delete this file in
the commit that completes the work.
