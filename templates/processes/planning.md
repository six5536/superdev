# Process: Planning a piece of work

## 1. Decide whether a plan is needed

- One obvious change in one place: no written plan — state the intent in a sentence and do it.
- Multiple files, an approach choice, irreversible steps, or user sign-off required: write the plan (see `templates/plan.md`).

## 2. Ground the plan in the actual code

- Explore first (see `codebase-exploration.md`): a plan written before reading the code is a guess with headings.
- Name the real files, functions, and patterns involved — "update the config module" is not a step; "add `retries` to `Config` in `src/config.ts` and thread it through `Client.request`" is.

## 3. Surface decisions before steps

- Identify the genuine forks: approach A vs B, scope in vs out, compatibility constraints.
- For each, either resolve it yourself with a stated rationale, or put it to the user with a recommended default. Don't bury open decisions inside step 4 of 7.

## 4. Sequence for safety

- Order steps so the codebase builds and tests pass after each one where possible.
- Front-load the risky/unknown step — if the approach is going to fail, learn it in step 1, not step 6.
- Mark steps that are hard to reverse; those are the ones needing confirmation.

## 5. Define done

- Attach verification to the plan itself: which tests, which commands, what observable behavior proves the goal.
- List non-goals so the plan can be judged complete rather than merely stopped.

## 6. Treat the plan as live

- When implementation contradicts the plan, update the plan and say so — don't silently diverge, and don't force the code to obey a wrong plan.
