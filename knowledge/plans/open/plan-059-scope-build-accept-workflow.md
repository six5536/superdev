---
type: Plan
id: plan-059-scope-build-accept-workflow
title: Scope build accept workflow
description: Replace contradictory agent workflows with one durable Rust-owned SCOPE to BUILD to ACCEPT process orchestrated by Pi.
lifecycle: open
phase: build
branch: work/059-scope-build-accept-workflow
links:
  - rel: implements
    to: issue-059-scope-build-accept-workflow
---

# Plan: Scope build accept workflow

## Goal and boundaries

Implement the approved root specification in `SCOPE-BUILD-ACCEPT-IMPLEMENTATION-PLAN.md` for [issue 059][sokf:issue-059-scope-build-accept-workflow]. SOKF records under `knowledge/` are canonical, Rust owns durable workflow state and safe Git operations, and Pi owns orchestration and interactive UI. `/file` remains independent. Remote CI, push, release, deployment, and branch deletion are excluded.

## Requirements

The only phases are SCOPE, BUILD, and ACCEPT. One issue, one plan, and one matching work branch retain stable identity across re-scope. Scope approval, configured acceptance, and abandonment require interactive human authority that an isolated child or direct transition flag cannot forge. Non-human gates derive from candidate-bound typed results and repository state. Cancellation pauses and releases ownership. Rejection preserves human feedback on the primary issue and returns the same plan to SCOPE. Acceptance integrates locally through a shell-free no-fast-forward merge without absorbing unrelated work.

## Contract changes

- `contract-011-interface-workflow`: implement all workflow authority, transition, evidence, retry, cancellation, rejection, closure, cache, and Git safety promises.
- `contract-002-cli-superdev`: replace the old run protocol with the typed workflow and filing command surfaces and their exact exit behaviour.
- `contract-003-api-sokf`: expose shared SOKF services used by workflow record transactions.
- `contract-009-interface-run-state`: retire the superseded run-state interface.

## ADR decisions

- `adr-052-the-workflow-is-scope-build-accept-under-a-durable-core` defines the active workflow decision and supersedes the contradictory workflow decisions.
- `adr-042-a-contracts-definition-is-materialized-from-source` continues to govern generated interface definitions.

## Source and interface changes

Implement workflow policy, state, transitions, cache transactions, filing, and shell-free Git in `crates/lib/superdev-core/src/workflow/`; expose only typed commands through `crates/app/superdev/src/workflow_cli.rs`; and materialize contract definitions from their Rust sources. Implement deterministic orchestration, isolated roles, UI decisions, lifecycle protection, and tool preflight in `.pi/extensions/superdev/`, then synchronize the owned extension into `pack/pi/extensions/superdev/`.

## Knowledge changes

Install strict Issue, Plan, and Documentation schemas; migrate historical records without inventing evidence; maintain ADR-052 and affected contracts; retire superseded workflow knowledge; maintain current indexes; and archive Claude workflow assets as inactive history.

## Documentation changes

