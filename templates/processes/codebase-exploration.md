# Process: Exploring an unfamiliar codebase

## 1. Orient from the top

- Read the README, CLAUDE.md, and top-level directory listing before any source file.
- Identify the project type (library, CLI, server, app), language/toolchain, and how it's built, tested, and run.

## 2. Find the entry points

- Locate where execution starts: `main`, exported package surface, route registrations, CLI command table.
- Follow one representative flow end to end before breadth-reading — depth on one path teaches the architecture faster than skimming everything.

## 3. Map ownership, not files

- Answer "which module owns X?" for the concerns relevant to the task: state, I/O, config, errors.
- Note the naming and layout conventions as you go — where tests live, how modules import each other, what a "typical" file looks like.

## 4. Search with intent

- Search for symbols and strings tied to the task (error messages are excellent anchors — they appear near the code that emits them).
- Prefer reading the specific parts needed over whole files; delegate broad fan-out sweeps to a search agent and keep only conclusions.
- Trust the code over the docs when they disagree — then note the disagreement.

## 5. Validate the mental model

- Predict something ("changing X should affect Y") and check it — run a test, trace a call — before building on the model.
- Keep a short scratch map of load-bearing facts: entry points, key types, the files the task will touch.

## 6. Know when to stop

- Exploration serves a task. Stop when you can name the files to change and the pattern to follow — not when you understand everything.
