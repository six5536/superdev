# Superdev Skill-First Workflow Plan

Status: Approved for direct implementation outside the Superdev workflow.

## Goal

Replace the ambiguous public `/scope`, `/build`, and `/accept` commands with explicit Pi skills that understand free-form user intent and invoke one deterministic phase tool. Keep semantic decisions with the LLM, authoritative state and Git safety in deterministic code, and all workflow internals out of the user-facing interface.

## Constraints

- Do not invoke SCOPE, BUILD, or ACCEPT while implementing this repair.
- Preserve unrelated work, including existing uncommitted changes to plan 077.
- Do not merge, push, delete branches, rewrite existing commits, or absorb unrelated changes.
- Keep the change targeted: reuse existing phase drivers, review logic, progress UI, diagnostics, question persistence, and Rust transitions where possible.
- Human discussion occurs through skills; deterministic trusted UI confirmation remains mandatory for authoritative decisions.
- Free-form intent is interpreted by an LLM, never by regex or deterministic keyword parsing.
- The LLM authors canonical issue and plan identities and content. Deterministic code validates and atomically persists; it does not invent canonical knowledge.
- Issue and plan identities are independent. Their authoritative association is a SOKF `rel: implements` link from plan to issue.
- The issue determines the work-branch name. A plan may not exist when SCOPE intake begins, and neither issue nor plan need exist when the user supplies a new feature request.

## Public interface

Create three explicit, non-auto-invoked Pi skills:

- `/skill:scope <free-form intent>`
- `/skill:build <optional execution intent>`
- `/skill:accept <optional acceptance intent>`

Each skill documents examples and valid input in `SKILL.md`. Set `disable-model-invocation: true` so ordinary conversation cannot start authoritative workflow behavior accidentally.

Remove the public `/scope`, `/build`, and `/accept` commands rather than retaining competing interfaces.

Add one model-facing deterministic tool, `superdev_run_phase`. It exposes human-level operations rather than sessions, revisions, ownership, or lock tokens:

- `run`
- `retry`
- `inspect`
- `record-answer`
- `revise-answer`
- `submit-answers`
- `approve`
- `cancel`

The tool accepts the selected phase and, where required, exact LLM-selected issue/plan identities and confirmed authored records. It translates those operations into the existing phase drivers, question controller, and Rust workflow service.

## SCOPE behavior

The SCOPE skill:

1. Interprets arbitrary user intent semantically.
2. Searches canonical knowledge for related issues and plans.
3. Recommends adopting an existing issue or creating a new issue.
4. Presents interpreted intent, recommendation, scope boundary, and proposed title/description.
5. Supports Confirm, Revise, or Cancel discussion.
6. After confirmation, authors the issue if needed using SOKF-aware tools.
7. Finds a suitable plan linked through `rel: implements`, or authors an independently identified initial plan.
8. Calls `superdev_run_phase` with exact identities after the issue/plan pair exists.

The deterministic phase tool validates IDs, schemas, lifecycle, link direction, branch compatibility, collisions, and current repository state. It establishes or resumes the workflow and runs the existing immutable checkpoint and exhaustive requirements review. It never derives a plan ID from an issue ID and never writes semantic issue or plan content.

Review findings, revisions, recovery, and final approval are discussed by the SCOPE skill. The phase tool returns typed states such as `review-clean`, `findings`, `failed`, and `ready-for-approval`. Final authoritative approval still requires trusted UI confirmation.

## BUILD and ACCEPT behavior

The BUILD skill interprets instructions only within the approved plan. Intent-changing requests return to `/skill:scope`; they are not silently incorporated. The BUILD phase tool enforces the approved revision and retains existing checkpoint, verification, exhaustive review, correction-budget, progress, cancellation, and routing behavior.

The ACCEPT skill explains assessment results and routing. Within-scope findings return to BUILD; intent-changing findings return to SCOPE. Authoritative acceptance remains trusted-UI gated according to configuration.

## Internal interfaces

- Make low-level `superdev_workflow_control` and `superdev_workflow_questions` unavailable to the main model after `superdev_run_phase` covers their operations.
- Block direct main-agent invocation of SCOPE, BUILD, requirements-review, code-review, and ACCEPT roles through `superdev_isolated_role`; phase drivers continue invoking isolated roles internally.
- Retain isolated filing support only where required by the skill flow.
- Keep revision-bound question persistence internally. Skills access it only through `superdev_run_phase` inspect/answer operations.
- Preserve bounded owner-only isolated diagnostics. Failures returned to skills include failed stage, exact bounded diagnostic path, whether partial work was preserved, valid recovery operations, and a recommendation.

## Recovery and interaction

Deterministic phase code must not present opaque recovery menus. It returns typed recovery state to the invoking skill. The skill explains the failure and discusses one choice at a time. The deterministic tool validates and executes the selected retry, revision, cancellation, or phase return.

Protocol and infrastructure failures do not consume semantic correction budgets. No free-form child output is authoritative. Existing progress, elapsed-time updates, Esc cancellation, process-group cleanup, and bounded artifacts remain active.

## Rust adjustments

Remove the assumption that issue, plan, and work branch share one `NNN-slug`. Validate instead that:

- issue and plan IDs independently satisfy their schemas;
- the plan has exactly one `rel: implements` issue link matching the selected issue;
- the plan records the selected issue-derived work branch;
- start/resume state agrees with canonical records;
- branch and identity changes remain compare-and-swap protected.

Where initial issue/plan content is supplied, Rust validates and commits exact LLM-authored content without synthesizing semantic fields.

## Verification

Add deterministic coverage for:

- Skill discovery, explicit-only invocation, documentation, and argument handling.
- Free-form new-feature intake instructions.
- Existing issue with no plan.
- Existing linked plan with a different numeric identity.
- Multiple linked plans requiring semantic selection.
- New LLM-authored issue and plan validation.
- Issue-derived collision-safe branch handling.
- Main-model hiding of low-level control/question tools.
- Rejection of direct phase-role invocation.
- Generic phase-tool run, retry, question, approval, and cancellation operations.
- Typed recovery outcomes with diagnostic paths and preserved-work state.
- Existing progress, timeout, Esc cancellation, immutable review, correction accounting, and integration behavior.
- Exact `.pi`/`pack` extension and skill mirrors, lock synchronization, script tests, Rust tests, clippy, documentation checks, and SOKF validation.

## Completion

Implement directly in small reviewable commits. Keep this root plan until the user removes it manually. Do not alter or commit the unrelated uncommitted plan-077 changes.
