# Ideas

Thoughts captured before they are lost, kept for considering later. Nothing
here is official: an idea is not candidate work, so it appears in neither the
[backlog][sokf:backlog] nor the [issue tracker](../issues/index.md) until it
is taken up, at which point it leaves.

* [Schemas carry a reading reminder][sokf:idea-001-schemas-carry-a-reading-reminder] - every schema fixes a short instruction its documents must carry, so an agent reading one is told how to treat it without opening the schema.
* [Support other harnesses][sokf:idea-002-support-other-harnesses] - materialise superdev's skills, agent wiring and hooks for harnesses beyond Claude Code — Codex, Pi, OpenCode and the like.
* [Ask whether work runs in a worktree or the main working tree][sokf:idea-003-ask-worktree-or-main-working-tree] - when a feature's branch is cut, ask the user whether the work runs in a linked git worktree or in the main working tree, instead of always switching the main checkout.
* [Schemas anchor a section's style to a known artifact][sokf:idea-004-schemas-anchor-a-section-style] - a section's guidance names a well-known artifact whose register the writer matches — "each `about` reads as `rg --help` prints one" — so the text is written for where it lands.
* [A documentation review skill][sokf:idea-005-a-documentation-review-skill] - a skill that reviews the documentation a reader outside the repository meets — README, CONTRIBUTING, changelog, help text, API docs — against the code and against the reader it serves.
* [Validation runs once, at the turn's end][sokf:idea-006-validation-runs-once-at-the-turn-end] - drop the PostToolUse validation hook and leave the Stop hook as the only gate, so an agent pays the validator's cost once a turn instead of once an edit.

<!-- sokf:links -->
[sokf:backlog]: /knowledge/backlog.md
[sokf:idea-001-schemas-carry-a-reading-reminder]: /knowledge/ideas/idea-001-schemas-carry-a-reading-reminder.md
[sokf:idea-002-support-other-harnesses]: /knowledge/ideas/idea-002-support-other-harnesses.md
[sokf:idea-003-ask-worktree-or-main-working-tree]: /knowledge/ideas/idea-003-ask-worktree-or-main-working-tree.md
[sokf:idea-004-schemas-anchor-a-section-style]: /knowledge/ideas/idea-004-schemas-anchor-a-section-style.md
[sokf:idea-005-a-documentation-review-skill]: /knowledge/ideas/idea-005-a-documentation-review-skill.md
[sokf:idea-006-validation-runs-once-at-the-turn-end]: /knowledge/ideas/idea-006-validation-runs-once-at-the-turn-end.md
