---
name: double-check
description: Use when the user asks to check or recheck a plan, design or implementation for errors or omissions.
---

# Double-check

1. Read the selected work and its governing intent. Establish the review boundary and the exact revision; do not assume a previous review was correct.
2. Trace each requirement to the proposed design or implementation and its verification. Check missing behaviour, contradictions, dependencies, boundaries, error handling, and affected interfaces and documentation.
3. Inspect the relevant code and evidence. For implementation reviews, check that tests exercise the required outcomes rather than merely accepting the implementation. Identify checks that were not run.
4. Report actionable findings with location, evidence, impact, and a recommended correction. Distinguish defects from unresolved decisions and optional improvements.
5. State the result and its limits. Do not claim skipped checks passed or treat a clean result as human approval.
6. Return findings to the caller. Correct them only under separate permission. Repeat the review when requested or after authorised corrections, not as an automatic loop until approval becomes possible.
