# superdev

superdev sets a repository up for agent-driven development and keeps that
setup current. Run it inside the repo you want managed:

```sh
superdev init      # install the tooling and record what was installed
superdev status    # report drift; exits 1 when there is work to do
superdev sync      # re-apply the blueprint (--dry-run to preview)
superdev update    # move version pins to this binary's defaults, then sync
```

`init` writes an AOKF knowledgebase, builds a code index, and installs the
Claude Code plugins superdev expects, then records the result in
`.superdev/`. Pass `--no-knowledge`, `--no-code-index`, `--no-workflows` or
`--no-frontend` to leave a capability out. Everything it owns can be repaired
by re-running `sync`.

superdev is opinionated for one stack — Claude Code, mise and AOKF — and is
still under construction. Nothing is published yet.

## Install

Not published yet. Once released:

```sh
npm install -g superdev   # prebuilt binaries for Linux, macOS, Windows
cargo install superdev    # build from source
```

## Development

See [CONTRIBUTING](CONTRIBUTING.md) for setup, everyday commands, and the
release procedure. Project design and conventions live in the
[`knowledge/`](knowledge/index.md) bundle.

## License

MIT — see [LICENSE](LICENSE).
