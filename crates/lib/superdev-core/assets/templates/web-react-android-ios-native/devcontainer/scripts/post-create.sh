#!/usr/bin/env bash
set -euo pipefail

# Fix ownership on volume-backed dirs (volumes mount as root on first creation).
sudo chown -R vscode:vscode /home/vscode 2>/dev/null || true
sudo chown -R vscode:vscode "${CONTAINER_WORKSPACE_FOLDER}" 2>/dev/null || true

# The full toolchain (+ Android SDK packages) is baked into the image as vscode, so this
# is a fast idempotent no-op / drift safety net, not the primary install. It also picks
# up whatever superdev wrote into .mise.toml — mise merges every config in the directory.
# See Dockerfile.
mise install

# Tools that belong to the container rather than the project, so they go in the
# container's global mise config and stay out of the repo's files. superdev
# itself has to be here: the knowledge validation hook it installs calls a bare
# `superdev`, and without it the hook fails with command-not-found and
# validation goes silently unenforced.
mise use --global "npm:superdev@latest" "npm:@anthropic-ai/claude-code@latest"

mise exec -- npm install

# Initialize the Playwright MCP workspace (skills + config) in the mounted repo. The
# browser binaries are baked into the image (see Dockerfile / playwright-browsers); this
# only writes .claude/skills/playwright-cli + .playwright into the workspace, which doesn't
# exist at build time — so it must run here. Cheap + idempotent.
mise run playwright-workspace
