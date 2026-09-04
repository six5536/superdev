---
type: Thing
---

# A sound thing

## Behaviour

- `P_starts` [event] WHEN asked, the thing SHALL start.
  - `AC_starts-fast` [event] WHEN asked, the thing SHALL start in 1 s.
  - `AC_starts-once` [event] WHEN asked twice, the thing SHALL start once.
    - a third-level marker is text of the criterion above it
    1. a marker of the other kind likewise
- `P_stops` [event] WHEN told, the thing SHALL stop.
  - `AC_stops` [event] WHEN told, the thing SHALL stop within 1 s.

## Notes

- `N_timing` measured at p99.
- A plain note.
