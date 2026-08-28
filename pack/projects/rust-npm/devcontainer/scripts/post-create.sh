#!/usr/bin/env bash
set -euo pipefail

export MISE_YES=1

# Fix ownership on volume-backed dirs (volumes mount as root on first creation).
sudo chown -R vscode:vscode /home/vscode 2>/dev/null || true
sudo chown -R vscode:vscode "${CONTAINER_WORKSPACE_FOLDER}" 2>/dev/null || true

# Every pin the repo carries: the project's own tools from mise.toml (Rust,
# Node, the cargo tooling) and whatever superdev wrote into .mise.toml
# (codegraph). mise merges all of them.
mise install

# Tools that belong to the container rather than the project, so they go in the
# container's global mise config and stay out of the repo's files. superdev
# itself has to be here: the knowledge validation hook it installs calls a bare
# `superdev`, and without it the hook fails with command-not-found and
# validation goes silently unenforced.
mise use --global "npm:superdev@latest" "npm:@anthropic-ai/claude-code@latest"

mise exec -- npm install
