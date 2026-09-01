# SOKF — Superdev Open Knowledge Format

**Version:** 0.4
**Status:** Draft
**Date:** 2026-08-28

SOKF is a format for canonical project knowledge: a directory of markdown
files with YAML frontmatter, kept inside the project's repository and
maintained largely by AI agents. Content is unrestricted; architecture
notes, decisions, conventions, and playbooks are typical. SOKF is a
superset of the Open Knowledge Format (OKF) v0.2.

The frontmatter (§1) identifies each document and carries its trust
state: what kind of document it is, what it describes, what it derives
from, how it relates to other documents, who has verified it. The
writers are mostly LLMs, and an LLM cannot be relied on to record a fact
— an author, a timestamp, a review — truthfully, while a validator
cannot detect a fabricated value that parses. Frontmatter therefore
carries only fields checkable against the repository, fields that are
explicitly judgements, and one field, closed to agents, that records
verification (§7). Two rules follow:

1. Every frontmatter field has a **write class** (§4) stating who may
   write it. Fields an agent cannot produce truthfully are closed to
   agents; a git diff check enforces this (§10).
2. No metadata git already records is stored. Authorship, change times,
   and prior content come from version control, not from frontmatter,
   where a stored copy could diverge.

> The key words MUST, MUST NOT, SHOULD, and MAY are used as defined in
> RFC 2119.

---

## 1. Terminology

- **SOKF knowledge**: the directory tree of knowledge documents this
  format describes; always named in full, since "knowledge" alone is an
  ordinary English word.
- **Concept**: one unit of knowledge, one markdown file.
- **Frontmatter**: the YAML block delimited by `---` at the top of a
  file. **Body**: everything after it.
- **Manifest**: the optional `manifest.sokf.yaml` at the SOKF knowledge
  root, describing the SOKF knowledge as a whole (§2).
- **Source**: a material a concept derives from, recorded in `sources`.
- **Link**: a directed, typed relationship from one concept to another,
  declared in `links` (§8).
- **Actor**: a string identifying who did something: `human:<id>` for a
  person, `process:<id>` for a deterministic automated process,
  `<producer>/<version>` for an agent or tool (§7).
- **Agent**: an LLM-driven writer.

## 2. Knowledge structure

The SOKF knowledge is a directory inside the repository (for example
`/knowledge`). Subdirectories group concepts however suits the
project; paths carry no mandated meaning, and identity does not depend
on them (§5).

```
<knowledge>/
  manifest.sokf.yaml  # Optional knowledge manifest.
  index.md            # Optional directory listing (§9).
  <concept>.md
  <subdirectory>/
    index.md
    <concept>.md
```

Reserved files are `manifest.sokf.yaml` (knowledge root only) and
`index.md` (any directory); neither is a concept. Every other `.md`
file is a concept. There is no change-log file: git log is the change
history.

The manifest declares the SOKF knowledge:

```yaml
sokf: "0.4" # spec version the SOKF knowledge targets
name: example-knowledge
description: Knowledge for the example project.
```

`producer`, `generated`, and `counts` keys are stamped (§4): written by
tooling when the SOKF knowledge is exported for use outside the
repository, never present in the working tree.

## 3. Concept documents

A concept is a UTF-8 markdown file: a YAML frontmatter block, then a
markdown body. `type` is the only required field. Type values are
free-form and unregistered; consumers must tolerate unknown types. For a
code project, expect values like `Module`, `Subsystem`, `Decision`,
`Convention`, `Playbook`, `Reference`.

```markdown
---
type: Module
id: planner
title: Planner
description: Pure planning stage; computes actions without touching the filesystem.
resource: /crates/lib/example-core/src/planner.rs
tags: [core, planning]
sources:
  - id: config-src
    resource: /crates/lib/example-core/src/config.rs
    title: Config source
links:
  - rel: depends-on
    to: config
    note: Reads mappings and the conflict policy.
---

# Role

The planner reads [config][sokf:config] and filesystem state and emits a
list of actions.[^config-src] It never writes.

[^config-src]: Config source

<!-- sokf:links -->
[sokf:config]: /knowledge/config.md
```

