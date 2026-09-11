# superdev

superdev sets a repository up for agent-driven development and keeps that
setup current.

`init` writes canonical project knowledge, installs the Pi workflow extension
and SOKF authoring skill, and can build a code index or install optional
capabilities before recording the result in `.superdev/`. Pass
`--no-code-index`, `--no-skills` or `--no-frontend` to leave an optional
capability out. The SOKF knowledge has no such
flag: it is part of superdev, not a capability something else could fill.
Everything it owns can be repaired by re-running `sync`.
## Install

```sh
npm install -g @six5536/superdev   # prebuilt binaries for Linux, macOS, Windows
cargo install superdev             # build from source
```

Either way the command is `superdev`. The npm package pulls a prebuilt
binary for your platform; the crate builds one.

## Quick start

Run it inside the repo you want managed:

```sh
superdev init      # install the tooling and record what was installed
superdev status    # report drift; exits 1 when there is work to do
superdev sync      # re-apply the blueprint (--dry-run to preview)
superdev update    # bring pins current, then sync
```

`init` refuses an already initialized repository; everything superdev owns can
be repaired by `sync`. A file you have edited is never overwritten in silence: `status`
reports it, and `sync` backs it up before writing.

## Usage

It also registers an MCP server for the canonical knowledge, so agents
search it instead of preloading every page of it:

```sh
superdev mcp sokf        # serve the SOKF knowledge over MCP (stdio)
superdev validate        # check it against the SOKF spec and the schemas
superdev sokf index      # rebuild the search index from scratch
```

The server offers four read-only tools — search, resolve-source, retrieve and
graph — plus two agent-safe mutations. Locating a file and rendering knowledge
are separate questions: `sokf_resolve_source` says where a concept's source is
and which of its lines are generated, while `sokf_retrieve` renders the
overview, a concept, or one section. In a Pi session the ordinary `read` tool
accepts `sokf:<id>` and returns that file's exact bytes, so an excerpt you copy
still matches the file you copied it from. The server keeps itself current:
every indexed call re-hashes the canonical knowledge and reindexes only what
changed. Search is hybrid, combining a BM25 index with a
small embedding model downloaded once per machine, and falls back to
keyword-only if that model is unavailable.

### The workflow

Pi's project extension orchestrates exactly `SCOPE → BUILD → ACCEPT`; durable
state remains in one canonical issue and plan while Rust owns legal transitions,
transient session ownership, and Git safety.

```
/superdev <request>  # run or continue the complete workflow
/scope               # requirements, isolated review, explicit approval
/build               # isolated block loop, evidence, verification, fresh review
/accept              # configured acceptance; leave the branch for a human merge
/superdev-resume     # reconstruct state from the canonical plan
/superdev-cancel     # pause and release transient ownership
/superdev-abandon    # explicit human-only abandonment
/superdev-force      # human-only override of a SCOPE or ACCEPT gate the workflow refuses
/skill:file <item>   # LLM-authored issue or idea on the default branch; worktree when elsewhere
```

Every plan implements exactly one issue and uses the matching
`work/<issue-number>-<slug>` branch. ACCEPT never pushes, releases, deletes the
branch, or runs remote CI. Human acceptance is controlled only by
`.superdev/config.toml`.

### Where the content comes from

The Pi assets, templates, schemas, and scaffolds superdev writes are a *content pack*. One
ships inside the binary, and `.superdev/config.toml` records which pack a repo
uses:

```toml
[[packs]]
source = "github:six5536/superdev"   # a git repo — a rev is required
rev    = "assets-v0.1.0"

[[packs]]
source = "./packs/acme"              # or a directory on this machine
```

Entries layer in the order written and a later item of the same name wins, so
you can add or supersede content without forking. An
entry naming the source superdev's own content comes from *replaces* it rather
than layering, so what that revision drops leaves your repo too. A repo that
names no pack behaves exactly as it always did.

