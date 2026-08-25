# Process: Fixing a bug

## 1. Reproduce first

- Get the failure to happen locally — a failing test, a command, a request. Exact commands, exact output.
- If it can't be reproduced, that is the current task: gather the missing detail (versions, inputs, logs) before theorizing.

## 2. Find the root cause

- Trace from symptom to mechanism: which input/state takes the code down the wrong path, and where.
- Distinguish the trigger (what exposed it) from the defect (what's wrong). Fix the defect.
- A signal that pattern-matches a known failure may have a different cause — confirm with evidence before acting on the pattern.

## 3. Write the regression test before the fix

- Add a test that fails for the reason the bug fails. Watch it fail — a test that never failed proves nothing.
- Put it where the project's tests for that module live, named for the behavior, not the ticket.

## 4. Fix minimally

- The smallest change that removes the defect. Resist drive-by refactors — note them for a separate change.
- Check the same defect pattern elsewhere: if the mistake was easy to make once, look for its siblings.

## 5. Verify

- The new test passes; the previously failing repro now behaves correctly.
- The surrounding test suite still passes — the fix must not trade one bug for another.

## 6. Report

- State the root cause as a mechanism, not a location: "X assumed Y, but Z" beats "fixed file.ts".
- Include the repro, the fix, the test added, and the verification output.
- If the fix has behavioral side effects, call them out explicitly.
