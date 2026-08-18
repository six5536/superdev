# superdev

superdev sets a repository up for agent-driven development — a
knowledgebase, a code index, committed skills — and keeps that setup
current. Under construction; nothing is published yet.

## Install

```sh
npm install -g superdev
```

This package is a thin launcher. It declares a prebuilt binary for each
supported platform as an `optionalDependency`, and npm installs only the one
matching your machine; a small JS shim then runs it.

**Supported platforms:** Linux and macOS (`x64` and `arm64`), and Windows
(`x64`). On any other platform, or to build from source, use the Rust
toolchain instead:

```sh
cargo install superdev
```

## Documentation

See the GitHub repository: <https://github.com/six5536/superdev#readme>.

## License

MIT — see <https://github.com/six5536/superdev/blob/main/LICENSE>.
