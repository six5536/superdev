#!/usr/bin/env bash
# Manual smoke: real `superdev init` with real mise/claude/codegraph.
# Run inside the devcontainer before a release. Not wired into CI.
set -euo pipefail
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
cargo build --release
git init -q "$scratch/repo"
cd "$scratch/repo"
"${OLDPWD}/target/release/superdev" init
"${OLDPWD}/target/release/superdev" status
echo "manage smoke OK"