`source` takes `github:owner/repo` and `gitlab:owner/repo` as shorthand for
those two forges, and otherwise a git URL over `https://`, `ssh://` or
`file://` — the scp form `git@host:path` included, so your ssh config and your
mirrors keep working. Those are the whole set: `git://` and `http://` are
refused because neither authenticates, so anyone on the path could answer for
the pack, and an `ext::`-style remote helper is refused because it names a
program rather than a transport. Anything with no scheme and no `host:` is a
directory, read from disk every run and taking no `rev`; a bare repository on
disk is a git URL, so spell it `file:///srv/mirror.git`. Fetching uses your
own `git`, so credentials stay yours and superdev stores no token. What
it fetches is verified against a digest in `.superdev/lock.toml`, and a
revision that resolves to different bytes than the lock recorded stops the run
rather than quietly applying them.

A pack is the files it contains, so it may not contain a symlink — anywhere,
including its own root and its `pack.toml` — nor a submodule. One stops the
run naming the path. Deduplicating an item with a link would have the pack
ship a file it does not carry, and a link does not survive a checkout the same
way on every platform, so the same revision would otherwise verify on one
machine and fail on another; a submodule is left empty by the shallow clone
superdev makes, so the pack would ship an item with nothing in it. Copy the
file, or vendor the directory, instead.

Content releases separately from the binary, under its own `assets-vX.Y.Z`
tags, so a skill fix does not wait for one. `superdev update` is what goes and
looks: it asks the default source for its newest release and moves the pin
there, even past what your binary embeds — as far as the newest release your
binary can actually read. It fetches the pack before writing the pin naming
it, so a release built for a later superdev leaves your pin where it is and
says why, rather than parking your repo on content nothing you have can open.
A pin naming any other source is
reported and left alone — pointing superdev at someone's pack is your
decision, and it stays yours to revisit. Unreachable, the pin moves no further
than your binary already carries and the run says so — including on a network
that neither answers nor refuses, which superdev gives a few seconds before
reporting the same thing and carrying on. It is the one request superdev makes
that you did not ask for, so it is the one on a clock. Nothing prompts for
credentials either: a pack you cannot read anonymously fails rather than
waiting for you to type.

superdev is opinionated for one stack — Pi, Rust, mise and SOKF — and is still
young: expect the surface to move before 1.0.

## Configuration

Everything superdev keeps in a repo lives under `.superdev/`.
`config.toml` is what the repo wants — committed, hand-editable, and
rewritten by `update`:

| Option | Default | Description |
|--------|---------|-------------|
| `blueprint` | the version that ran `init` | The superdev version this repo's setup was written by. |
| `[[packs]]` | the pack inside the binary | Where skills, templates and scaffolds come from. Layers in the order written. |
| `[workflow]` | safe local defaults | Human-acceptance policy and positive retry limits. |
| `[knowledge.embeddings]` | the local model | Search on an API instead. The key comes from the environment, never the file. |
| `[frontend]`, `[skills]`, `[code-index]` | all enabled | One table per capability; an absent table means off. |


```toml
blueprint = "0.2.0"                  # the superdev version that wrote this

[[packs]]                            # where content comes from (above)
source = "github:six5536/superdev"
rev    = "assets-v0.1.0"

[knowledge]                          # the SOKF knowledge's own settings
custom = ["maintain"]                # skills you have taken over
[knowledge.embeddings]               # optional: search on an API instead
provider = "openai"                  #   of the local model
model    = "text-embedding-3-small"

[code-index]                         # one table per enabled capability
provider = "codegraph"
version  = "1.5.0"

[workflow]
human_acceptance_required = true
max_stalled_block_attempts = 3
max_final_correction_cycles = 3
```

An absent capability table means that capability is off. `lock.toml`
beside it is what superdev actually applied, and `cache/` is machine
state, gitignored by `init`. The API key for an embeddings provider comes
from the environment and never from the file.

## Development

```sh
mise install     # install all pinned tools
npm install      # install the JS workspace
npm test         # run the test suite
```

See [CONTRIBUTING](CONTRIBUTING.md) for everyday commands and the
release procedure. Project design and conventions live in the
[`knowledge/`](knowledge/index.md) tree.

## License

MIT — see [LICENSE](LICENSE).