The `depends-on` entry is mirrored by the `[config][sokf:config]` body
link, as §8 requires. The link names the target's `id`, not its path;
the definition below it is generated (§9).

The body is standard markdown. Prefer structure (headings, lists,
tables, fenced code) over freeform prose; there are no required
sections. Per-claim attribution uses footnotes keyed to `sources[].id`
(§6).

Knowledge MUST NOT be duplicated between the SOKF knowledge and the
rest of the repository. When a document has to exist outside it (a README,
contributor docs), the concept covering that ground carries a concise
summary and cites the file in `sources`; it does not copy the content.
Knowledge with no such external home lives here only.

## 4. Write classes

Every frontmatter field and manifest key belongs to one class:

| Class        | Who may write                                                                          | Enforced by           |
| ------------ | -------------------------------------------------------------------------------------- | --------------------- |
| `open`       | Anyone: humans, agents, tooling.                                                       | Nothing to enforce.   |
| `restricted` | Humans and deterministic processes. **Agents MUST NOT add, edit, reorder, or delete.** | Diff check (§10).     |
| `stamped`    | Deterministic export tooling, at export time. **MUST NOT appear in the working tree.** | Document check (§10). |

Field reference:

| Field         | Class      | Notes                                                                              |
| ------------- | ---------- | ---------------------------------------------------------------------------------- |
| `type`        | open       | Required. Kind of concept.                                                         |
| `id`          | open       | Stable identity slug; immutable once assigned (§5).                                |
| `title`       | open       | Display name; consumers may fall back to the filename.                             |
| `description` | open       | One-line summary, used by indexes and previews.                                    |
| `tags`        | open       | Short labels for grouping concepts across directories.                             |
| `resource`    | open       | Repo path or URL of the thing the concept describes. Absent for abstract concepts. |
| `status`      | open       | `draft` \| `stable` \| `deprecated`; absent ⇒ `stable`.                            |
| `sources`     | open       | Entries carry `resource`, `id`, `title` only (§6).                                 |
| `links`       | open       | Typed relationships to other concepts (§8).                                        |
| `verified`    | restricted | §7.                                                                                |
| `generated`   | stamped    | `{ by, at }`, derived from git history at export. Never hand-written.              |

Producer-defined extension keys are permitted and default to `open`.
Consumers must not reject documents over unknown keys. A project
classifies a new key in its own conventions: `open` if the value is
checkable against the repository or is openly a judgement, `restricted`
or `stamped` if it asserts a fact nobody can check.

## 5. Identity

`id` gives a concept an identity that survives file moves.

- An `id` is a slug: lowercase, words separated by `-`. It MUST be
  unique within the SOKF knowledge.
- Once assigned, an `id` MUST NOT change, even when the file is renamed
  or moved. For agent commits the diff check enforces this (§10); a
  human may change one deliberately and take responsibility for the
  broken references.
- When `id` is absent, the concept's identity is its repo-root-relative
  file path.
- `id` is the preferred target for typed links (§8): it is stable where
  paths are not.

## 6. Sources

`sources` records what a concept derives from:

```yaml
sources:
  - id: planner-src
    resource: /crates/lib/example-core/src/planner.rs
    title: Planner source
  - id: clap-docs
    resource: https://docs.rs/clap/latest/clap/
    title: clap documentation
```

- `resource` (required): a repo-root-relative path or a URL.
- `id`: stable key for footnote attribution, local to the file. It is
  unrelated to the concept `id` of §5. Required when the body cites the
  source.
- `title`: optional display label.

To attribute a specific claim, use a markdown footnote whose label is a
`sources[].id`:

```markdown
The planner never writes to the filesystem.[^planner-src]

[^planner-src]: Planner source
```

The footnote label is the join key into `sources`. It is a key, not a
position, so reordering the list does not change what a footnote
attributes.

There is no per-source author, usage count, or last-modified date.

## 7. Verification

`verified` records who has confirmed a concept's content against the
things it describes. It is a list of `{ by, at }` entries; a bare
mapping is read as a one-element list.

```yaml
verified:
  - { by: human:rsewell, at: 2026-08-04T09:00:00Z }
  - { by: process:link-checker, at: 2026-08-04T02:00:00Z }
```

