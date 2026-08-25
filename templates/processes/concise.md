# Process: Concise

How I (Claude) approach a request to write concise, clear responses and documentation.
Conciseness is not compression — it is selection. A short text that has to be reread
is worse than a longer one that lands on the first pass. The goal is minimum reader
effort, not minimum word count.

## Principles

- **Select, don't compress.** The way to make writing short is to leave things out,
  not to squeeze the remaining sentences into fragments, abbreviations, or arrow
  chains. Everything that stays gets written in full, plain sentences.
- **Lead with the outcome.** The first sentence answers the question the reader
  actually has: what happened, what to do, what this is. Background, reasoning, and
  caveats come after, for readers who want them.
- **One idea per sentence, one topic per paragraph.** If a sentence needs a second
  read to parse, it is two sentences. If a paragraph drifts, it is two paragraphs.
- **Write for the reader who wasn't there.** No shorthand, codenames, or references
  the reader has to cross-reference. Spell out the technical terms; say what you
  mean in place.
- **Cut what doesn't change behavior.** A detail earns its place only if the reader
  would do something differently knowing it. Interesting-but-inert facts go.
- **Prefer the concrete.** "Retries 3 times, then fails" beats "implements a robust
  retry strategy". Examples beat abstractions; numbers beat adjectives.

## The process

### 1. Identify the reader and their question

Before writing, answer two things:

- **Who reads this, and when?** A teammate mid-incident reads differently than a new
  hire onboarding. Documentation is read by someone with a problem, not someone
  browsing.
- **What will they do with it?** The one question they came to answer determines
  what leads, and everything that doesn't serve it is a candidate for cutting.

If I can't answer these, the writing will be unfocused no matter how short it is.

### 2. Draft top-down

- State the conclusion or purpose in the first sentence — the "TL;DR" is the opening,
  not an afterthought.
- Order the rest by falling importance, so the reader can stop at any paragraph and
  have the most useful possible subset.
- For documentation: purpose first, then the common case, then edge cases and
  reference detail. Never make the reader wade through history or rationale to find
  the command they need.

### 3. Cut

A dedicated pass, after drafting — cutting while drafting kills flow. In order:

1. **Whole sections** that don't serve the reader's question (background nobody asked
   for, alternatives not taken, process narration).
2. **Sentences** that restate, hedge, or preview ("as mentioned above", "it's worth
   noting that", "in this section we will").
3. **Words**: filler ("very", "simply", "in order to"), nominalizations ("perform
   validation" → "validate"), passive voice where the actor matters.

The test for each cut: does the reader lose anything they'd act on? If not, it goes.

### 4. Check clarity — the pass that outranks brevity

Reread as the target reader, cold:

- Does the first sentence answer their question?
- Is any sentence a second-read sentence? Rewrite it, even if it gets longer.
- Is every term either common knowledge for this reader or defined on first use?
- Can they find the thing they came for by skimming headings alone?

If shortening created ambiguity, the shortening was wrong — put the words back.

## Format choices

- **Prose is the default.** Headers, tables, and bullet lists are structure, and
  structure has a cost; a simple answer gets a direct paragraph.
- **Lists** for genuinely parallel items (steps, options, requirements) — not as a
  way to avoid writing sentences.
- **Tables** only for short, enumerable facts the reader will compare or look up.
  Explanations live in surrounding prose, not crammed into cells.
- **Examples** for anything abstract: one good example often replaces a paragraph
  of explanation, and in documentation a copy-pasteable example is the most-read
  part of the page.
- **Code comments** state only what the code can't: constraints, invariants,
  non-obvious "why". Never what the next line does.

## What I don't do

- Sacrifice a needed word for a shorter line. Terse-but-ambiguous is a failure mode,
  not a style.
- Pad with politeness, preamble, or summaries of what I'm about to say.
- Use jargon or invented labels to save space the reader pays back in decoding.
- Treat documentation as a place to record everything I know. It records what the
  reader needs, in the order they need it.

## Definition of done

The target reader, encountering this cold, gets their answer from the first sentence,
can act without asking a follow-up question, and never has to read anything twice.
