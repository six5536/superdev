# Decision Format

Decisions are AOKF concepts under `knowledge/decisions/`, one file per
decision, named `Dnnn-<slug>.md` (e.g. `D001-event-sourced-orders.md`).
Create the directory lazily — when the first decision is recorded — and
add each new concept to the bundle's `index.md`.

## Template

```md
---
type: Decision
id: decision-event-sourced-orders
title: Event-sourced orders
description: The order write model is event-sourced.
---

{1-3 sentences: what's the context, what did we decide, and why.}
```

That's it. A decision can be a single paragraph. The value is in
recording *that* a decision was made and *why* — not in filling out
sections.

## Optional sections

Only include these when they add genuine value. Most decisions won't
need them.

- **Considered options** — only when the rejected alternatives are worth remembering
- **Consequences** — only when non-obvious downstream effects need to be called out
- **Lifecycle** — a revisited decision uses AOKF natively: the new concept declares a `supersedes` link (with the mirroring body link), and the old one gets `status: deprecated`. No prose markers.

## Numbering

The filename's `Dnnn` orders decisions at a glance: scan
`knowledge/decisions/` for the highest number and increment by one. The
`id` is the stable link target and never changes, even when the file is
renamed.

## When to offer a decision record

All three of these must be true:

1. **Hard to reverse** — the cost of changing your mind later is meaningful
2. **Surprising without context** — a future reader will look at the code and wonder "why on earth did they do it this way?"
3. **The result of a real trade-off** — there were genuine alternatives and you picked one for specific reasons

If a decision is easy to reverse, skip it — you'll just reverse it. If it's not surprising, nobody will wonder why. If there was no real alternative, there's nothing to record beyond "we did the obvious thing."

### What qualifies

- **Architectural shape.** "We're using a monorepo." "The write model is event-sourced, the read model is projected into Postgres."
- **Integration patterns between contexts.** "Ordering and Billing communicate via domain events, not synchronous HTTP."
- **Technology choices that carry lock-in.** Database, message bus, auth provider, deployment target. Not every library — just the ones that would take a quarter to swap out.
- **Boundary and scope decisions.** "Customer data is owned by the Customer context; other contexts reference it by ID only." The explicit no-s are as valuable as the yes-s.
- **Deliberate deviations from the obvious path.** "We're using manual SQL instead of an ORM because X." Anything where a reasonable reader would assume the opposite. These stop the next engineer from "fixing" something that was deliberate.
- **Constraints not visible in the code.** "We can't use AWS because of compliance requirements." "Response times must be under 200ms because of the partner API contract."
- **Rejected alternatives when the rejection is non-obvious.** If you considered GraphQL and picked REST for subtle reasons, record it — otherwise someone will suggest GraphQL again in six months.