- `by`: a `human:<id>` or `process:<id>` actor. The agent actor form
  never appears here — an agent verifying its own output is not
  verification.
- `at`: an ISO 8601 datetime.

Rules:

- An entry may be added only by the actor it names.
- Agents MUST NOT touch the field. When an agent rewrites a
  concept's content, it leaves existing `verified` entries in place;
  whether a verification still applies is derived, not edited (below).
- A verification covers the file as it stood at `at`. A consumer or
  validator compares each entry's `at` against the file's last content
  change in git: verification older than the last change is **lapsed**
  and confers no trust.

**Trust tiers**, derived from the non-lapsed entries, lowest to highest:

- none ⇒ **unverified**
- `process:` actors only ⇒ **machine-confirmed**
- any `human:` actor ⇒ **human-reviewed**

Tiers are advisory signals, not access control. A concept with no
`verified` key is still consumable.

## 8. Relationships

Typed relationships are declared in a `links` frontmatter array. Each
entry is a map:

| Key    | Rule | Meaning                                                      |
| ------ | ---- | ------------------------------------------------------------ |
| `rel`  | MUST | The relationship type (below).                               |
| `to`   | MUST | Target concept: an `id` (preferred) or a `/` repo-root path. |
| `note` | MAY  | One-line explanation of this specific edge.                  |

A link asserts a directed edge from the containing concept to `to`.
Consumers resolve `to` as an `id` first, then as a path.

**Relationship vocabulary**, with defined inverses:

| `rel`         | Inverse          | Meaning                                                                |
| ------------- | ---------------- | ---------------------------------------------------------------------- |
| `relates-to`  | `relates-to`     | Generic association (symmetric).                                       |
| `part-of`     | `has-part`       | Composition or containment.                                            |
| `depends-on`  | `depended-on-by` | Requires the target to function.                                       |
| `references`  | `referenced-by`  | Cites or points at the target.                                         |
| `supersedes`  | `superseded-by`  | Replaces the target; the target is deprecated.                         |
| `implements`  | `implemented-by` | Delivers or realises the target — a plan or issue implementing a feature request or contract. |
| `contradicts` | `contradicts`    | Known conflict (symmetric); resolution belongs in prose.               |

Producers SHOULD use a core value where one fits and MAY introduce
custom values (lowercase kebab-case) where none does. Consumers MUST
read an unknown `rel` as `relates-to` rather than reject it. There is
no `derived-from` value: derivation is recorded in `sources` (§6), and
a consumer treats each repo-internal source as a derivation edge.

Producers SHOULD declare each edge once, from whichever side is more
natural; a consumer building a graph SHOULD synthesise the inverse edge.

**Body links.** A body link whose target is a concept MUST address it by
`id`, as a reference-style markdown link whose label is `sokf:` followed
by that id:

```markdown
The planner reads [config][sokf:config] before it plans.
```

A consumer resolves the label's id against the SOKF knowledge exactly as
it resolves a `to` (above), and MUST NOT resolve through the reference
definition (§9). A `sokf:<id>` label naming no concept is a broken edge,
not an unresolved reference: a consumer reports it rather than rendering
it as literal text. A link to anything that is not a concept — a source
file, a README, a URL — names a path or a URL, as markdown always has.

**Body mirroring.** For every `links` entry the body MUST contain at
least one markdown link to the same target, in either form, so the edge
is visible to a reader of the markdown alone. A body link with no
corresponding `links` entry is an untyped `relates-to` edge; its meaning
lives in the surrounding prose.

## 9. Paths and indexes

- Paths beginning with `/` resolve from the **repository root**. This is
  the recommended form for links and for path-valued fields
  (`resource`, `sources[].resource`).
- Relative paths resolve from the containing file, as standard markdown.
- Absolute URLs work as anywhere else.

Consumers tolerate broken links; the validator fails on them (§10).

**The generated definition block.** A document that carries an
id-addressed body link (§8) MUST carry, at its foot, one HTML comment
line reading exactly `<!-- sokf:links -->` followed by one reference
definition per cited id, each naming that concept's current
repo-root path, in ascending id order:

