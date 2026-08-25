# Process: Debugging / investigating

## 1. Define the question

- Write down the precise symptom and how to observe it. "Slow" becomes "startup takes 8s, expected <2s".
- Establish what changed: last known-good state, recent commits, config or environment differences.

## 2. Gather evidence before hypotheses

- Read the actual error output, logs, and stack traces in full — not the summary of them.
- Reproduce under observation: add targeted logging, use a debugger, bisect commits, or shrink the failing input.
- Record each observation with its source so the chain is checkable later.

## 3. Hypothesize and test cheaply

- Form the smallest set of hypotheses that explain all the evidence — not just the loudest symptom.
- Test the cheapest-to-check hypothesis first; design each check so either outcome teaches something.
- When a hypothesis dies, record it as ruled out with the disproving evidence, and don't revisit it without new data.

## 4. Confirm the mechanism

- The investigation is done when you can narrate the full chain: initial condition → wrong path → observed symptom.
- A fix that works without an understood mechanism is a data point, not a conclusion — say so.

## 5. Distinguish assessment from action

- If the user described a problem or asked a question, the deliverable is the finding — report it and stop; don't apply a fix unasked.
- If a fix was requested, proceed to the bug-fix process (`bug-fix.md`).

## 6. Report

- Conclusion first, confidence level attached. Then the evidence chain, ruled-out hypotheses, and remaining uncertainty (see `templates/investigation.md`).
