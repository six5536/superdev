# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
superdev uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While
superdev is pre-1.0, minor versions may contain breaking changes.

Every released tag needs its own section here. The release workflow refuses to
publish a version it cannot find a heading for.

## [Unreleased]

### Added

- **`superdev validate`.** One command checks both specs this repository
  owns: the AOKF bundle, and the superdev-format files — the skills, the
  schemas and `.agents/core.md` — against the grammar that defines the
  language they are written in. One report, findings grouped by file, one
  exit code, and one PostToolUse hook, so a file both checks have something
  to say about is reported once and the two cannot reach different verdicts.
  The grammar is read from `.agents/format/grammar.yaml`, or from the copy
  inside the binary when a repository has none, and `--doc` prints it as
  prose. `superdev aokf validate` stays as a hidden alias, because the hook
  marker is the lock key in every managed repo — but its positional argument
  is now the scope of the run rather than the bundle, which moved to
  `--bundle <DIR>`. See P006.

### Removed

- **The Node format validator.** `scripts/superdev-format/` and the
  meta-schema beside the grammar are gone: the checks are in the binary, the
  Rust types are the meta-schema, and nothing in the repository needs Node to
  validate the format. The reference's behaviour is held by goldens captured
  from it while it still ran.

- **The AOKF conformance ladder.** A bundle now passes or fails; there is no
  level to grade against. `superdev aokf validate --level` is gone, and the
  report drops `checked_level`, `achieved_level` and each finding's
  `error_at_level`. No verdict changes: every implementation graded at the top
  level already, where a finding was an error exactly when it carried any level
  at all. What does change is that `--level 0` can no longer wave a bundle with
  broken links and no manifest past the hook and the pre-PR check. AOKF is
  0.3; see ADR-017.

### Added

- **Content packs.** A `[[packs]]` array in `.superdev/config.toml` names
  where superdev's skills, templates and scaffolds come from — a git URL with
  a `rev`, or a path on this machine — recorded relative to the repository, so
  a committed pin reads the same in every clone. Entries layer in the order
  written and a later item of the same name wins; an entry naming the source
  the binary's own content is a copy of replaces it rather than layering, so
  what that rev drops leaves the repo. A manifest naming no pack behaves
  exactly as before, and `sync` never adds an entry to one that lacks it.

  A git source is fetched with the user's own `git` — shallow, blobless and
  sparse — so credentials and forge access stay theirs and superdev holds no
  token. Each resolved pack is verified against the digest the lock records
  for its rev and kept under `.superdev/cache/packs/`, so a later `sync` or
  `status` needs the network only for bytes the machine does not have. A
  directory pack records no digest in the lock — it is read afresh every run,
  so there are no pinned bytes to check against, and a value there would be
  rewritten by every commit touching the pack and read by nothing. A rev
  that resolves to different bytes than the lock recorded fails the run and
  writes nothing: re-pinning is the only way forward, and is itself the new
  trust decision. `status` gains `content:` lines naming what it resolved
  from and any item one pack hid from another.

  `init` writes the default entry into the manifest rather than leaving the
  array absent — both resolve identically, but the written one is the pin a
  reader can see and edit. `superdev update` asks that default source for its
  newest content release and moves the pin there, even ahead of what the
  binary embeds: it is how a skill fix reaches you without a new binary. A pin
  naming any other source is reported and left alone, because naming a source
  is your trust decision to make again. With no network the pin moves no
  further than the binary's own default and the run says it could not check,
  and a manifest written by an earlier binary gains the entry on the first
  `update`. The newest release *your binary can read*: the pack is fetched
  before the pin naming it is written, and a release built for a later
  superdev leaves your pin where it is and says why. Written first, that pin
  would have been unreachable by any superdev command — `update` saves the
  manifest before the sync that validates it, and never moves a pin backwards,
  so hand-editing `.superdev/config.toml` was the only way out. That costs one
  extra fetch on a run that actually moves a pin, and nothing on the runs that
  do not. That query is the one request superdev makes that you did not ask
  for, so it is the one on a clock: a few seconds, after which a network that
  neither answers nor refuses is reported as unreachable rather than holding
  the command for however long your OS takes to give up. A clone is not
  bounded — you pinned that pack and asked for it, and a slow link is not
  superdev's to give up on. No git call prompts for credentials, so a source
  you cannot read anonymously fails rather than waiting for you to type. `source` accepts `github:owner/repo` and `gitlab:owner/repo` as
  shorthand, and otherwise a git URL over `https://`, `ssh://` or `file://` —
  the scp form and a bare ssh alias included, so your ssh config and your
  mirrors keep working.

  Releasing is one command per release. `npm run release X.Y.Z` cuts the
  binary and, from the same commit, the content release its pin names, so
  there is no second step to forget and no way to ship a binary pinned at
  content it did not embed. `npm run release:pack` cuts a content release
  alone: superdev's skills, templates and scaffolds change without a new
  binary, no workflow runs and nothing reaches a registry. The pack carries a
  version series of its own — `assets-vA.B.C`, apart from the binary's
  `vX.Y.Z` — so binary semver stays contiguous. A release candidate cuts a
  candidate content tag, which `update` never moves a released pin to; a pin
  left on one comes forward as soon as a release covers it.
