FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

# git comes from a devcontainer feature (see devcontainer.json); everything
# else — Rust, Node, the cargo tooling, superdev's own codegraph pin —
# is a mise tool, installed by post-create.sh. So mise itself is the one thing
# the image has to bootstrap, from a pinned GitHub release verified against
# committed SHA256s.
#
# To bump: change MISE_VERSION, then update both SHA256 values from
#   https://github.com/jdx/mise/releases/download/${MISE_VERSION}/SHASUMS256.txt
ARG MISE_VERSION=v2026.8.8
ARG MISE_SHA256_X64=58edfbdba6d4255b6536a61daeaf3b21f7a059430c789e948c8494ba32d59e1f
ARG MISE_SHA256_ARM64=6e6e96d319fe274996db5aed691f5398552865e641dc4b6fb6b01d73f4853a17
RUN set -eux; \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
      amd64) mise_arch=x64;   mise_sha="${MISE_SHA256_X64}"   ;; \
      arm64) mise_arch=arm64; mise_sha="${MISE_SHA256_ARM64}" ;; \
      *) echo "unsupported arch: $arch" >&2; exit 1 ;; \
    esac; \
    tarball="mise-${MISE_VERSION}-linux-${mise_arch}.tar.gz"; \
    curl -fsSLo /tmp/mise.tar.gz \
      "https://github.com/jdx/mise/releases/download/${MISE_VERSION}/${tarball}"; \
    echo "${mise_sha}  /tmp/mise.tar.gz" | sha256sum -c -; \
    tar -xzf /tmp/mise.tar.gz -C /tmp; \
    install -m 0755 /tmp/mise/bin/mise /usr/local/bin/mise; \
    rm -rf /tmp/mise /tmp/mise.tar.gz; \
    echo 'eval "$(mise activate bash)"' >> /home/vscode/.bashrc; \
    echo 'eval "$(mise activate zsh)"'  >> /home/vscode/.zshrc
