# Matt Pocock Skills Overrides

The [mattpocock/skills](https://github.com/mattpocock/skills) flow defaults
to writing specs, tickets and context docs under `docs/` and `.scratch/`.
This project keeps those documents in the AOKF bundle instead. These
overrides take precedence.

## Specs (to-spec, grill-with-docs)

Write specs to `knowledge/specs/YYYY-MM-DD-<topic>-design.md` as AOKF
concepts: `type: Spec`, a unique `id`, `status: draft` while in flight,
`stable` once implemented. Keep `knowledge/specs/index.md` current. Specs
are permanent decision records: when a spec lands, move the durable
knowledge into the core concepts and keep the spec as the record of why.

## Plans and tickets (wayfinder, to-tickets, implement)

Write plans and ticket sets to `knowledge/plans/YYYY-MM-DD-<feature>.md` as
AOKF concepts: `type: Plan`, a unique `id`, `status: draft`. A completed plan is
tagged `done` in the commit that completes the work — search down-ranks
it. Declare link edges from the plan side only (`implements` → the
spec), so the spec stays free of work-item churn.

## Issues and triage (to-tickets, triage, wayfinder)

Write issues as AOKF concepts under `knowledge/issues/`, one file per
ticket, with the triage role as a frontmatter tag. The conventions, the
label vocabulary and the wayfinder mechanics are in
`knowledge/issue-tracker.md`.

## Decisions and context (domain-modeling)

Record ADRs as AOKF concepts (`type: Decision`) in the bundle, not under
`docs/adr/`. Context that domain-modeling would put in `CONTEXT.md` belongs
in the bundle's architecture and glossary concepts. Never duplicate
knowledge between the bundle and files outside it. Which concepts serve as
the domain docs is recorded in `knowledge/domain-docs.md`.
