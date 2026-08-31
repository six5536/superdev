---
type: CliContract
id: contract-002-cli-superdev
title: CLI Contract
description: The superdev command line — the manage verbs, the knowledge verbs, the run verbs, and what each one promises its callers.
lifecycle: active
resource: /crates/app/superdev/src/main.rs
---

# Commands

```
superdev                     print help, exit 0
superdev init                set this repo up; --no-frontend, --no-skills
                             and --no-code-index each disable a capability.
                             The SOKF knowledge is always written: it is
                             not a capability.
superdev status              report drift; exit 1 when there is work to do
superdev sync                re-apply the blueprint; --dry-run prints the plan only
superdev update [TARGET]     bring pins current, then sync;
                             TARGET is `<capability>[@<version>]`;
                             --provider <ID> switches TARGET's provider
superdev validate [PATH...]  check the SOKF knowledge, every document against
                             the schema its type names, and the files the
                             grammar governs; exit 1 on errors. A PATH
                             replaces both defaults. --fix repairs the
                             knowledge's links first, --json, --doc renders
                             the grammar as prose, --knowledge <DIR>,
                             --repo-root <DIR> for `/`-rooted paths
superdev template list       list the shipped project templates
superdev template render     write a template's token-substituted tree into
  --name <NAME> --dir <DIR>  an empty directory (created if absent)
  <TEMPLATE>
superdev sokf index [PATH]   rebuild the search index from scratch
superdev run begin           arm an unattended run: create the run state
  [--session <ID>]           exclusively; refuses when one exists
  [--next <TEXT>]
superdev run advance         record a step forward: rewrite next, reset the
  --next <TEXT>              watchdog, refresh the owner
superdev run end             end the run: remove the state; harmless when
                             none exists
superdev hook validate       the Claude Code PostToolUse hook: payload on
                             stdin, validate when the edit touched the canonical knowledge
                             or a tree the grammar governs
superdev hook run            the Claude Code Stop hook: payload on stdin,
                             continue an active run or let the turn end
superdev mcp sokf            serve the canonical knowledge to agents over MCP on stdio
superdev completions <SHELL> write a completion script to stdout
                             (bash | zsh | fish | powershell | elvish)
superdev man                 (hidden; roff to stdout, for packaging)
-V, --version                print `superdev x.y.z` and exit
```

# Behaviour

Every verb acts on the current directory.

- **`init`** refuses a directory that is not a git repo, and refuses a re-run
  once `.superdev/config.toml` exists (it points at `sync`). The guard is the
  manifest rather than the directory, because the knowledge verbs create
  `.superdev/cache/` in repos that were never initialised. It writes the
  manifest, then applies the whole blueprint and the `.gitignore` lines. It
  also ensures `CLAUDE.md` contains the line `@AGENTS.md`, appended to an
  existing file or created as a one-line file: Claude Code reads only
  `CLAUDE.md`, and that line is what makes it load the canonical entry point.
  AGENTS.md gets the same treatment with `@.agents/superdev.md` — the file
  is the user's; appending to a pre-existing one reports the hint that
  superdev's old sections can be trimmed.
  Skills the repo already has under a pack or knowledge-carried name are
  released into `[skills] custom` or `[knowledge] custom` first, so adoption
  never overwrites work superdev did not write.
  `--template <name>` seeds the repo from a shipped project
  template (an unknown name fails naming the shipped set), `--template none`
  declines, and `--name` sets the substitution values; on a TTY with neither
  flag, init prompts — template list first, then the project name prefilled
  with the directory name. Without a TTY there is no prompt and no template,
  so scripted init is unchanged. Template files are write-once scaffolds:
  existing files win and are reported as kept. A knowledge-enabled init ends
  with the hint to run `/bootstrap` in Claude Code — filling the canonical knowledge
  from existing docs and an owner interview is judgement work the agent does
  after the mechanical scaffolding.
