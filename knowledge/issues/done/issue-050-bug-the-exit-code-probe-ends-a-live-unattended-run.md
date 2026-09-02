---
type: BugReport
id: issue-050-bug-the-exit-code-probe-ends-a-live-unattended-run
title: The exit-code drift test probes `superdev run end` in the real repository, so a full test run ends any live unattended run
description: The exit-code drift test runs every probe with the repository root as its working directory, and `run end` removes `.superdev/cache/run.toml` there; the unattended driver's build phase runs the suite, so the driver ends its own run on the first slice.
lifecycle: done
links:
  - rel: references
    to: issue-026-chore-rehearse-the-driver-on-a-real-feature
    note: Found by the rehearsal that chore asked for, on plan-024's first slice.
---

# Bug: the exit-code probe ends a live unattended run

## Resolved

Fixed 2026-09-02, the same turn it was found: `run()` in
`contract_exit_codes.rs` gives the `run` verbs a scratch directory.
Proved by arming a run, running the exit-code suite, and finding the
run state still there.

## Summary

Found by the rehearsal
[I026][sokf:issue-026-chore-rehearse-the-driver-on-a-real-feature]
asked for, on plan-024's first slice. `superdev run end` is probed for its exit code with the repository
root as the working directory. It removes the run state it finds
there. A build phase runs the test suite, so an unattended run ends
itself the first time a slice's checks run.

## Environment

- This repository, any platform.
- `crates/app/superdev/tests/contract_exit_codes.rs`, `run()`, which
  sets `current_dir(REPO_ROOT)` for every probe.

## Steps to reproduce

1. `RS_c1` `superdev run begin --session x --next y` — `.superdev/cache/run.toml`
   exists.
2. `RS_c2` `cargo nextest run -p superdev --test contract_exit_codes`.
3. `RS_c3` `.superdev/cache/run.toml` is gone; `superdev run advance` reports
   `no run is active`.

## Expected behaviour

A test probes exit codes; it does not change the repository's run
state.

## Actual behaviour

The run state is removed, and the driver's next `run advance` reports
`no run is active`.

## Root cause (if known)

`run()` uses one working directory for every probe because most probes
need the repository — `validate` reads the knowledge. The `run` verbs
do not, and `end` writes.

## Proposed fix / workaround

- `run()` gives the `run` verbs' probes a scratch directory, since
  they act on the working directory and need nothing from the
  repository, and every other probe the repository root.

## Regression risk

Low: the probe list is the same; two probes move to a scratch
directory where `end` finds nothing and returns 0, and `advance` finds
nothing and returns 2, which are the codes the contract declares.

<!-- sokf:links -->
[sokf:issue-026-chore-rehearse-the-driver-on-a-real-feature]: /knowledge/issues/open/issue-026-chore-rehearse-the-driver-on-a-real-feature.md
