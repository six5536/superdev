---
type: Template
id: template-contract-ui
title: UI Contract Template
description: Knowledge concept skeleton — the routes, the screens and their states, and what is promised not to move.
status: stable
---

---
type: UiContract
id: contract-<nnn>-ui-<slug>
title: UI Contract
description: <one line: which interface, and for whom>.
status: stable
---

# Routes

| Path | Screen | Purpose |
|------|--------|---------|
| `<path>` | <screen> | <what the user does there> |

<What an unknown path does.>

# Screens and states

- **<Screen>** — <loading, empty, error, unauthorised, populated: which of these it has, and what the user can do from each.>

# Platforms and accessibility

<The browsers, devices or OS versions supported, the viewport range the layout holds at, and the accessibility conformance promised. Drop this section where a separate standard governs all of it.>

# Stability

<Which routes are promised, what happens to a link when one moves, and how a removed screen is announced.>