- **`status`** never writes. It exits `1` on any drift, missing component,
  planned removal, or pin behind this binary's registry, so CI can gate on it.
  Each skill released by `[skills] custom` prints as
  `skills: <name> custom, unmanaged`, and each one released by
  `[knowledge] custom` the same way under its own capability name — a released
  skill is the user's file, not drift, so it leaves the code alone. Released orphans and the
  blueprint-version line print as reports and never affect the exit code.
  A `content:` line names where the content came from, because which entry
  superdev treated as layer 0 is inferred from the source and would otherwise
  be invisible ([ADR-004][sokf:adr-004-base-pack-identity]):
  `content: embedded pack <version>` when no entry replaced it,
  `content: base <source> at <rev>` when one did, `content: layer <source>`
  per pack above it, and `content: <source> not resolved` for a pin `status`
  could not satisfy — which it never fetches to satisfy. One pack hiding
  another's item prints
  `content: <winner> supersedes <loser>'s <item>`; hiding layer 0's is what a
  pack is for and prints nothing. All of these are reports and none affects
  the exit code — layering is what the manifest asked for, not drift.
- **`sync`** refuses to run while a registry-locked capability
  (`code-index`, `skills`) is pinned anywhere other
  than the registry default, and says to run
  `superdev update <capability>`. `code-index`
  is downloaded by URL and verified against a
  checksum baked into this binary beside the version, so no other version has
  provenance — or a URL; the skill pack's content and the SOKF knowledge's are
  embedded in the binary, so
  the binary is its provenance. On a fresh clone it runs `mise trust` then
  `mise install` before any provider command, because the committed pins need no
  edit yet name tools this machine has never installed — and mise will not
  install from a config this machine has never trusted. That install names
  superdev's own tools, so a repo pin superdev knows nothing about can never
  fail the run. Orphan removals run after every write, so a rename whose write
  fails rolls back before anything is deleted; an orphan the user has edited is
  released instead of removed. A successful run stamps this binary's version as
  the manifest's `blueprint`.
- **`update`** rejects an explicit `code-index@<version>` or
  `skills@<version>` for the same reason. Every other
  capability takes an explicit version. `--provider <id>` is the only CLI path
  that switches a provider: it needs a capability target, rewrites that
  capability's provider and sets its version to the new provider's registry
  default, then syncs. Bare `update` moves versions and leaves every provider
  alone — and it alone moves the pack pin, asking the default source for its
  newest release and taking that even when it is past what this binary embeds
  ([ADR-009][sokf:adr-009-update-queries-default-source]); a targeted
  `update <capability>` makes no such request. `update workflows` is an
  unknown-capability error: the capability was
  removed, and a manifest still carrying its table fails at load with the
  guided migration error.

`validate` with no `PATH` covers the SOKF knowledge at `--knowledge` (default
`knowledge/`) and every tree the grammar's `roots` names; `sokf index`
defaults `PATH` to `knowledge/`; the hook always reads the same whole set. The
search index lives in `.superdev/cache/sokf-index/`; `sokf index` and the
server use it, `validate` never opens it.

- **`validate`** runs both halves and reports once, with findings grouped by
  file, so a file both have something to say about is reported once and the
  two cannot reach different verdicts
  ([P006 D-17][sokf:plan-006-adhoc-rust-format-validator]). The SOKF half
  checks the knowledge against the specification; the schema half checks each
  document against the schema its frontmatter `type` names — sections present,
  in order, none prohibited, declared table columns, the line limit — and the
  skills and `.agents/core.md` against the grammar. A document whose `type`
  names no schema is reported, because a type that resolves to nothing reads
  as governed and is not.

  Documents with no frontmatter to dispatch on — `README.md`, `CHANGELOG.md` —
  are named by a schema's `target-files` glob instead. The glob is matched
  against the candidate list, never against the filesystem, which is what
  bounds it: nothing outside the SOKF knowledge and that named pair is ever a
  candidate.

  It prints findings as text, or as JSON under `--json`: `passed`, `concepts`,
  `files`, `findings`, and a `knowledge` key carrying the directory the run
  covered. Warnings alone exit `0`; any error exits `1`. A `PATH` replaces
  both defaults: only what it names is read, and the knowledge is validated
  only when a `PATH` is the knowledge or contains it. The grammar comes from
  `.agents/sokf/grammar.yaml`, or from the copy inside the binary when the
  repository has none.

  **`--fix`** is the one way `validate` writes. Before checking, it repairs
  what is mechanically repairable in the SOKF knowledge: a body or index link
  naming a concept by path becomes the id form of SPEC §8, and every
  document's `<!-- sokf:links -->` block is regenerated from the ids its body
  cites (§9). A link's id comes from the target document's own `id`, or from
  the filename stem where the path resolves to nothing and the stem names a
  concept — a link a rename left behind. Nothing else is touched: an image, a
  reference-style link, a path naming a file that is no concept, and every
  byte of prose and frontmatter are left as written.

  It writes only inside the resolved knowledge directory, covers it on the
  same condition the check does — so a `PATH` naming something else repairs
  nothing — and is idempotent: a second run writes nothing. The report then
  describes the repository as `--fix` left it, and names each file rewritten,
  under a `repaired` key in JSON. `hook validate` never passes it.
