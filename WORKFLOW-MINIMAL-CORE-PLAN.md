# Workflow minimal-core plan

## Raw user intent

The deterministic code holds too much. The LLM should decide; the deterministic
code should manage state and the branch. The current harness is overcomplicated.

Issue-077 is on hold until the harness is fixed.

## Evidence

The harness is about 7,900 lines: 5,400 Rust under `crates/*/workflow*`, 2,485
TypeScript under `.pi/extensions/superdev`. Every workflow failure observed over
several days originated in that harness, not in the isolated children:

- terminal submission protocol rejections
- a valid SCOPE correction refused for contradictory status
- `ask` without a finding ID reported as a phase failure
- mechanical findings corrected before the substantive findings they depend on

The reviewer children returned well-formed, actionable findings on every run.

## Decisions

1. Reduce the deterministic core to state, branch, and a refusal list.
2. Plan markdown stays canonical. Nothing parses it. The LLM reads and writes
   progress; Rust tracks phase only.

## Boundary

The core enforces guarantees **about** the agent, not judgements the agent can
make. An agent cannot enforce a refusal on itself, and a race is not a reasoning
problem.

### Retained (about 900 lines)

- Git refusal list: never push, merge, delete branches, stash, reset, discard.
- Branch identity and identity reservation under the repository lock.
- `start`, `resume`, `cancel`, `status`: one executing workflow per checkout.
- `Phase` and legal `Transition`: durable state survives session loss.
- `abandon`: human-only authority.
- Bounded child output and owner-only artifacts.
- In `GateEvidence`: `human_scope_approved`, `human_acceptance_approved`,
  `human_abandonment_approved`. These record authority.

### Removed (about 6,000 lines)

- Prose parsing: `evidence.rs` (305) entirely, and the parsing half of
  `records.rs`. These prefix-match `"Final verification: "` inside markdown an
  LLM wrote. `records.rs` also holds `emit`, `plan_revision`, `phase_text`,
  `close_records`, and the transactional edit helpers, so it is trimmed rather
  than deleted.
- Block accounting: `block`, `attempt`, `correction`, `correction-checkpoint`,
  `scope-baseline`, `scope-checkpoint`, `activity-start`.
- Attestation ceremony: `assess`, `sync`, `integrate`, `candidate_revision`,
  `verified_default_revision`, and stale-default recovery.
- Correction scheduler in `phases.ts`: cycle budgets, fingerprint answer reuse,
  and the mechanical-first shortcut that ignores `dependsOn`.
- Review arbitration in `review.ts`: status and classification contradiction
  checks that terminate otherwise valid runs.
- Terminal-protocol policing in `process.ts`, including the repair turn.
- `GateEvidence::requirements_review_clean`. This judges competence.

`questions.ts` keeps durable answer storage. Its ordering logic moves to the LLM.

## Consequences requiring their own records

1. **Public contract change.** `contract-002-cli-superdev.md:315` embeds the
   workflow CLI through `sokf:include`. Removing subcommands rewrites that
   Definition section.
2. **ADR-052 is superseded in part.** Its Decision states that a canonical plan
   carries durable progress and evidence, and that BUILD owns every work block
   and final verification. A new ADR must supersede it; ADR-052 is never edited.
3. **Internal contract change.** `contract-011-interface-workflow.md` embeds
   `state.rs` and `transition.rs` wholesale, so it regenerates when the cache
   fields and gate evidence shrink.
4. **Prose parsing reaches into the transition path.** `transition.rs` reads
   `"Candidate revision"`, `"Verified default revision"`, and the BUILD retry
   line out of plan markdown, so the parsing removal is wider than the two
   dedicated modules.

## Steps

TypeScript changes come first. Removing a Rust subcommand the extension still
calls breaks every intermediate state; making the extension stop calling it
first leaves dead code that deletes with the suites green throughout.

1. Author the superseding ADR: the core owns state, branch, and refusals; the
   plan carries progress that nothing parses.
2. Repair the SCOPE correction loop in `phases.ts`. One substantive finding
   blocks every correction in its set, because a mechanical finding may declare
   one as a dependency. Pass the complete finding set to the correction child
   with the confirmed answers. Remove the mechanical-first shortcut, the
   semantic-fingerprint answer reuse, and the cycle scheduling around them.
3. Reduce `review.ts` to a result shape and `process.ts` to spawn, bounded
   output, and one typed result. Remove status and classification arbitration
   and terminal-submission policing.
4. Reduce `questions.ts` to durable answer storage with dependency eligibility.
   Ordering belongs to the LLM.
5. Move the removed responsibilities into the role prompts and skills.
6. Delete the Rust subcommands the extension no longer calls: `assess`, `sync`,
   `integrate`, `scope-baseline`, `scope-checkpoint`, `block`, `attempt`,
   `correction`, `correction-checkpoint`, `evidence`, `activity-start`,
   `activity-finish`. Replace the three specialized commit verbs with one
   generic `commit` that checks the owned branch and plan revision and commits
   what is present. Committing is a branch operation; deciding what to commit is
   not.
7. Delete `evidence.rs`, `build.rs`, `retry.rs`, and the parsing half of
   `records.rs`. Remove prose parsing from `transition.rs`, including
   `evidence_revision`, `build_state_line`, `filter_completion_evidence`, and
   the completion-evidence edits. Drop `closure_integrated`,
   `requirements_review_clean`, `RecoverStaleDefault`, `candidate_revision`,
   `verified_default_revision`, and `scope_base_revision`. Human approval
   remains the SCOPE gate.
8. Delete tests bound to the removed surface. Retain and extend the real-Git
   safety tests covering the refusal list, identity reservation, and one
   workflow per checkout.
9. Regenerate `contract-002` and `contract-011`, and update `contract-004`,
   `configuration.md`, ADR index, and user documentation.
10. Verify: `cargo test --workspace`, `npm run test:scripts`,
    `npm run test:launcher`, `npm run check:docs`, `cargo run -- sync`,
    `cargo run -- status --drift`, `cargo clippy --workspace --tests -- -D warnings`,
    `cargo fmt --all`, and `superdev validate --fix` until PASS.

## Constraints

Work happens on `main`, outside the Superdev workflow. Issue-077 keeps its three
commits on `work/077-sokf-file-tool-parity`. Do not push that branch, merge it,
stash, discard work, or delete branches.

## State of issue-077

Already parked: `superdev workflow status` reports no owner and no phase, and the
checkout is clean. No further action holds it.

## Progress and verification

Plan written before changing the implementation. Nothing removed yet.
