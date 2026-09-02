---
type: Thing
---

# A faulty thing

## Behaviour

- `P_starts` [event] WHEN asked, the thing SHALL start.
- `P_Stops` [event] WHEN told, the thing SHALL stop — the key is malformed.
- [state] WHILE running, the thing SHALL answer — no key at all.

## Stability

- `P_stable` [ubiquitous] The thing SHALL keep its name.
- `P_starts` [ubiquitous] The thing SHALL keep its start — a repeated key.