```markdown
<!-- sokf:links -->
[sokf:config]: /knowledge/config.md
[sokf:planner]: /knowledge/core/planner.md
```

The block is **generated**, not authored: tooling writes it and rewrites
it whenever a concept moves. Nothing else follows it.

It exists so a plain markdown renderer — a repository host, an editor
preview — follows the link. It is not the resolution path: a consumer
resolves the id (§8) and MUST NOT read the block, so a stale or absent
block changes nothing about what a link means. The validator still
reports one; the remedy is to regenerate it (§10).

**The include block.** A concept document MAY materialize another
concept's body in place, between a marker pair:

```markdown
<!-- sokf:include <id> -->
…the named concept's body…
<!-- /sokf:include -->
```

The open marker names a concept by `id`. The content between the
markers is **generated**, not authored: `superdev validate --fix` fills
it with the named concept's body — that body's own definition block
excluded — and refreshes every copy when the source changes. Author the
marker pair, never the content; ids the copied content cites join this
document's definition block. The validator reports a stale, empty or
unresolvable include block as an error (§10). Include blocks do not
nest: a concept that carries one cannot itself be included.

An `index.md` may appear in any directory to list its contents. It
contains no frontmatter. The body is one or more heading-grouped link
lists:

```markdown
# Core

- [Planner](planner.md) - pure planning stage; no filesystem writes.
- [Executor](executor.md) - applies planned actions.
```

Entries should carry the linked concept's `description`. Indexes may be
generated; consumers may synthesise one when absent.

## 10. Validation

Two deterministic layers.

**Document check** — any time, per file:

1. Frontmatter parses as YAML; `type` is present and non-empty. The
   manifest, when present, parses as YAML.
2. `stamped` fields are absent.
3. `restricted` fields, when present, are well-formed (`verified`
   entries each have `by` in `human:`/`process:` form and an ISO 8601
   `at`).
4. `id` values are valid slugs and unique across the SOKF knowledge.
   `links` entries each have `rel` and `to`.
5. Body links address concepts by id (§8): a `sokf:<id>` label resolves
   to a concept, and a path link resolves to something that is not one.
   The generated definition block (§9) defines each cited id, and only
   the cited ids, at their current paths. An include block (§9) names a
   concept, carries its current body, and does not nest.
6. Fail on what the repository alone settles: a body link, a
   `resource`, a `sources[].resource` or an `index.md` entry naming a
   file that is not there; a footnote label matching no `sources[].id`;
   a `links` `to` that resolves to nothing; a `links` entry with no
   mirroring body link; and `sources` entries the body cites that lack
   an `id`. The tree is the whole input, so there is nothing else the
   answer could depend on.
7. Warn on what it cannot settle alone: a `rel` outside the core set,
   whose meaning is the consumer's to decide.

**Diff check** — per commit, in CI or a hook:

1. Classify the commit's author as agent or not, by whatever identity
   convention the repository uses (committer identity, a trailer, a bot
   account).
2. In an agent commit, every `restricted` field must be byte-identical
   before and after, in every touched concept.
3. In an agent commit, a modified concept keeps its `id`. An agent may
   assign an `id` to a new concept; it must not change an existing one.

## 11. Conformance

Knowledge conforms when all of the following hold.

- Every non-reserved `.md` file passes the document check (§10).
- Every concept has a unique `id`, and a manifest declares `sokf` and
  `name`.
- Every `links` entry has a valid `rel` and a `to` that resolves, and is
  mirrored by a body link (§8).
- Every body link to a concept addresses it by id, and every document
  carrying one carries a current definition block (§9).

A repository conforms if, additionally, its agent commits pass the diff
check (§10). This is independent of whether the SOKF knowledge conforms.

Consumers must be permissive: never reject knowledge for missing
optional fields, unknown `type` values, unknown frontmatter keys,
unknown `rel` values, broken links, or a missing `index.md` or manifest.

This binds a consumer displaying knowledge. It does not bind a validator
checking a repository, which fails on everything §10 lists as a failure:
a validator that never fails is not permissive, it is ignored.

## 12. Versioning

See: [changelog.md](changelog.md).
