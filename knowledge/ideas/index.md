# Ideas

Thoughts captured before they are lost, kept for considering later. Nothing
here is official: an idea is not candidate work, so it does not appear in
the [issue tracker](../issues/index.md) until it is taken up, at which point
it stays on file, linked from the issue that took it up.

* [Schemas carry a reading reminder][sokf:idea-001-schemas-carry-a-reading-reminder] - every schema fixes a short instruction its documents must carry, so an agent reading one is told how to treat it without opening the schema.
* [Support other harnesses][sokf:idea-002-support-other-harnesses] - materialise superdev's skills, agent wiring and hooks for harnesses beyond Claude Code — Codex, Pi, OpenCode and the like.
* [Ask whether work runs in a worktree or the main working tree][sokf:idea-003-ask-worktree-or-main-working-tree] - when a feature's branch is cut, ask the user whether the work runs in a linked git worktree or in the main working tree, instead of always switching the main checkout.
* [Schemas anchor a section's style to a known artifact][sokf:idea-004-schemas-anchor-a-section-style] - a section's guidance names a well-known artifact whose register the writer matches — "each `about` reads as `rg --help` prints one" — so the text is written for where it lands.
* [A documentation review skill][sokf:idea-005-a-documentation-review-skill] - a skill that reviews the documentation a reader outside the repository meets — README, CONTRIBUTING, changelog, help text, API docs — against the code and against the reader it serves.
* [Validation runs once, at the turn's end][sokf:idea-006-validation-runs-once-at-the-turn-end] - drop the PostToolUse validation hook and leave the Stop hook as the only gate, so an agent pays the validator's cost once a turn instead of once an edit.
* [A knowledge-capture skill][sokf:idea-007-a-knowledge-capture-skill] - the write-side complement to the search-first AGENTS.md — a skill that teaches an agent when and how to record a durable learning in the knowledge mid-task.
* [Templates pre-fill knowledge skeletons][sokf:idea-008-templates-pre-fill-knowledge-skeletons] - a project template fixes facts about the repository it creates, so it could pre-fill parts of the technology-stack and architecture skeletons instead of leaving them TBD.
* [Comment-preserving manifest stamping][sokf:idea-009-comment-preserving-manifest-stamping] - stamp the blueprint version into config.toml with a targeted toml_edit edit of the one key, so a hand-editable file keeps its comments.

<!-- sokf:links -->
[sokf:idea-001-schemas-carry-a-reading-reminder]: /knowledge/ideas/idea-001-schemas-carry-a-reading-reminder.md
[sokf:idea-002-support-other-harnesses]: /knowledge/ideas/idea-002-support-other-harnesses.md
[sokf:idea-003-ask-worktree-or-main-working-tree]: /knowledge/ideas/idea-003-ask-worktree-or-main-working-tree.md
[sokf:idea-004-schemas-anchor-a-section-style]: /knowledge/ideas/idea-004-schemas-anchor-a-section-style.md
[sokf:idea-005-a-documentation-review-skill]: /knowledge/ideas/idea-005-a-documentation-review-skill.md
[sokf:idea-006-validation-runs-once-at-the-turn-end]: /knowledge/ideas/idea-006-validation-runs-once-at-the-turn-end.md
[sokf:idea-007-a-knowledge-capture-skill]: /knowledge/ideas/idea-007-a-knowledge-capture-skill.md
[sokf:idea-008-templates-pre-fill-knowledge-skeletons]: /knowledge/ideas/idea-008-templates-pre-fill-knowledge-skeletons.md
[sokf:idea-009-comment-preserving-manifest-stamping]: /knowledge/ideas/idea-009-comment-preserving-manifest-stamping.md
