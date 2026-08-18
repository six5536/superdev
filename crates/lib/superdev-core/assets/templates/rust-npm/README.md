# {{superdev:project-name}}

A Rust CLI, distributed as prebuilt binaries through npm.

## Install

```sh
npm install -g {{superdev:project-slug}}
```

The npm package is a thin launcher: it resolves the prebuilt binary for the
host platform (an optional dependency) and runs it.

## Develop

Rust toolchain pinned by `rust-toolchain.toml`; Node 18+ for the launcher.

```sh
npm run build     # cargo build --workspace
npm run test      # cargo nextest run --workspace
npm run lint      # cargo clippy --workspace
```

See [CONTRIBUTING](CONTRIBUTING.md) for the layout and the release procedure.

## License

Proprietary — see [LICENSE](LICENSE).
