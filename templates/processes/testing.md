# Process: Writing tests

## 1. Decide what the tests must protect

- List the ways this code is most likely to break (see `templates/test-plan.md`); tests exist to catch those, not to inflate coverage numbers.
- Test observable behavior through the public surface, not internal implementation details — tests coupled to internals break on every refactor and catch nothing.

## 2. Follow the house style

- Find how this project tests similar code: framework, file placement, naming, fixture patterns, mocking approach. Match it exactly.
- Reuse existing fixtures and helpers before writing new ones.

## 3. Cover the risk tiers in order

1. **Happy path** — the documented, common case.
2. **Edges** — empty, zero, one, max, unicode, duplicates, boundaries.
3. **Failure paths** — invalid input, dependency errors, timeouts; assert the error behavior, not just "it throws".
4. **Concurrency/ordering** — only where the code actually has it.

## 4. Make each test diagnostic

- One behavior per test, named so a failure reads as a sentence about what broke.
- Assert specific values, not just "truthy" or "no exception".
- No conditional logic in tests; a test that adapts to the output tests nothing.
- Deterministic: control time, randomness, and external services — flaky tests are worse than no tests.

## 5. Watch tests fail

- For new tests guarding new code: break the code (or write test-first) and confirm the test fails for the right reason before trusting it green.

## 6. Run and report

- Run the new tests, then the file's suite, then the wider suite if cheap. Report actual pass/fail output, never inferred results.
