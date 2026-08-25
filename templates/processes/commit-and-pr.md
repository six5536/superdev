# Process: Committing and opening a PR

Only commit or push when the user asks. On the default branch, create a branch first.

## 1. Review what's actually staged

- `git status` and `git diff` before every commit — commit what you verified, not what you remember changing.
- Exclude accidental artifacts: debug prints, scratch files, unrelated formatting churn, secrets and credentials (never commit these; if one slipped into history, say so immediately).

## 2. Shape the commits

- One logical change per commit; if the message body needs "and also", split it.
- Follow the repo's existing message convention (check `git log`); default to conventional commits (see `templates/commit-message.md`).
- Message body explains why, not what — the diff already shows what.
- End commit messages with the Co-Authored-By trailer.

## 3. Verify before pushing

- Tests/lint/build green on the exact tree being pushed, not an earlier state.
- `git log` the branch once to confirm it contains what you intend and nothing else.

## 4. Open the PR

- Use `gh` for GitHub operations.
- Title: imperative, ≤72 chars. Body per `templates/pr-description.md`: summary, motivation, test plan with real results.
- Target the repo's usual base branch; link the issue; end the body with the Generated-with-Claude-Code footer.

## 5. Report back

- Give the user the branch name, PR URL, and one-line summary of what it contains.
- State verification honestly: which checks ran locally, which are pending in CI.
