# Workflow fixes plan

## Raw User Instruction

Outside of the superdev workflow, review the implemented superdev workflow (SCOPE -> BUILD -> ACCEPT)

The idea is that it works like this:

- The user creates an issue in the current session (or not). Issues are created using the /issue skill (maybe this is called file and not a
  skill, which is wrong!). Deterministic checks ensure the issue is committed and modified on 'main' branch (whatever the main branch is
  called)
- The user calls /skill:scope <issue or description> (or the system realises the user is wanting to do this and invokes automatically)
- The skill walks the user through properly defining the issue (grills the issue with the user, using a question and suggestion UI with
  multiple answers, and the recommended one, and the ability to chat to answer as well)
- The skill double checks the issue is well defined and fits the existing system
- The skill creates a plan from the issue that includes all the contract changes required among other things
- The skill defers to a 'sub-agent' or new pi process to review the issue and plan, with the results coming back to the main session for
  the user to work through as a question/answer similar to the grilling sessions. The agent fixes any and all issues it can without user
  input (no decision required)
- The user submits answers and the process may repeat until issues are trivial
- Then the system can suggest entering the build skill, which uses a sub-agent to build the tests and code and documentation changes on a feature / bug / chore branch in parts, with full tests only run after all parts implemented
- This may exit to rescope if problems found
- Once complete it automatically enters accept that does the acceptance and may require human input depending on a flag in the config. accept may return to build or scope as required (scope requiring human input, build automatic).
- Once accept passes, the user is notified and may merge the branch. The main session always stays on main branch

## Agreed outcome

Keep one executing workflow per checkout. Create and commit issues and initial linked plans on the discovered default branch, reserving independent numeric identities before switching to the issue's work branch. The parent session and children then share that checkout for all document and product updates. Cancellation preserves the checkout and partial work. ACCEPT closes the accepted records on the work branch and notifies the user; it never merges automatically.

This maintenance task runs outside the Superdev workflow, on main as requested. Preserve issue-077's drafts and unrelated configuration on its existing branch. Do not push, merge that branch, stash, discard work, or delete branches.

## Implementation steps

1. Selectively import the reviewed workflow implementation from `work/077-sokf-file-tool-parity`. Import workflow code, its tests, pack copies, and directly affected workflow contracts/configuration documentation only. Exclude issue-077, its contracts/ADR, ideas, indexes, root proposals, and unrelated Entire settings. Inspect shared-file diffs before import.
2. Repair nominal phase execution. Resolve the BUILD executable and digest at invocation time. Distinguish reviewed candidate H from the administrative attestation commit. Bind re-scope questions to the post-transition revision. Preserve complete findings through correction and recovery.
3. Simplify acceptance. Retain configured human acceptance, automatic assessment and within-scope correction. Finish with an accepted, clean work branch and released ownership; leave merging to the human. Automatically return stale-default candidates to BUILD. Require fresh human SCOPE approval for changes in intent.
4. Expose interaction and skills. Provide an issue skill and a typed human-confirmed filing operation with bug/feature/chore kinds. Enable semantic skill discovery. Expose multiple-choice questions, recommendations, typed answers and chat for intake and review. Keep low-level transitions private. Make confirmed human change requests work after earlier question batches.
5. Enforce checkout and identity safety. Discover the default branch without hardcoding main, preserve it in canonical workflow metadata, and recover paused workflows from their actual branches. Reserve initial issue/plan records on the default branch under a repository lock, reject duplicate numeric identities, and switch only with a clean checkout. Refuse unrelated filing while on an active work branch. Block concurrent parent mutation while a child runs.
6. Define verification stages. Run focused verification at each block checkpoint and complete-suite verification only after all blocks are implemented. Preserve approved command boundaries and correction verification. Include contract, knowledge, test, and documentation changes in SCOPE plans.
7. Add regression tests for real extension initialization, BUILD pinning, administrative candidate assessment, question UI reachability, re-scope revisions, manual merge boundary, non-main branch recovery, numeric collisions, ownership exclusion, and staged verification. Exercise Rust-backed temporary repositories where Git state matters; do not rely solely on mocks that keep HEAD fixed.
8. Synchronize pack resources, user documentation, ADR-052, workflow/CLI/configuration contracts and relevant canonical knowledge. Split touched oversized modules into logical files. Run focused tests during implementation, then full Rust and script suites, documentation checks, formatting and `superdev validate --fix` until PASS. Record actual verification and remaining limitations below.

## Acceptance checks

- Issue creation and initial plan reservation happen on the default branch before work-branch creation.
- Independent workflows cannot reuse an existing issue or plan number in the local repository.
- Parent and child edits target the same checked-out work branch; no worktree routing architecture is introduced.
- SCOPE supports semantic intake, an explicit question UI, isolated review and confirmed approval.
- BUILD starts with a valid service pin, executes all approved blocks, and enters ACCEPT automatically.
- ACCEPT tolerates the service-owned attestation commit, routes findings correctly, honors human acceptance configuration and never merges the branch.
- Cancellation and failures preserve work and permit explicit recovery without losing answers or identities.
- Non-main defaults and multiple unfinished branches recover correctly; one checkout has one executing workflow.
- Tests and canonical/public documentation describe the same behavior.

## Progress and verification

Plan written before importing or changing the implementation. The workflow carryover and fixes are implemented on main, without merging the issue-077 branch or importing its additional proposal documents.

Implemented: invocation-time BUILD service pinning; administrative attestation assessment above reviewed H; post-transition re-scope question revisions; public choice/recommendation/typed/chat interaction; revisable answer batches; discoverable issue and phase skills; shared-checkout ownership and default-branch recovery; locked independent identity reservation; default-branch-only filing with preserved issue category; cancellation propagation; staged verification; manual acceptance/merge separation. Split the extension, Rust workflow CLI, and Git implementation into logical modules. Synchronized pack assets, managed hashes, workflow contracts, ADR-052, instructions, and development guidance. Repaired the index entry for issue-077 already present on main; its content was not changed.

Verification:
- `cargo test --workspace`: 864 tests passed, including the real-Git workflow safety regression and explicit human-only integration test.
- `npm run test:scripts`: 32 passed, including the extension fixture executed directly against the pinned Pi dependency. The fixture requires an explicit completion marker rather than trusting a successful Pi CLI exit.
- `npm run test:launcher`: 9 passed.
- `npm run check:docs`: 6 documentation surfaces checked.
- `cargo run -- sync` and `cargo run -- status --drift`: passed; live/pack resources and managed hashes synchronized.
- `cargo run -- validate --fix`: PASS, zero errors or warnings, zero repairs on the final validation run.
- `cargo clippy --workspace --tests -- -D warnings`: passed.
- `cargo fmt --all` and `git diff --check`: passed. Stable rustfmt reports unsupported optional configuration settings.

Limits: automated extension tests use controlled UI/process responses; a live provider-backed interactive rehearsal, cross-platform CI, and coverage thresholds have not run. Numeric reservation coordinates this local repository, not independent clones. Existing `work/<issue-number>-<slug>` naming remains; issue category is stored independently. Filing retains its internal temporary staging worktree, not workflow document routing. Legacy large integration-test files remain; the split production workflow modules are below 800 lines. Changes remain uncommitted for review.
