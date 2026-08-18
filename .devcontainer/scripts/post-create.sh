#!/usr/bin/env bash
set -euo pipefail

export MISE_YES=1
export MISE_VERBOSE=1

# Fix ownership on volume-backed dirs (volumes mount as root on first creation).
sudo chown -R vscode:vscode /home/vscode 2>/dev/null || true
sudo chown -R vscode:vscode ${CONTAINER_WORKSPACE_FOLDER} 2>/dev/null || true


mise install
mise exec -- npm install

# The Claude Code validation hook this repo installs calls a bare `superdev`.
# Link the dev shim so it resolves to the working tree; without it the hook
# fails with command-not-found and validation goes silently unenforced.
mkdir -p "$HOME/.local/bin"
ln -sf "${CONTAINER_WORKSPACE_FOLDER}/scripts/superdev" "$HOME/.local/bin/superdev"

# Submodules are consumed read-only: init them, then break each push URL so an
# accidental `git push` from inside one fails loudly. (The push URL is local
# config, so it must be reapplied per clone.)
git submodule update --init
git submodule foreach 'git remote set-url --push origin no_push'

