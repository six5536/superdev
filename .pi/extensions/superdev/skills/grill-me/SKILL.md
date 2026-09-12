---
name: grill-me
description: Use when the user asks to stress-test a plan, design, or something else, through questions.
---

# Interview

1. Read the subject and identify the decisions needed to establish a shared understanding. Resolve prerequisite decisions before dependent ones.
2. Investigate questions the repository can answer. Distinguish observed behaviour from proposals and unknowns.
3. Ask one remaining human decision through `superdev_ask`. Give each choice a stable `id` and clear `label`; name the recommended `choiceId` and explain it.
4. Accept a listed or typed answer. On Discuss, return to conversation and keep the question open. On another action or Pause, stop the interview without inventing an answer.
5. Use the answer to choose the next question. Examine boundaries, alternatives, failure cases, and verification until the relevant decisions are settled or the human stops.
6. Summarise the decisions and remaining questions for the caller. An interview does not approve a document or give permission to write it.
