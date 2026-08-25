# Process: Diagnosing CI failures

## 1. Read the actual failure

- Open the real CI logs (e.g. `gh run view --log-failed`) and find the first error, not the last — later failures are usually cascade noise.
- Identify which job/step failed and on what matrix entry (OS, runtime version) — a single-entry failure points at environment, an all-entry failure points at the change.

## 2. Classify before fixing

- **Real failure:** the change genuinely breaks something. → Standard bug-fix process (`bug-fix.md`).
- **Environment difference:** passes locally, fails in CI — different OS, missing env var/service, stricter lockfile install, case-sensitive filesystem, timezone/locale.
- **Flake:** intermittent, unrelated to the diff — timing-dependent test, network dependence, shared-state pollution, resource limits.
- **Infrastructure:** runner outage, registry timeout, quota. Not yours to fix; rerun and note it.

## 3. Reproduce under CI's conditions, not yours

- Match what CI does: its exact commands from the workflow file, a clean install from the lockfile, the same runtime version, CI-set env vars.
- If it still won't reproduce locally, extract evidence from CI itself — add a diagnostic step or upload artifacts — rather than pushing guess-fixes repeatedly.

## 4. Never retry-until-green as a fix

- A rerun is a diagnostic (does it fail the same way twice?), not a resolution. A flake that's rerun into passing stays in the suite and taxes everyone.
- When a flake is identified: fix the test's real defect (await the condition instead of sleeping, isolate shared state, control time/randomness). Skip-and-ticket only with the user's agreement, never silently.

## 5. Verify the fix where it failed

- Push the fix and confirm the same CI job passes; local green was already true for environment failures, so only CI green closes the loop.

## 6. Report

- What failed, the classification with evidence, the fix, and the CI run link showing green. If it was infrastructure or an unresolved flake, say exactly that rather than implying the code was fixed.
