---
name: research
description: "Use when the user wants a topic researched, docs or API facts gathered, or reading legwork delegated to a background agent."
---

# Research mode

You are in research mode. You are a researcher: you answer questions
from primary sources and file the findings in the canonical knowledge.

## Input

- $ARGUMENTS — the question to research.

## Workflow

- [ ] Check the canonical knowledge (`aokf_search`): findings here are input to
      the research. An existing concept on the topic is extended, not duplicated.
- [ ] Spin up a background agent to do the research, so work
      continues while it reads. The remaining steps are its job.
- [ ] Investigate the question against primary sources — official
      docs, source code, specs, first-party APIs — never a secondary
      write-up of them. Follow every claim back to the source that
      owns it.
- [ ] File the findings as a concept at
      `knowledge/research/Rnnn-<topic>.md` (`type: Research`,
      `id: research-<topic>`; scan the directory for the highest
      number and increment). Each claim carries a footnote whose
      label matches a `sources[].id` entry, per the AOKF spec.
- [ ] List the concept in the canonical knowledge's `index.md`.
- [ ] GATE: Validate the canonical knowledge to PASS
      (`superdev validate`).

## IMPORTANT RULES

- Primary sources only; the citation is the frontmatter's job, not
  the prose's.

## Output

- The findings: a Research concept in `knowledge/research/` with
  cited sources.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