The canonical-knowledge surface covers ADRs, contracts, schemas, procedures, configuration, architecture, security, and indexes; generate with `cargo run -- validate --fix` and verify with `cargo run -- validate --warnings`. The repository-handwritten surface covers `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, and the root bootstrap specification; verify with `npm run check:docs`. The generated contract-definition surface is owned by the relevant Rust blocks and checked by validation and sync drift. No website surface is declared for this repository.

## Work blocks

### Block 1: Install canonical workflow records and migration

- [x] Done.
- Dependencies: none.
- Areas: `knowledge/schemas/`, workflow ADRs and contracts, historical issues and plans, and `knowledge/documentation.md`.
- Outcome: canonical records encode phase ownership, executable evidence, documentation obligations, and uncertainty-preserving migration.
- Verification: `cargo run -- validate --warnings` and `cargo test -p superdev-core --test normative_shapes`.
- Tests: schema and normative-shape tests cover the Plan, Issue, and Documentation invariants; contract keys are cited in their owning contracts.
- Structural evidence: commits `b38646a`, `c83b4c7`, and `864baeb` contain the schema, migration, and repair work.
- Documentation: canonical-knowledge; run validation repair and validation.

### Block 2: Complete Rust-owned authority and atomic transitions

- [x] Done.
- Dependencies: Block 1.
- Areas: `crates/lib/superdev-core/src/workflow/`, `crates/app/superdev/src/workflow_cli.rs`, and contracts 002 and 011.
- Outcome: typed candidate-bound results, unforgeable human authority, service-owned approval/checkpoint/attestation/closure/reopening commits, phase ownership, atomic record publication and cache CAS, rejection, and abandonment are enforced by Rust.
- Verification: `cargo test -p superdev-core workflow` and `cargo test -p superdev --test manage`.
- Tests: transition, bypass, stale-state, publication rollback, rejection, closure, and integration tests cover contract-011 workflow promises.
- Structural evidence: Git invocations use validated argument arrays without a shell, and authority cannot be supplied as caller-controlled booleans.
- Documentation: generated contract definitions and canonical-knowledge; run `cargo run -- validate --fix` and validation.

### Block 3: Implement deterministic Pi orchestration

- [x] Done.
- Dependencies: Block 2.
- Areas: `.pi/extensions/superdev/`, private role prompts, Pi settings, and `pack/pi/extensions/superdev/`.
- Outcome: a typed phase state machine schedules bounded isolated roles, consumes structured terminal results, persists across compaction, protects session switch/fork, displays status, gates human decisions in UI, and preflights prohibited tool calls.
- Verification: `npm test` and `node --experimental-strip-types scripts/test/fixtures/superdev-extension-smoke.ts`.
- Tests: extension smoke and real-Pi journeys cover start, resume, cancellation, rejection, acceptance, and child restrictions for contract-011.
- Structural evidence: workflow commands call typed Rust operations directly rather than forwarding orchestration prose to the parent model.
- Documentation: repository-handwritten and canonical-knowledge; run `npm run check:docs` and validation.

### Block 4: Enforce BUILD checkpoints and bounded correction loops

- [x] Done.
- Dependencies: Blocks 2 and 3.
- Areas: Rust workflow services, Pi scheduler, plan evidence, and workflow configuration.
- Outcome: stable blocks execute separately with fingerprints, focused evidence, separate commits, configured stalled-attempt limits, full verification, immutable review, and bounded final correction cycles.
- Verification: `cargo test --workspace --all-targets` and `npm test`.
- Tests: retry, fingerprint, stale evidence, criterion coverage, documentation coverage, and correction-limit journeys cover contract-011 build and retry promises.
- Structural evidence: the candidate and reviewer result are bound to immutable Git revisions and distinct isolated sessions.
- Documentation: all declared surfaces affected by completed blocks; run each mapped generation and verification command.

### Block 5: Complete safe filing, packaging, archival, and migration surfaces

- [ ] Done.
- Dependencies: Blocks 2 and 3.
- Areas: filing service, pack manifests and assets, archive, launchers, indexes, configuration, and migration tests.
- Outcome: `/file` remains independently transactional and race-safe; Pi assets ship from first-class pack sources; Claude assets remain archived and inactive; configuration and indexes are current.
- Verification: `cargo test -p superdev --test manage`, `npm test`, `npm run check:docs`, and `cargo run -- sync --dry-run`.
- Tests: filing duplicates/concurrency/cleanup, pack synchronization, migration, and launcher tests cover affected contract promises.
- Structural evidence: automatic paths never push, release, delete branches, stash, reset, discard, or resolve conflicts.
- Documentation: repository-handwritten, generated definitions, and canonical-knowledge; run all mapped commands.

### Block 6: Verify and accept an immutable candidate

- [ ] Done.
- Dependencies: Blocks 1 through 5.
- Areas: complete repository diff and local integration path.
- Outcome: every bootstrap checklist item is satisfied, full verification passes, a fresh isolated read-only review reports no findings for immutable candidate `H`, configured acceptance is recorded, closure is committed, and the local default branch receives a no-fast-forward merge.
- Verification: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-targets`, `npm test`, `npm run check:docs`, `cargo run -- validate --warnings`, `cargo run -- sync --dry-run`, and `git diff --check`.
- Tests: complete acceptance journeys cover contract-011 acceptance, stale-default, rollback, ownership-CAS, and integration promises.
- Structural evidence: review input is the immutable diff from the approved base to candidate `H`, and integration verifies identity, ancestry, closure-at-revision, and exact administrative descendants.
- Documentation: every applicable surface must be current before this block can complete.

