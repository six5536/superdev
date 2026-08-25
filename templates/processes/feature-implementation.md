# Process: Implementing a feature

## 1. Understand the request

- Restate the goal in one sentence: what works after this that doesn't today.
- Identify what's ambiguous. Resolve it from the codebase or sensible convention where possible; ask the user only for decisions that genuinely change the work.
- Note explicit constraints (API stability, performance, style) before touching code.

## 2. Explore before writing

- Find the code the feature touches and read it — entry points, the module that owns the behavior, its tests.
- Find a similar existing feature and note how it's structured; the new code should look like it belongs.
- Check for existing utilities before writing new ones.

## 3. Plan proportionally

- Small change: hold the plan in your head, state it in one sentence, go.
- Larger change: write the ordered steps first (see `templates/plan.md`), sequenced so the codebase stays working after each step. Get user sign-off if scope or approach is a real decision.

## 4. Implement

- Work in the order planned: types/contracts first, then core logic, then call sites, then tests — unless tests-first fits better.
- Match the surrounding code's idioms, naming, and comment density.
- Do not gold-plate: implement what was asked, note (don't build) adjacent improvements.
- When a step reveals the plan was wrong, stop and revise the plan; don't patch around it.

## 5. Verify

- Run the tests that cover the touched area, then the wider suite if cheap.
- Run lint/typecheck/build as the project defines them.
- Exercise the feature the way a user would (run the app/CLI) — passing tests on an unexercised path is not verification.

## 6. Report

- Lead with what now works and how it was verified (exact commands, real results).
- List files changed with one line each.
- State anything deferred or known-incomplete plainly. Never claim more than was verified.
