# Superpowers Overrides

The [Superpowers](https://github.com/obra/superpowers) skills default to
writing under `docs/superpowers/`. This project keeps those documents in the
AOKF bundle instead. These overrides take precedence; never create
`docs/superpowers/`.

## Specs (brainstorming skill)

Write specs to `knowledge/specs/YYYY-MM-DD-<topic>-design.md` as AOKF
concepts: `type: Spec`, a unique `id`, `status: draft` while in flight,
`stable` once implemented. Keep `knowledge/specs/index.md` current. Specs are
permanent decision records: when a spec lands, move the durable knowledge
into the core concepts and keep the spec as the record of why.

## Plans (writing-plans skill)

Write plans to `knowledge/plans/YYYY-MM-DD-<feature>.md` as AOKF concepts:
`type: Plan`, a unique `id`, `status: draft`. A completed plan is
tagged `done` in the commit that completes the work — search down-ranks
it. Declare link edges from the plan side only (`implements` → the
spec), so the spec stays free of work-item churn. `implements`
is a custom rel; the validator's "read as relates-to" warning on it is
expected.
