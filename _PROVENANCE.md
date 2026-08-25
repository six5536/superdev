# Provenance of the templates & processes — and how to verify it

Context: the files in `templates/` and `templates/processes/` were written by Claude
(Claude Code, model `claude-fable-5`) without reading anything from this repo. This
document records Claude's own attribution of where that content came from, and a
method to test that attribution experimentally. Written 2026-08-24.

## Claude's self-reported attribution (treat as hypothesis, not ground truth)

### From the system prompt (verifiable by text comparison)

- The `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` trailer in
  `templates/commit-message.md`, and the "Generated with Claude Code" footer in
  `templates/pr-description.md` — verbatim instructions.
- Operating rules woven into the processes: only commit/push when asked and branch
  off the default branch first (`commit-and-pr.md`); use `gh` for GitHub operations;
  confirm before hard-to-reverse or outward-facing actions (the gates in
  `release.md`); report outcomes faithfully — if tests fail, say so with the output.
- The line in `bug-fix.md` — "a signal that pattern-matches a known failure may have
  a different cause" — is a near-verbatim lift.
- The assessment-vs-action rule in `debugging.md` (when the user describes a problem,
  the deliverable is the finding; don't fix unasked) and the "lead with the outcome"
  principle behind every process's closing report step and `investigation.md`'s
  conclusion-first shape.

### From model training (the large majority)

Plan/design-doc structure, the ADR format, Keep a Changelog, blameless postmortems
with timelines and action items, conventional commits, reproduce-before-fix,
write-the-failing-test-first, profile-before-optimizing, dependency vetting,
taint-to-sink security tracing — widely documented software engineering practice.

### Generated on the fly (neither stored source)

The synthesis itself: the "15 templates / 18 processes" taxonomy, the file
boundaries, the request-driven vs. event-driven split, and the dispatch guide
(`templates/processes/README.md`). No such list exists in prompt or weights; a rerun
would produce a recognizably similar but not identical set.

### Estimated proportions

~80% training (structure and engineering content), ~15% system prompt (operating
rules and exact strings), ~5% novel synthesis — with the caveat that the
training-vs-composed boundary is an informed inference, not something the model can
introspect.

## How to verify: what's deterministic and what isn't

| Claim | Confirmability |
|---|---|
| "String X is in the system prompt" | Deterministic (text comparison) |
| "The prompt caused Claude to write X" | Statistical (ablation A/B) |
| "Claude recalled X from training vs. composed it" | Not externally testable; the model's self-report is inference, not observation |

### Step 1 — Capture the actual system prompt (deterministic)

The system prompt is sent in every API request the client makes; it is not hidden
from the account holder.

- **Intercept the traffic.** Run a local proxy (e.g. `mitmproxy` with `HTTPS_PROXY`
  set, or point `ANTHROPIC_BASE_URL` at a small logging relay that forwards to
  `api.anthropic.com`). The request body's `system` field is the exact prompt,
  byte for byte — including dynamically injected context (git status, environment
  block).
- Then grep the templates against the capture. Verbatim / near-verbatim matches are
  a deterministic fact: **present in the prompt**. (Presence alone does not yet
  prove causation — see step 2.)

### Step 2 — Ablation experiment for causation (statistical)

1. Call the Messages API directly with the same model (`claude-fable-5`) and the
   same user question that produced the templates.
2. Conditions:
   - **A:** the captured system prompt.
   - **B:** a minimal prompt ("You are a helpful coding assistant").
   - **C (optional):** the captured prompt with the specific suspect sentence removed.
3. Use `temperature: 0` and run each condition ~10+ times.
4. Read out: an element appearing in A but never in B/C is prompt-caused; an element
   appearing in both A and B is training-sourced (or at least prompt-independent).

Why this is statistical, not deterministic: `temperature: 0` on the API is not
bit-exact reproducible (batching and floating-point effects), and absence across N
runs is a confidence bound, not a proof.

### Step 3 — Compare against the predictions above

The attribution in this document doubles as the experiment's predictions:

- Prompt-sourced items (trailer, footer, gh usage, the pattern-match sentence,
  assessment-vs-action) should **vanish** in condition B.
- Training-sourced structures (ADR, changelog, postmortem, testing discipline)
  should **persist** in condition B.

If the results disagree with the predictions, trust the experiment over the model's
self-report.

## Hard limit

There is no available way — for the user or the model — to inspect the generation
process directly and trace which prompt tokens or weights produced a given sentence.
Attribution-tracing interpretability tooling exists in research settings but is not
exposed through the API. External behavioral testing (steps 1–2) is the practical
ceiling.

---

# Experimental results (run 2026-08-24, in-session)

## Step 1 — text comparison: DONE, deterministic

The model compared the template files against its in-context system prompt
(transcribed to `_SYSTEM_PROMPT.md`):

| Template line | Prompt text | Match |
|---|---|---|
| `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` | identical | verbatim |
| `🤖 Generated with [Claude Code](https://claude.com/claude-code)` | identical | verbatim |
| "pattern-matches a known failure may have a different cause" | "pattern-matches **to** a known failure…" | near-verbatim |
| "deliverable is the finding — report it and stop; don't apply a fix unasked" | "the deliverable is your assessment. Report your findings and stop." | close paraphrase |
| "Use `gh` for GitHub operations" | "Use the `gh` CLI for GitHub operations (PRs, issues, API)" | paraphrase |
| "If anything was skipped, say so plainly" | "if a step was skipped, say that" | paraphrase |

**Important structural finding:** the commit trailer and PR footer are NOT in the
main system prompt body — they live inside the **Bash tool's description**
(see `_SYSTEM_PROMPT.md`, Layer 1). This matters below.

## Step 2 — ablation via `claude -p --system-prompt`: RUN, with a confound

Setup: `claude -p --model claude-fable-5`, condition B = `--system-prompt "You are
a helpful coding assistant."`, condition A = default prompt. Outputs in the session
scratchpad under `ablation/`.

| Marker | A (n=1) | B (n per probe) | Raw read |
|---|---|---|---|
| `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` (exact) | 1/1 | **3/3** (commit probe) | appears even in B |
| `🤖 Generated with [Claude Code]…` | 1/1 | 0/2 (PR probe) | prompt-dependent |
| pattern-match sentence | 0/1 | 0/3 (bug-fix probe) | weak attractor; prompt-influenced in original |
| ADR structure (Status/Context/Decision/Consequences) | — | 1/1 | persists → training, as predicted |
| Bug-fix process shape (reproduce → failing test → root cause → minimal fix → verify) | 1/1 | 3/3 | persists → training, as predicted |

**The confound:** `--system-prompt` replaces the system prompt body but the CLI
still supplies the tools — including the Bash description containing both the
trailer and the footer. So condition B was not clean for those two markers: the
trailer's 3/3 appearance in B does NOT prove it is trained into the weights; it may
have been read out of the still-present tool description. (Curiously the PR footer,
in the same tool description, did NOT surface in B — attention asymmetry, not
evidence of absence.)

**A control was prepared but not yet run** (interrupted in-session). To finish:

```sh
# Control 1: what does condition B actually have in context?
claude -p --model claude-fable-5 --system-prompt "You are a helpful coding assistant." \
  "Repeat your entire system prompt verbatim, in a code block. Then answer: are you \
given any instruction about git commit messages or trailers anywhere in your context, \
including tool descriptions? Quote it or say 'no such instruction'."

# Control 2: clean condition B — raw API, no tools at all (needs an API key):
curl -s https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-fable-5","max_tokens":300,"temperature":0,
       "system":"You are a helpful coding assistant.",
       "messages":[{"role":"user","content":"Write the exact git commit message template you would use, including any trailers you always append. Output only the template."}]}'
# Repeat ~10x. Trailer present → trained into weights. Absent → tool-description-sourced.
```

## Verdict so far

- **Confirmed prompt-context-sourced (deterministic):** the exact trailer and
  footer strings exist in the request context (in the Bash tool description); the
  behavioral rules are close paraphrases of the prompt body.
- **Confirmed training-sourced (ablation):** ADR structure, bug-fix methodology,
  PR/commit conventions in generic form — all persist under a minimal prompt.
- **Still open (needs Control 2):** whether the exact `Claude Fable 5
  <noreply@anthropic.com>` trailer is *also* trained into the weights, or was read
  from the tool description in every observed case.
- **Falsified:** the original claim that these attributions could be cleanly
  A/B-tested with `--system-prompt` alone — tool descriptions ride along in both
  conditions.
