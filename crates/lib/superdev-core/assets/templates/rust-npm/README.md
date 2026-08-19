# {{superdev:project-name}}

A Rust CLI, distributed as prebuilt binaries through npm.

## Install

```sh
npm install -g {{superdev:project-slug}}
```

The npm package is a thin launcher: it resolves the prebuilt binary for the
host platform (an optional dependency) and runs it.

## Develop

Open the repo in the dev container (`.devcontainer/`) and everything is
already there — the pinned Rust toolchain, Node, and the superdev agent
tooling. Without it, [mise](https://mise.jdx.dev) installs the same versions
from `mise.toml`:

```sh
mise install
```

The layout, the command set, and the release procedure are in
[CONTRIBUTING](CONTRIBUTING.md).

## License

Proprietary — see [LICENSE](LICENSE).
