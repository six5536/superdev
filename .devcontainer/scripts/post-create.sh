#!/usr/bin/env bash
set -euo pipefail

export MISE_YES=1
export MISE_VERBOSE=1

# Fix ownership on volume-backed dirs (volumes mount as root on first creation).
sudo chown -R vscode:vscode /home/vscode 2>/dev/null || true
sudo chown -R vscode:vscode ${CONTAINER_WORKSPACE_FOLDER} 2>/dev/null || true


mise install
mise exec -- npm install

# Submodules are consumed read-only: init them, then break each push URL so an
# accidental `git push` from inside one fails loudly. (The push URL is local
# config, so it must be reapplied per clone.)
git submodule update --init
git submodule foreach 'git remote set-url --push origin no_push'

# Superpowers (Claude Code plugin). `mise install` above fetched the
# checksum-verified checkout pinned in .mise.toml; register it as a local
# marketplace and install the plugin from it. Remove-then-add so a version
# bump in .mise.toml lands on the next container create; both install steps
# are no-ops when already current.
superpowers="$(mise where http:superpowers)"
mise exec -- claude plugin marketplace remove superpowers-dev >/dev/null 2>&1 || true
mise exec -- claude plugin marketplace add "$superpowers"
mise exec -- claude plugin install superpowers@superpowers-dev

