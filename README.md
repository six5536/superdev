# superdev

superdev sets a repository up for agent-driven development and keeps that
setup current. Run it inside the repo you want managed:

```sh
superdev init      # install the tooling and record what was installed
superdev status    # report drift; exits 1 when there is work to do
superdev sync      # re-apply the blueprint (--dry-run to preview)
superdev update    # bring pins current, then sync
```

`init` writes an AOKF knowledgebase with a full engineering skill set,
builds a code index, wires up a bash output filter that compacts command
output before it reaches agent context, and installs the Claude Code
plugin superdev expects, then records the result in `.superdev/`. Pass
`--no-knowledge`, `--no-code-index`, `--no-skills`,
`--no-bash-output-filter` or `--no-frontend` to leave a capability out.
Everything it owns can be repaired by re-running `sync`.

It also registers an MCP server for the knowledgebase, so agents search it
instead of preloading every page of it:

```sh
superdev mcp aokf        # serve the knowledgebase over MCP (stdio)
superdev aokf validate   # check it against the AOKF spec; exits 1 on errors
superdev aokf index      # rebuild the search index from scratch
```

The server offers four read-only tools — search, read, graph, overview — and
keeps itself current: every call re-hashes the bundle and reindexes only what
changed. Search is hybrid, combining a BM25 index with a small embedding model
downloaded once per machine, and falls back to keyword-only if that model is
unavailable.

## Where the content comes from

The skills, templates and scaffolds superdev writes are a *content pack*. One
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
you can add your own skills, or supersede superdev's, without forking. An
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

Content releases separately from the binary, under its own `assets-vX.Y.Z`
tags, so a skill fix does not wait for one. `superdev update` is what goes and
looks: it asks the default source for its newest release and moves the pin
there, even past what your binary embeds. A pin naming any other source is
reported and left alone — pointing superdev at someone's pack is your
decision, and it stays yours to revisit. Unreachable, the pin moves no further
than your binary already carries and the run says so; on a network that
neither answers nor refuses, expect it to wait for your OS to give up first.

superdev is opinionated for one stack — Claude Code, mise and AOKF — and is
still young: expect the surface to move before 1.0.

## Install

```sh
npm install -g @six5536/superdev   # prebuilt binaries for Linux, macOS, Windows
cargo install superdev             # build from source
```

Either way the command is `superdev`. The npm package pulls a prebuilt
binary for your platform; the crate builds one.

## Development

See [CONTRIBUTING](CONTRIBUTING.md) for setup, everyday commands, and the
release procedure. Project design and conventions live in the
[`knowledge/`](knowledge/index.md) bundle.

## License

MIT — see [LICENSE](LICENSE).