- A `.agents/tools.md` scaffold: reach for internal and MCP tools before Bash.
  Written once, like the other agent rules, and imported by
  `.agents/superdev.md` after the capability files.

### Security

- A pack source can no longer choose what superdev runs, or what it arrives
  over. A `[[packs]]` entry arrives with a repository, and git's `ext::`
  transport takes a command as its connection — so on a machine configured to
  permit that transport, running `sync` or `update` in a repo you cloned ran
  whatever its manifest named. A source may now name only `https`, `ssh` or
  `file`, and a `<name>::<address>` remote helper is refused as one whatever
  its address; anything else is refused when the manifest is read, naming the
  source and the transport, before superdev spawns anything. Every git call
  superdev makes then refuses every transport it did not admit, `git://` and
  `http://` by name — neither authenticates, so anyone on the path could
  answer for a pack, and because a source keys the same however it is spelled,
  the pack they answered with would be the one that replaces superdev's own
  content. A source or rev beginning with `-` is refused when the manifest is
  read, and operands are passed after `--`. A stock git already refused
  `ext::`; what changes is that superdev no longer depends on your
  configuration to say so.
- A pack cannot ship a file it does not contain. A symlink in a pack tree was
  followed when it pointed at a file, so a pack could name a link to anything
  the user running superdev could read and have those bytes written into the
  repository as pack content. A symlink anywhere in a pack — the tree, the
  pack's own root, its `pack.toml` — now stops the run naming the path.
  Refused rather than stepped over, because a pack that resolves without the
  item its author meant a link to stand for is a pack shipping less than it
  declares, and neither `sync` nor `status --drift` would have said so. If you
  were deduplicating a file with a link, copy it instead.

  For a fetched pack, git's index decides what a link is rather than the
  filesystem. Windows without `core.symlinks` checks a link out as a plain
  file holding the target's path, which no filesystem check can tell from
  content — so the same revision used to digest differently there, and a lock
  committed from Linux failed on Windows saying only that the bytes did not
  match. It now fails on both, naming the file. A submodule under the pack is
  refused on the same answer: the shallow clone leaves it empty, so the pack
  would have shipped an item with nothing in it.

### Fixed

- The lock describes what is on disk, not only what the last run wrote. A file
  edited into agreement with what superdev ships — the way a contributor tries
  a change before shipping it — kept the hash of what it replaced, so the next
  run that touched it announced an edit nobody had made and backed the file
  up. The same staleness on a managed `.mcp.json` or `.claude/settings.json`
  key was worse: superdev decides whether such an entry is its own to remove
  by that hash, so a stale one left its registration behind for good when the
  capability was disabled.

- `superdev update` said it moved pins to this binary's defaults. It stopped
  meaning that when it began asking the pack source for a newer release — the
  one verb that reaches the network, describing itself as one that does not.
  Corrected in `--help`, the man page and the completions, which all come from
  the same line. The README now says where superdev's content comes from at
  all: what a pack is, how entries layer, and that content releases under its
  own tags.

### Changed

- The agent-rules files are each wrapped in a tag naming what they are —
  `<coding-rules>`, `<professionalism>`, `<code-exploration>` — as
  `.agents/process.md` and the aggregator already were. `.agents/codegraph.md`
  is owned, so it is rewritten on the next `sync` (backed up first); the rest
  are scaffolds and existing repos keep their own copies.

## [0.2.0] - 2026-08-25

### Changed

- **Breaking:** the knowledge-carried skill set is replaced by the process
  layer. The derived engineering skills (code-review, codebase-design,
  diagnosing-bugs, domain-modeling, tdd, to-spec, to-plan, wayfinder and the
  rest, with their MIT licence file) are swept on the next `sync` — backed up,
  as any removal is — and in their place land the eight workflow phases
  (`frame`, `spec`, `interface-design`, `feature-plan`, `build`, `verify`,
  `integrate`, `accept`) and their support skills (`adhoc-plan`, `bootstrap`,
  `brainstorm`, `grill-me`, `handoff`, `how-do-i`, `maintain`, `prototype`,
  `research`). `aokf-bootstrap` and `aokf-maintain` continue as `bootstrap`
  and `maintain`.