- **`sokf index`** forces a full rebuild. Nothing else needs it: the server
  syncs lazily on every tool call. It says so when no embedding model loaded
  and the index is lexical-only.
- **`run`** owns the state of an unattended workflow run,
  `.superdev/cache/run.toml` — the shared seam between the driver skill and
  the Stop hook, fixed in
  [contract-009-interface-run-state][sokf:contract-009-interface-run-state].
  `begin` creates it exclusively and refuses when one exists, naming the
  owning session and `run end` as the way to clear it. `advance --next`
  rewrites the next step, resets the watchdog counter and refreshes the
  owning session. `end` removes the state, and says so harmlessly when none
  exists. No `run` verb touches git, the network, or any file outside the
  cache.
- **`hook validate`** reads the PostToolUse payload from stdin and exits
  `0` unless the edited path is under the canonical knowledge or under a tree the grammar
  governs. Then it validates the whole set in-process and, on errors, prints
  them to stderr and exits `2` — which Claude Code hands back to the agent as a
  blocking error. It resolves the repo from `CLAUDE_PROJECT_DIR` when Claude
  Code sets it, else the working directory.
- **`hook run`** reads the Stop payload from stdin and the run state from the
  cache, and exits `0` when the state is absent, the payload's session is not
  the owner, the next step is empty, or the watchdog counter has reached its
  cap — otherwise it increments the counter and exits `2` naming the next
  step, which Claude Code feeds back as the instruction to keep going. An
  unreadable payload is a loud exit `2`, like `hook validate`; an unreadable
  run state is reported and exits `0`, because a Stop hook that fails closed
  holds every session in the repo open. It resolves the repo the same way
  `hook validate` does.
- **`mcp sokf`** starts the MCP server; its contract is
  [contract-003-mcp-sokf][sokf:contract-003-mcp-sokf].

A usage error (unknown flag or subcommand) exits `2` — the npm launcher's
smoke test relies on that code. `completions` and `man` render into a buffer
before writing, because `clap_complete` panics rather than returning an error
when a write fails. Exit codes are in [error-handling][sokf:error-handling]; the
manifest the verbs read is in [configuration][sokf:configuration].

# Stability

Unreleased. Every verb, flag and exit code above may change without notice.

<!-- sokf:links -->
[sokf:adr-004-base-pack-identity]: /knowledge/adrs/active/adr-004-base-pack-identity.md
[sokf:adr-009-update-queries-default-source]: /knowledge/adrs/active/adr-009-update-queries-default-source.md
[sokf:configuration]: /knowledge/configuration.md
[sokf:contract-003-mcp-sokf]: /knowledge/contracts/public/active/contract-003-mcp-sokf.md
[sokf:contract-009-interface-run-state]: /knowledge/contracts/internal/active/contract-009-interface-run-state.md
[sokf:error-handling]: /knowledge/error-handling.md
[sokf:plan-006-adhoc-rust-format-validator]: /knowledge/plans/done/plan-006-adhoc-rust-format-validator.md
