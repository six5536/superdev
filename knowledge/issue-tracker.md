---
type: Convention
id: issue-tracker
title: Issue Tracker & Triage
description: Where issues live — one AOKF concept per ticket under knowledge/issues/ — plus the triage label vocabulary.
status: stable
---

Issues live as markdown files in this bundle, not on GitHub. Specs and
plans already live here ([Matt Pocock skills
overrides](/.agents/MATT-POCOCK-SKILLS.md)); issues follow the same
conventions. The engineering skills (`to-tickets`, `triage`, `to-spec`,
`wayfinder`) read this concept to learn where to publish and fetch.

# Conventions

- One feature per directory: `knowledge/issues/<feature-slug>/`.
- The spec, when one exists, is
  `knowledge/specs/YYYY-MM-DD-<feature>-design.md`.
- Implementation issues are one file per ticket at
  `knowledge/issues/<feature-slug>/NN-<slug>.md`, numbered from `01` —
  never a single combined tickets file.
- Each issue is an AOKF concept: `type: Issue`, a unique id
  `<feature-slug>-NN-<slug>`, a `title`, a `description`, `status: draft`
  while open.
- The triage role is a string in the frontmatter `tags` list (see
  [Triage labels](#triage-labels)).
- An issue that implements or affects a spec declares the link from the
  issue side (`references` → the spec's id), so deleting the issue leaves
  no dangling references in the bundle.
- Comments and conversation history append under a `## Comments` heading.
- Issues are ephemeral like plans: delete the file in
  the commit that resolves it — git history is the archive. Durable
  knowledge found while resolving moves into the core concepts.
- Keep `knowledge/issues/index.md` current (open issues, grouped by
  feature heading), and list it from the root `knowledge/index.md`.
  Create the directory with the first issue.

When a skill says "publish to the issue tracker": create a new file under
`knowledge/issues/<feature-slug>/`, creating the directory and index
entries if needed. When a skill says "fetch the relevant ticket": read the
file at the referenced path — the user normally passes the path or issue
number directly.

# Triage labels

The skills speak in five canonical triage roles. This repo keeps the
default strings; a "label" here is a string in the issue's frontmatter
`tags` list.

| Role              | Tag in this repo  | Meaning                                  |
| ----------------- | ----------------- | ---------------------------------------- |
| `needs-triage`    | `needs-triage`    | Maintainer needs to evaluate this issue  |
| `needs-info`      | `needs-info`      | Waiting on reporter for more information |
| `ready-for-agent` | `ready-for-agent` | Fully specified, ready for an AFK agent  |
| `ready-for-human` | `ready-for-human` | Requires human implementation            |
| `wontfix`         | `wontfix`         | Will not be actioned                     |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"),
use the matching tag from this table.

# Wayfinding operations

Used by `/wayfinder`. The **map** is the effort's plan concept; the
**children** are issue files.

- **Map**: `knowledge/plans/YYYY-MM-DD-<effort>.md` (`type: Plan`,
  `status: draft`) — the Notes / Decisions-so-far / Fog body.
- **Child ticket**: `knowledge/issues/<effort>/NN-<slug>.md`, numbered
  from `01`, with the question in the body. A `Type:` line records the
  ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:`
  line records `claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is
  unblocked when every file it lists is `resolved`.
- **Frontier**: scan `knowledge/issues/<effort>/` for files that are
  open, unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set
  `Status: resolved`, then append a context pointer (gist + link) to the
  map's Decisions-so-far.
