# Process: Refactoring

## 1. Pin down the invariant

- A refactor changes structure, not behavior. State what "behavior" means here: outputs, API surface, performance envelope, error messages, wire formats.
- Verify test coverage over the code being restructured. If coverage is thin, add characterization tests first — they are the safety net.

## 2. Establish the baseline

- Run the full relevant test suite and record the result before touching anything. A pre-existing failure discovered mid-refactor is indistinguishable from one you caused.

## 3. Move in small, reversible steps

- Sequence the work so the code compiles and tests pass after every step: rename, then extract, then move, then delete — not all at once.
- Use mechanical, tool-assisted transformations where available (rename symbol, codemod) over hand-editing many sites.
- Commit (or checkpoint) at each green state so any step can be rolled back alone.

## 4. Keep behavior changes out

- If the refactor exposes a genuine bug, do not silently fix it mid-refactor: record it, finish the refactor preserving the old (buggy) behavior if safe, and fix it in a separate change — or pause and ask.
- Resist scope creep: improvements spotted along the way go on a list, not in the diff.

## 5. Verify equivalence

- Full test suite green; lint/typecheck/build clean.
- Diff review pass over the final change asking one question: could this hunk change behavior?

## 6. Report

- What was restructured and why it's better (duplication removed, dependency untangled, name clarified) — with before/after shape, not a hunk-by-hunk tour.
- Explicitly state that behavior is unchanged and how that was verified; list any bugs found and deferred.