- The agent-rules scaffolds grow up: `.agents/prose.md` becomes
  `.agents/professionalism.md`, a new `.agents/process.md` describes the
  eight-phase workflow, and the `.agents/superdev.md` aggregator imports
  professionalism, process, then coding. Existing repos keep their own copies
  (scaffolds are write-once); fresh repos get the new set.
- The skill pack slims to `double-check` and `template-update`; `humanise`
  and `self-improve` leave the pack and are swept on sync unless marked
  custom.
- The init hint now says `run /bootstrap in Claude Code`.

### Added

- The seeded knowledge bundle now carries what the workflow skills reference:
  the document template library at `knowledge/templates/` (owned, so it
  versions with the skills), the issue-tracker convention, and a plans index
  beside the specs index — all listed from the bundle index.

## [0.1.0] - 2026-08-19

### Added

- A second project template, `web-react-android-ios-native`: one product as
  three native codebases — a React web app, a Kotlin/Jetpack Compose Android
  app and a SwiftUI iOS app, each a hello-world stub that builds and passes
  CI. It ships the tooling that makes that stack workable for agents: an HTTP
  debug server compiled into debug builds only, an MCP server wrapping it so
  an agent can drive a running app, a dev CLI for the
  build/install/launch/logs/screenshot loop, an Android-capable dev
  container, and a fastlane release pipeline keyed off a single
  `release/release.yaml`. Two artefacts can't be seeded and are bootstrapped
  instead — the Gradle wrapper jar and the Xcode project — both documented
  in the template's own `docs/BUILD.md`
- Two template tokens for spellings the slug can't express:
  `{{superdev:project-compact}}` (hyphens dropped) for reverse-domain app
  ids, which Android and iOS constrain in opposite directions, and
  `{{superdev:project-pascal}}` for Swift and Kotlin type names, Xcode
  schemes and Gradle root projects. `template render` prints both alongside
  the existing values
- Project templates: `init --template rust-npm` (or a prompt, on a TTY)
  seeds a new repo with a Rust CLI workspace deployed as prebuilt binaries
  through npm — crates, launcher and platform packages, CI and release
  workflows, repo docs, policy configs, and a dev container that brings up
  the pinned toolchain alongside superdev's own tooling, all
  token-substituted with the project's name. Template files are
  write-once scaffolds: existing files
  win, and `sync` never touches them. `--template none` and `--name` script
  the answers; `.superdev/config.toml` records the choice under `[template]`
- Knowledge-owned skills: the aokf component carries its lifecycle skills —
  the new `aokf-bootstrap` (harvest a repo's existing docs into the bundle,
  then interview the owner to flesh out the seeded skeleton) and the
  relocated `aokf-maintain` — plus the validation hook, so all three
  exist exactly where a bundle exists and a `--no-knowledge` repo gets no
  hook. `[knowledge]` takes a `custom` list like the other skill-writing
  capabilities, and a knowledge-enabled `init` ends with the
  `/aokf-bootstrap` hint
- Search down-ranks settled work: sections of a `deprecated` concept, or
  one tagged `done`, `resolved` or `wontfix`, score lower after fusion, so
  finished plans and issues stop crowding live knowledge. The index schema
  changed; the cache rebuilds itself on the next call
- A fuller knowledge seed: `init` now scaffolds a starter concept skeleton
  (glossary, architecture, testing strategy and the rest) instead of a
  three-file stub, ready for agents to fill in
- Project scaffold: CI and release machinery, npm launcher + platform
  packages, cargo workspace, and the AOKF knowledgebase
- `superdev init`, `status`, `sync` and `update`: set a repo up for
  agent-driven development and keep it matching the blueprint compiled into
  the binary. `status` exits 1 when there is work to do; a failed apply rolls
  back and reports anything it could not undo
- `.superdev/config.toml` (what the repo wants) and `.superdev/lock.toml`
  (what was applied, with hashes of superdev-owned files), plus a gitignored
  `.superdev/cache/` for backups
