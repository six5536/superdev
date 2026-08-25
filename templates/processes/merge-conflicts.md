# Process: Merge conflicts and branch syncing

## 1. Sync deliberately

- Follow the repo's convention for updating a branch (rebase vs merge — check history or ask); don't impose a preference.
- Never rebase commits that others may have pulled; on published shared branches, merge.
- Start from a clean working tree so conflict resolution is the only uncommitted change in play.

## 2. Understand both sides before resolving

- For each conflicted file, establish what each side was trying to do — `git log` both branches for the file, read the commits, not just the conflict markers.
- The goal is a result that satisfies both intents, which is often neither hunk verbatim. "Pick ours/theirs" is only correct when one side's change is genuinely superseded.

## 3. Resolve with the whole file in view

- After editing out the markers, read the full resolved file, not just the conflicted region — the surrounding code must still make sense with both changes present.
- Watch for the classic traps: both sides added a similar function (keep one, merge call sites), one side renamed what the other side modified (apply the modification under the new name), import/dependency lists merged into duplicates or omissions.

## 4. Hunt the semantic conflicts

- A merge with no textual conflicts can still be broken: side A renamed a function that side B added a new call to; side A changed a contract side B relies on. Git cannot see these.
- After any sync — conflicted or clean — build and run the affected tests before pushing. The merged tree is a new state nobody has tested yet.

## 5. Verify and finish

- Full relevant test suite green on the merged/rebased tree; diff against the upstream base to confirm the branch's own changes survived intact and nothing extra crept in.
- If a rebase went bad, abort or use the reflog to return to the pre-rebase state and reattempt — don't hand-repair a mangled rebase mid-flight.

## 6. Report

- What was synced onto what, which files conflicted and how the intent of each side was preserved, and the verification run. Flag any resolution that involved a judgment call the branch authors should double-check.
