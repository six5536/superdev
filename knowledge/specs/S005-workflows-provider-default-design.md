---
type: Spec
id: spec-workflows-provider-default
title: Workflows Provider Default
description: Design for defaulting the workflows capability to mattpocock-skills as materialised repo files, with superpowers kept as a supported plugin-based secondary.
status: deprecated
links:
  - rel: relates-to
    to: spec-cli-core-blueprint-engine
    note: Makes the registry's capability-to-provider map real; providers become selectable.
  - rel: relates-to
    to: spec-blueprint-migrations
    note: Provider switches and upstream skill drops ride the orphan pass unchanged.
  - rel: relates-to
    to: spec-skill-pack
    note: Reuses the custom-release mechanism; drops the pack's grill-me skill.
---

# Goal

Make [mattpocock/skills](https://github.com/mattpocock/skills) the default
workflows provider and relegate Superpowers to a supported secondary. The
default must not be tied to the user: a collaborator who clones a managed
repo gets working skills from git alone, with no superdev, mise, or Claude
Code plugin install on their machine.

This also makes provider selection real. Today `components::enabled` maps
each capability to one hardcoded component and ignores the manifest's
`provider` field entirely — the [blueprint
engine](S001-cli-core-blueprint-engine-design.md)'s
capability-to-provider map exists only in prose.

# Registry-backed provider selection

The registry becomes per-(capability, provider): each entry carries its own
version, availability, and install data (URL and checksum where
binary-pinned), with exactly one entry per capability flagged as the
default. `Manifest::default_for` picks the defaults, so a fresh `init`
writes `[workflows] provider = "mattpocock-skills"` with the registry
version.

`components::enabled` becomes fallible. For each enabled capability it
resolves the manifest's `provider` string to a component; an unknown
provider fails with a message naming the valid ids
(`workflows provider must be one of: mattpocock-skills, superpowers`).
`behind_pins` and `checksum_pin_mismatch` look the registry up by
(capability, manifest provider) instead of capability alone. Both workflow
providers are binary-pinned: the binary carries each provider's checksum,
so only its registry version installs.

## CLI

- `superdev init --workflows-provider <id>` — validated against the
  registry; absent, the default (`mattpocock-skills`) applies. The
  `--no-workflows` flag is unchanged.
- `superdev update workflows --provider <id>` — the one CLI path that
  switches provider: rewrites the manifest's provider, sets its version to
  that provider's registry default, then syncs. `update workflows` without
  the flag moves the version for the current provider, as today. Bare
  `superdev update` never changes a provider.
- Hand-editing `config.toml` and running `sync` stays equivalent.

`sync` never involves a provider choice; it applies what the manifest says.

# The mattpocock-skills provider

Delivered as materialised repo files, not a plugin. Claude Code's
project-scope plugin settings (`extraKnownMarketplaces`, `enabledPlugins`)
were considered and rejected: as of Claude Code 2.1.195 a repo-declared
external plugin still requires a per-user `claude plugin install`, and
per-plugin version pinning does not exist. The upstream plugin declares
only skills — no hooks, commands, or MCP servers — so flattening it to
files loses nothing.

- **Fetch.** `.mise.toml` pins `http:mattpocock-skills` to the release
  tarball
  (`https://github.com/mattpocock/skills/archive/refs/tags/v1.2.3.tar.gz`,
  `strip_components = 1`) with a sha256 checksum baked into the binary
  beside the version, computed at implementation time. mise is the
  checksummed downloader into machine cache, exactly as for codegraph.
  `MANAGED_MISE_TOOLS` gains the tool.
- **Materialisation.** A new engine action copies the pinned checkout's
  skill directories into `.claude/skills/<name>/`, flattened from the
  upstream `skills/engineering/` and `skills/productivity/` grouping. Every
  file write carries backup, journal, and lock hashing like any owned
  write, so a mid-materialisation failure unwinds cleanly. The files are
  committed: collaborators need nothing installed.
- **Local drift detection.** `status` compares disk against lock hashes and
  the lock's workflows version against the manifest — no network, no
  checkout. Only a refresh (`sync` after a version bump or drift) reads the
  checkout; a missing or unreadable checkout at that point fails loudly.
- **Lock attribution.** The lock records which capability owns each
  materialised file — a small extension, back-compatible: entries without
  attribution behave exactly as before. `owned()` claims the pin plus the
  attributed files, without hardcoding the upstream skill list. Upgrades
  are then self-cleaning: a skill the next release drops becomes an
  unclaimed lock entry and the
  [orphan pass](S004-blueprint-migrations-design.md) sweeps it, and a
  provider switch sweeps the whole set.
- **User edits.** Same rules as the skill pack: owned files are overwritten
  with backup and a note. `[workflows]` gains the same `custom = [...]`
  release list, and `init` adoption seeds it for pre-existing skills under
  upstream names.
- **Setup hint.** The upstream `/setup-matt-pocock-skills` skill is
  interactive (issue tracker, domain-doc layout) and cannot run unattended.
  After a materialisation, `sync` prints one report line pointing at it.
  Reports never move the exit code.

Upstream is MIT-licensed; attribution is recorded where the provider is
documented.

# Superpowers, the secondary

Unchanged machinery: mise-pinned checkout, local marketplace registration,
`claude plugin install` as optional actions, per-machine. Its hooks and
commands need the plugin runtime, so it cannot be flattened to files; the
per-user install step is structural and is part of why it is the
secondary. Repos with `provider = "superpowers"` keep working exactly as
today — the upgrade changes nothing for them beyond the skill-pack sweep
below.

Switching a repo, either direction, is one sync: the old provider's pin
loses its claim and is swept; the new provider's pin, files or plugin
actions, and override file apply. Reports cover what superdev cannot do:
the leftover user-level plugin (`claude plugin uninstall superpowers`), the
AGENTS.md import to update, and the setup hint. A failed apply rolls the
whole switch back — removals are planned last.

# Provider-matched knowledge override

The knowledge component reads the workflows provider from the manifest and
ships the matching override file:

- superpowers → `.agents/SUPERPOWERS.md`, byte-identical to today.
- mattpocock-skills → `.agents/MATT-POCOCK-SKILLS.md`, new owned asset,
  same shape and job: `to-spec` output goes to
  `knowledge/specs/YYYY-MM-DD-<topic>-design.md` as AOKF concepts (type
  `Spec`, unique id, `draft` until implemented); `wayfinder` and
  `to-tickets` plans go to `knowledge/plans/`, ephemeral, deleted when the
  work lands; decisions and ADRs live as AOKF `Decision` concepts, not
  `docs/adr/`; nothing the skills write may duplicate ground the canonical knowledge
  covers.
- workflows disabled → no override file. Today `SUPERPOWERS.md` ships
  unconditionally; that becomes provider-gated.

`AGENTS.md` is a user-owned scaffold written once, and it imports the
override by name. The scaffold template becomes provider-aware at init (a
`{workflows_overrides}` substitution beside the existing `{name}`), and
filenames stay per-provider so an existing repo never has its file renamed
under a scaffold that still imports the old name. On a provider switch the
old override is swept, the new one written, and a report line tells the
user to update the import — superdev does not edit user-owned files.

# Skill pack: grill-me leaves

The pack goes five skills to four (`aokf-maintain`, `double-check`,
`humanise`, `self-improve`): the default provider ships its own `grill-me`,
and two variants in one picker is noise. No removal code — the next sync
finds the unclaimed lock entry and the orphan pass sweeps it, or releases
it with a report where the user had edited it.

The [skill pack](S003-skill-pack-design.md)'s custom-list validation
softens with it: a `[skills] custom` name that is not in the pack becomes
the report line
`skills: custom names unknown skill '<name>' — no effect` instead of a
plan failure. A repo that had marked `grill-me` custom upgrades cleanly.
Typo protection drops from a hard stop to a visible status line — a typo
also produces the real skill's drift plan, so it stays discoverable.

# Errors and exit codes

- Unknown provider (manifest or `--provider`): error naming the valid ids,
  exit 2.
- Materialisation failures (missing tool, unreadable checkout, unwritable
  target): fail the run loudly; per-file journal entries unwind what was
  written.
- Planned materialisation and sweeps are planned actions: `status` exits 1
  while any remain. The setup, uninstall, unknown-custom and import-update
  lines are reports and never move the exit code.

# Dogfooding: this repo switches

This repo currently manages only the skills capability, and its own
development has run on Superpowers. Once the feature lands, the closing
act of the sub-project switches it:

- Enable workflows here: add
  `[workflows] provider = "mattpocock-skills"` to `.superdev/config.toml`,
  sync with the new binary, and commit the materialised skills and lock.
  The sync writes the `http:mattpocock-skills` pin into `.mise.toml`,
  which carries uncommitted local edits — committing that pin needs
  coordination with the repo owner at that point.
- This repo's `.agents/` and knowledge files are hand-maintained, not
  superdev-owned, so the same change hand-applies what a managed repo
  would get: `.agents/SUPERPOWERS.md` is replaced by
  `.agents/MATT-POCOCK-SKILLS.md` mirroring the shipped asset, the
  `AGENTS.md` import updated, and the process docs
  (`development-procedure.md` and the affected parts of
  `development-commands.md`) rewritten from the
  brainstorming/writing-plans/subagent-driven flow to the
  `to-spec`/`wayfinder`/`implement` flow.
- From the next sub-project on, superdev's own development runs on the
  default provider. The Superpowers plugin stays installed at user level
  on the development machine until its owner removes it; the sub-project
  in flight finishes under the process it started with.

# Testing

- Registry: two workflow entries, one default; version and checksum per
  provider.
- `enabled()`: resolves each provider string; unknown provider errors with
  the valid ids.
- CLI: `init --workflows-provider` validation and default;
  `update workflows --provider` rewrites provider and version; bare
  `update` leaves providers alone.
- Engine: the materialisation action's per-file backup, journal, lock
  attribution, and unwind.
- `owned()`: the new component claims pin plus attributed files; the aokf
  component's override claim follows the provider and disappears with
  workflows disabled.
- Lock: an unattributed 0.1.0-era lock reads correctly.
- Skill pack: four skills; an unknown custom name reports instead of
  failing.
- End to end: fresh init materialises the default provider's skills into
  disk and lock; `init --workflows-provider superpowers` reproduces
  today's behaviour; a provider switch sweeps and re-materialises in both
  directions with the reports above; `grill-me` is swept on upgrade; a
  repo with a stale custom name syncs.

# Out of scope

- No generic `--provider` surface for other capabilities; nothing else has
  a second provider.
- The frontend, code-index, knowledge and skills capabilities keep their
  single providers.
- Structured AOKF update over MCP and knowledge upkeep remain the later
  sub-projects.
