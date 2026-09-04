---
type: Thing
---

# A faulty thing

## Behaviour

- `P_starts` [event] WHEN asked, the thing SHALL start.
  - `AC_starts-fast` [event] WHEN asked, the thing SHALL start in 1 s.
  - [event] WHEN asked twice, the thing SHALL start once — no key.
  - `AC_starts-once` WHEN asked twice, the thing SHALL start once — no tag.
  - `AC_starts-retired` [event] The thing MUST start — a retired verb.
    - a third-level marker is text of the criterion above it
- `P_stops` [event] WHEN told, the thing SHALL stop — no criterion beneath.
- `P_answers` [state] WHILE running, the thing SHALL answer.
  - `AC_starts-fast` [state] WHILE running, the thing SHALL answer — a repeated key.

## Notes

- `N_timing` measured at p99.
- A plain note, held to nothing but the prohibited pattern.
- A plain note that is TBD.
- `N_Late` measured at p50 — malformed, so plain, and `item-pattern` does not bind it.
- `N_late` Measured at p50 — keyed, and held to the pattern.