## Build state

Current block: 5. Attempts: 0. Final corrections: 0. Fingerprint: none. Blocker: none.

## Implementation decisions

Block 2: repository workflow transactions retain the cache lock across canonical publication and ownership CAS so concurrent sessions cannot observe or overwrite half-applied progress. A rejected ACCEPT transition updates the plan and primary issue in one staged knowledge publication. Human-gated transitions no longer accept approval flags: Rust verifies an in-memory Pi capability against the owning cache digest, while the parent-only control tool releases it only after trusted UI confirmation. Review evidence consumes a single-use extension-issued run bound to fixed base/candidate revisions. Rust executes every plan Verification command before final review, then records candidate-bound verification, review evidence, and the BUILD-to-ACCEPT phase change in one attestation commit. Workflow start, checkpoints, other transitions, closure, and reopening also receive knowledge-only service commits. ACCEPT rejection invalidates the rejected final evidence while preserving feedback. ACCEPT also rechecks the verified default before closure; drift invalidates final evidence and returns the same plan to BUILD in an administrative commit. Local integration rechecks both refs and safely switches from the work branch to the verified default branch before its no-fast-forward merge.

Block 3: every isolated role must terminate with role-specific `SUPERDEV_RESULT` JSON; the extension rejects missing, malformed, contradictory, or out-of-vocabulary results before orchestration can consume them. Owned sessions block switch and fork, child tool preflight enforces read-only reviewer roles and prohibits authoritative transitions, and the `/scope`, `/build`, and `/accept` commands now schedule their typed phase gates directly rather than forwarding orchestration prose. Failed immutable final reviews invoke a UI-authorized Rust correction command that durably increments the project-bounded correction count, invalidates candidate evidence, and causes Pi to schedule correction, verification, and a fresh immutable review until clean or exhausted. Pi reloads canonical workflow state from Rust into every parent turn so compaction summaries cannot become state authority, and `/superdev-resume` deterministically resumes one named open plan rather than forwarding recovery prose to the model. Scope-review publication now compare-and-swaps the pre-SCOPE owner revision against the changed reviewed plan revision, permits only knowledge changes, and commits the validated scope plus review evidence atomically.

Block 4: failed BUILD commands are normalized and SHA-256 fingerprinted by the shared Rust retry service. The typed attempt command updates the plan's durable consecutive-attempt state in a knowledge-only commit, marks the configured limit as stalled, refuses equivalent attempts beyond that limit, and permits checkpoint reset only when the same stable block newly completes. Successful checkpoints now require that current block to become newly complete, derive and enforce its declared dependencies and backtick Verification commands, and commit product, test, documentation, and plan changes only when every changed path is within the block's backtick-delimited Areas or the owning plan. Human abandonment closes and preserves the work-branch records, then publishes only the abandoned plan, wontfix issue, and generated indexes through a detached default-tip worktree and compare-and-swap update; partial product history never reaches the default branch. BUILD re-scope now transactionally preserves the child's discovery on the same primary issue, invalidates prior SCOPE evidence, records the existing product tip as the SCOPE baseline, and requires fresh review and approval while retaining the same issue, plan, branch, and stable blocks. Before final verification, typed BUILD synchronization compares both expected tips, uses `git merge-tree --write-tree` so conflicts leave the live index and worktree untouched, creates a hook-free merge object, rechecks both refs, and fast-forwards the checked-out work branch. Checkpoint publication rejects caller changes to service-owned retry fields, resets attempts and fingerprints only after the current stable block newly completes, preserves final-correction accounting, advances to the next incomplete block, and stores the resulting canonical plan revision in transient ownership.

## Follow-up issues

none.

## Completion evidence

The root specification was explicitly approved before commit `83e04ea`, and an isolated requirements pass reported no unresolved requirement before BUILD. Candidate commits through `1e8f20e` were rejected by fresh isolated review and are not acceptable candidates. Block 1 evidence is recorded in its block; remaining BUILD and acceptance evidence is pending.

<!-- sokf:links -->
[sokf:issue-059-scope-build-accept-workflow]: /knowledge/issues/open/issue-059-scope-build-accept-workflow.md
