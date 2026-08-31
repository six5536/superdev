---
type: Schema
id: schema-contract-ui
title: UI Contract Schema
description: The user-facing surface — its routes, its screens and their states, the platforms supported, and the stability promise, a public contract.
---

# UI Contract Schema

Structural rules for one public user-interface contract, filed at
`contract-{nnn}-ui-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. The standing surface a user or a link depends
on: which routes exist, what each screen can be showing, and what is promised
not to move.

A feature request describes one change and is settled once it ships;
this document is the surface as it now stands, and is read by anyone rebuilding or linking to it. Routes here
are the ones a browser or a deep link addresses, not the API paths behind them
— those belong to the [rest][sokf:schema-contract-rest] or
[graphql][sokf:schema-contract-graphql] contract.

````yaml
description: >
  One user-facing surface — the routes it answers, the screens behind them and
  the states each can be in, the platforms and accessibility level promised,
  and what is promised not to move.
line-limit: 400

frontmatter:
  type:
    required: true
    const: UiContract
  id:
    required: true
    pattern: '^contract-\d{3}-ui-[a-z0-9-]+$'
    description: >
      contract-{nnn}-ui-{slug}, the slug naming which user interface. The
      number is the next free one across every contract, public and
      internal together and every lifecycle folder — a duplicate is
      an error.
  title:
    required: true
  description:
    required: true
  lifecycle:
    enum: [active, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^UI contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Routes"
    level: 2
    required: true
    content: table
    columns: [Path, Screen, Purpose]
    description: >
      Every addressable route, with its parameters shown in the path. A route
      reachable only by navigation is still a route. Include what an unknown
      path does.
  - heading: "Screens and states"
    level: 2
    required: true
    content: bullet-list
    description: >
      One entry per screen: the states it can be in — loading, empty, error,
      populated, unauthorised — and what the user can do from each. A state
      with no design is a state a rebuild will get wrong.
  - heading: "Platforms and accessibility"
    level: 2
    content: prose
    description: >
      The browsers, devices or OS versions supported, the viewport range the
      layout holds at, and the accessibility conformance promised. Omit where a
      separate standard governs all of it, and name that standard under Routes.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    description: >
      Which routes are promised, what happens to a link when one moves, and how
      a removed screen is announced. Redirects are the contract; a 404 on a
      previously working link is a break.

example: |
  ---
  type: UiContract
  id: contract-001-ui-widget-app
  title: UI Contract
  description: The widget web app — four routes, links promised for a year.
  lifecycle: active
  ---

  # UI contract: widget app

  The widget web app: four routes, links promised for a year.

  ## Routes

  | Path | Screen | Purpose |
  |------|--------|---------|
  | `/` | Widget list | every widget in the tenant, newest first |
  | `/w/:id` | Widget detail | one widget, its history and its actions |
  | `/w/:id/edit` | Widget editor | rename and retag; requires the editor role |
  | `/settings` | Settings | tenant name, members, API tokens |

  An unknown path renders the not-found screen with a link to `/`, and never
  redirects, so a mistyped link is visibly wrong rather than silently rerouted.

  ## Screens and states

  - **Widget list** — loading (skeleton rows), empty (a create prompt, no empty
    table), error (a retry, the last good list kept on screen), populated.
  - **Widget detail** — loading, not-found, unauthorised (a request-access
    prompt, never a bare 403), populated.
  - **Widget editor** — populated, saving (the form disabled, not hidden),
    conflict (someone else saved first; both versions shown), error.
  - **Settings** — populated only. It loads with the shell, so it has no
    loading state of its own.

  ## Platforms and accessibility

  The last two major versions of Chrome, Firefox, Safari and Edge, and Safari
  on iOS 17 or later. The layout holds from 360 px to 2560 px wide. WCAG 2.2 AA:
  every action reachable by keyboard, every state announced to a screen reader,
  and no colour used as the only carrier of meaning.

  ## Stability

  The four routes above are promised for a year from any release that
  announces a change. A route that moves leaves a permanent redirect at the old
  path for that year — a working link that starts 404ing is a break, not a
  cleanup. A removed screen is announced in the release notes one release ahead.
````

<!-- sokf:links -->
[sokf:schema-contract-graphql]: /knowledge/schemas/contract-graphql.md
[sokf:schema-contract-rest]: /knowledge/schemas/contract-rest.md
