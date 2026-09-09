---
name: issue
description: Capture a bug, feature, or chore as a canonical issue committed on the default branch. Use when the user wants to create or file an issue without starting implementation. Invoke with /issue or /skill:issue.
allowed-tools: read sokf_search sokf_graph superdev_ask superdev_file_issue superdev_run_phase
---

# Create an issue

Treat the appended User line as intent, not command syntax. Search canonical issues for duplicates and read plausible matches. Recommend reusing an existing issue when its intent matches.

Inspect with `superdev_run_phase`, `phase: "scope"`, `action: "inspect"`. Issue creation requires an unowned checkout on the discovered default branch. If a workflow is active, ask the user to pause it and return explicitly to the default branch. Never switch branches while files are uncommitted.

Discuss the problem, expected behavior, boundaries, and bug/feature/chore category. Use `superdev_ask` for one unresolved question at a time with concrete choices and a recommendation. Let the user type another answer or discuss it in chat.

Present the complete title, description and category. Call `superdev_file_issue` only after the user agrees to create it. Trusted UI confirms the exact content; Rust allocates the ID under the repository lock, validates the record and commits it on the default branch. Do not select or write a numeric issue ID yourself.

Report the returned identity. Offer SCOPE when the user wants to proceed. Filing alone never creates a work branch or starts BUILD.
