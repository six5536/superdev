# Process: Reviewing code

## 1. Establish context

- Read the PR/commit description and the issue behind it: what is this change supposed to do?
- Skim the whole diff once for shape before judging any line — reviewing hunks without the whole picture produces false positives.

## 2. Review in passes, most severe concern first

- **Correctness:** trace the changed paths with hostile inputs — empty, zero, huge, concurrent, failing dependencies. For each suspicion, try to construct a concrete failure scenario (inputs/state → wrong output). No scenario, no finding.
- **Security:** anywhere the change touches user input, authn/authz, secrets, file paths, or subprocesses.
- **Design fit:** does it duplicate an existing utility, fight the codebase's conventions, or add complexity the task didn't need?
- **Tests:** do the added tests fail if the feature breaks? Is the risky path covered, or only the happy path?

## 3. Verify before reporting

- Read the surrounding unchanged code before claiming a bug — the guard you think is missing may live one call up.
- Run the code or tests when a suspicion is checkable cheaply; a confirmed finding beats a plausible one.
- Drop findings that don't survive verification; keep a "checked and fine" note so they aren't re-raised.

## 4. Rank and write findings

- Order by severity, each with: location (`file:line`), one-sentence claim, concrete failure scenario, suggested fix (see `templates/code-review.md`).
- Separate blocking findings from nits and style preferences; never let a nit list bury a correctness bug.
- Critique the code, not the author; state facts and mechanisms, not taste.

## 5. Deliver a verdict

- One sentence up front: mergeable as-is, mergeable after fixes, or needs rework — so the author knows the stakes before reading details.
