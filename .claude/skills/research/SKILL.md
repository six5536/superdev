---
name: research
description: Investigate a question against high-trust primary sources and capture the findings as a Research concept in the knowledge bundle. Use when the user wants a topic researched, docs or API facts gathered, or reading legwork delegated to a background agent.
---

First check the bundle: `aokf_search` (or `knowledge/` directly) — a
question a concept already answers needs no agent.

Otherwise spin up a **background agent** to do the research, so you
keep working while it reads.

Its job:

1. Investigate the question against **primary sources** — official
   docs, source code, specs, first-party APIs — not a secondary
   write-up of them. Follow every claim back to the source that owns
   it.
2. Write the findings as an AOKF concept at
   `knowledge/research/Rnnn-<topic>.md` (`type: Research`,
   `id: research-<topic>`; scan the directory for the highest number
   and increment). Each claim carries a footnote whose label matches a
   `sources[].id` entry, per the AOKF spec — the citation is the
   frontmatter's job, not prose's.
3. Add the concept to the bundle's `index.md`. The validator must pass:
   `superdev aokf validate knowledge`.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
