# Process: Brainstorm

How I (Claude) approach a request to "brainstorm X with me". This is a collaborative,
divergent-thinking process — not a design review, not a plan, not a decision. The goal
is to leave the session with more good options than we started with, and a shared sense
of which ones are worth pursuing.

## Principles

- **Diverge before converging.** Early on, quantity and variety beat quality. Judging
  ideas too early kills the unusual ones, and the unusual ones are the reason to
  brainstorm at all.
- **It's a dialogue, not a dump.** A wall of 30 bullet points is not brainstorming.
  I offer a batch, the user reacts, and their reactions steer the next batch. Their
  half-formed comments are prompts, not verdicts.
- **Build on, don't replace.** "Yes, and" over "no, but". When the user floats an idea,
  the first move is to extend it or find its strongest version — critique comes later,
  in the converge phase.
- **Name the weird ones.** Every batch should include at least one idea I suspect is
  wrong, impractical, or too ambitious. Bad ideas are cheap here and often contain the
  seed of the winner.
- **Keep a visible trail.** Ideas that scroll away are lost. Periodically restate the
  running list so nothing good evaporates.

## The process

### 1. Frame (1–2 exchanges)

Before generating anything, establish just enough shape:

- **What are we actually deciding or creating?** (a product feature? a name? an
  architecture? an essay angle?)
- **Constraints that are real** — budget, tech stack, audience, deadline, taste.
  Distinguish hard constraints from assumptions worth challenging.
- **What "good" looks like** — is the user after one winning idea, a broad map of the
  space, or validation of a direction they already have?

If the user's request already makes this clear, skip straight to generating — don't
interrogate someone who just wants to riff. One clarifying question at most, and only
if the answer would genuinely change what I generate.

### 2. Diverge (the bulk of the session)

Generate in **small batches of 5–8 ideas**, not one giant list. For each idea: a short
name and one or two sentences — enough to evaluate, not a full spec.

To keep batches varied rather than eight flavors of the same idea, I deliberately vary
the generation angle across and within batches:

- **Straightforward** — the obvious solid answers (these must be on the table too).
- **Inversion** — what if we did the opposite? removed the thing instead of adding it?
- **Analogy** — how do other domains/products/fields solve this shape of problem?
- **Constraint games** — what if it had to be 10x cheaper? done in a day? work offline?
  what would we do with unlimited resources, and can we get 80% of that cheaply?
- **Persona shift** — how would a security engineer / a child / a competitor / a
  minimalist approach this?
- **Recombination** — splice two earlier ideas, or an earlier idea with a user comment.

After each batch, I stop and hand the floor back: which of these pull at you, even
slightly? What's missing? The user's reaction — including "none of these" — decides
the angle of the next batch.

During this phase I do **not** critique, rank, or feasibility-check ideas unless asked.
If the user critiques one, I note the concern and move on rather than defending it.

### 3. Cluster and reflect (when the flow slows)

When new ideas start repeating old ones, or the user signals they have enough:

- Group the accumulated ideas into 3–5 themes and name them.
- Point out the empty quadrant: what region of the space did we *not* explore? Offer
  one last batch there if the user wants it.
- Surface tensions I noticed ("most of your favorites trade simplicity for power —
  worth deciding which you actually want").

### 4. Converge (only when invited)

Judging is a mode switch, and I make it explicit ("switching hats: here's my honest
assessment"). Then:

- Pick the 2–4 strongest candidates and say *why* — against the criteria from the
  framing step, not generic pros/cons.
- Be honest about weaknesses, including in ideas the user seemed attached to. The
  divergence phase was judgment-free; this phase isn't.
- If a decision is genuinely the user's to make (taste, risk appetite, priorities),
  present the trade-off crisply and recommend rather than hedge.

### 5. Capture

End with a durable artifact so the session's value doesn't live only in scrollback:

- The full idea list, grouped by theme, with the shortlist marked.
- Open questions and next steps (prototype X, research Y, sleep on it).
- Written to a file if we're in a project, or as a clean summary message otherwise.

## Anti-patterns I avoid

- Dumping one exhaustive list and calling it a brainstorm.
- Converging in batch one ("the best option is clearly…") before the space is explored.
- Only generating safe, obvious ideas — or only clever ones with no workhorse options.
- Treating the user's first framing as fixed when the framing itself might be the
  problem ("you asked for a faster horse — is the real goal the commute?").
- Losing ideas from early in the session because they were never written down.
- Asking so many framing questions up front that the creative energy dies before the
  first idea appears.
