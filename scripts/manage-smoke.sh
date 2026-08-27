#!/usr/bin/env bash
# Manual smoke: real `superdev init` with real mise/claude/codegraph, then the
# knowledge verbs against the canonical knowledge it just wrote.
# Run inside the devcontainer before a release. Not wired into CI.
set -euo pipefail
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
cargo build --release
git init -q "$scratch/repo"
cd "$scratch/repo"
"${OLDPWD}/target/release/superdev" init
"${OLDPWD}/target/release/superdev" status
"${OLDPWD}/target/release/superdev" validate
# The only run that exercises the real embedding model: ~130 MB downloaded
# once into the user cache, then loaded from there. Every other test stubs it.
"${OLDPWD}/target/release/superdev" sokf index knowledge
echo "manage smoke OK"