- Managed capabilities: `knowledge` (a native AOKF bundle), `code-index`
  (codegraph), `workflows` (Superpowers), `frontend` (Anthropic's
  frontend-design plugin) and `skills` (superdev's own pack, below); each can
  be disabled with `init --no-<capability>`
- `workflows` and `code-index` install from checksum-verified release
  bundles pinned in the binary, so `update <capability>@<version>` refuses
  an explicit version for them — bare `update` moves them to the binary's
  pins
- `superdev mcp aokf`: an MCP server over the knowledge bundle with four
  read-only tools — `aokf_search`, `aokf_read`, `aokf_graph` and
  `aokf_overview`. Search is hybrid (BM25 plus a pinned local embedding
  model, fused by reciprocal rank fusion) and degrades to lexical-only when
  no model is available. The index sits in `.superdev/cache/aokf-index/` and
  re-syncs lazily on every call, so edits are visible to the next question
- `superdev aokf validate` and `superdev aokf index`: validate the bundle
  against the AOKF spec (exit 1 on errors, `--json`, `--level`,
  `--repo-root`) and force a full index rebuild
- Optional `[knowledge.embeddings]` in `.superdev/config.toml` to embed
  through an API instead of the local model; the key comes from the
  environment, never the file
- `init` registers the server in `.mcp.json` under
  `mcpServers.superdev-aokf`, merging into whatever servers are already
  there
- The `skills` capability: five skills (aokf-maintain, double-check,
  grill-me, humanise, self-improve) written into `.claude/skills/` as
  superdev-owned files, plus a PostToolUse hook in `.claude/settings.json`
  that runs `superdev aokf hook validate` and blocks edits that break the
  bundle. Claude Code loads both natively — nothing to install
- Per-skill customisation: a `PROJECT.md` beside a skill extends it and is
  never touched; `custom = ["<name>"]` under `[skills]` releases a skill
  from management entirely. The pack's version is the binary's, so
  `update skills@<version>` is refused like the other pinned capabilities
- `superdev aokf hook validate`: the hook as a subcommand — payload on
  stdin, validates in-process, works on every platform superdev ships for
- `init` adopts a repo's existing skills: one already sitting under a pack
  name, with content of its own, is released into `[skills] custom` and
  reported, instead of being overwritten and backed up
- Blueprint migrations: `sync` now removes what the blueprint no longer
  ships — dropped files, renamed paths' old copies, a disabled capability's
  pins and registrations. Unmodified leftovers are removed with a backup;
  user-edited ones are left in place, released from the lock, and reported
- `sync` ensures `CLAUDE.md` imports `AGENTS.md` (`@AGENTS.md`), so Claude
  Code actually loads the managed entry point
- `blueprint` in `.superdev/config.toml` now records the version last
  applied: `sync` stamps it, `status` reports a difference without failing
- Knowledge-carried skills: the knowledge capability ships the full
  aokf-converted skill set — 25 skills, each its whole directory, most
  derived from mattpocock/skills (MIT) — as owned repo files under
  `.claude/skills/`, released per skill by `[knowledge] custom`. The
  workflows capability is gone: a manifest still naming `[workflows]` fails
  with a guided migration error, and the next sync after the table is
  deleted swaps same-named skills to knowledge ownership and sweeps the
  dropped upstream files. The superpowers plugin remains installable by
  hand: `claude plugin install superpowers`
- `superdev template list` and `superdev template render`: read-only views
  of the shipped project templates — render writes the token-substituted
  tree into an empty directory and prints the derived token values.
  `[template]` in `.superdev/config.toml` now records the seeding binary's
  `version`; older manifests parse unchanged
- The skill pack's `template-update`: update a repo from its project
  template, or adopt one into a repo that never used a template — the
  template confirmed with the user, a summary and per-area questions
  before anything is written, and `[template]` restamped afterwards. The
  engine still never touches template files; every update is a user edit

### Fixed

- `sync` no longer installs the repo's whole toolchain. `mise install` and
  `mise exec` now name superdev's own pinned tools, so an unrelated pin that
  cannot build on this machine no longer fails the entire apply — found
  adopting superdev in a repo pinning `cargo:cargo-ndk`

### Changed

- The AOKF validator is now the binary's own `aokf validate`; the bundled
  Python `validator.py` is deleted, and the validation hook, `check:aokf`
  and CI all call the Rust one. Findings, JSON and exit codes are unchanged
- AGENTS.md no longer preloads every concept in the knowledgebase. It keeps
  `knowledge/index.md` as the map and tells agents to search the MCP server
  for the rest

### Removed

- The skill pack's `grill-me` — the default workflows provider ships its
  own; the next sync sweeps the packaged copy (a user-edited copy is left
  in place and released). A `[skills] custom` name that is no longer in
  the pack now reports instead of failing the plan
