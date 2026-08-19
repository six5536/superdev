FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_SDK_ROOT=/opt/android-sdk

# Install shared build and runtime dependencies
COPY .devcontainer/scripts/install-deps.sh /tmp/install-deps.sh
RUN /tmp/install-deps.sh && rm /tmp/install-deps.sh

# On arm64 hosts, enable amd64 multiarch so the Android SDK's x86_64 tools
# (aapt2, etc.) can run under Rosetta. No-op on amd64.
COPY .devcontainer/scripts/setup-amd64-multiarch.sh /tmp/setup-amd64-multiarch.sh
RUN chmod +x /tmp/setup-amd64-multiarch.sh && /tmp/setup-amd64-multiarch.sh && rm /tmp/setup-amd64-multiarch.sh

# Import Node.js release signing keys for GPG verification
# https://github.com/nodejs/node#release-keys
# Required for mise to install Node.js versions
# gpg keys are used to verify the integrity of downloaded Node.js binaries
USER vscode
RUN gpg --keyserver hkps://keys.openpgp.org --recv-keys \
  4ED778F539E3634C779C87C6D7062848A1AB005C \
  141F07595B7B3FFE74309A904F0EEBB37E0E2C14 \
  74F12602B6F1C4E913FAA37AD3A89613643B6201 \
  DD792F573C6DE52C432CBDAC77ABFA00DDBF2B7C \
  CC68F5A3106FF448322E48ED27F5E38D5B0A215F \
  8FCCA13FEF1D0C2E91008E09770F7A9A5AE15600 \
  C4F0DFFF4E8C1A8236409D08E73BC641CC11F4C8 \
  890C08DB8579162FEE0DF9DB8BEAB4DFCF555EF4 \
  C82FA3AE1CBEDC6BE46B9360C43CEC45C17AB93C \
  108F52B48DB57BB0CC439B2997B01419BD92F80A \
  A363A499291CBBC940AE88276104A21DB7A5B203
USER root

# Install mise from a pinned GitHub release, verified against committed SHA256s.
# To bump: change MISE_VERSION, then update both SHA256 values from
#   https://github.com/jdx/mise/releases/download/${MISE_VERSION}/SHASUMS256.txt
ARG MISE_VERSION=v2026.7.11
ARG MISE_SHA256_X64=6652ee5dd3bfa804a29355f7bb936134892bb9ac002b4cda172a44c67597cd0c
ARG MISE_SHA256_ARM64=fca1ffa5fb7fc848f6af0be7aa45260b5a8507fc4f97b6b974a9d60ed9647817
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


# Android command-line tools.
# NOTE: Google's filename embeds a build number that changes. Get the current
# linux .zip URL from https://developer.android.com/studio#command-line-tools-only
ARG CMDLINE_TOOLS_ZIP=commandlinetools-linux-14742923_latest.zip
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools \
 && curl -fsSL "https://dl.google.com/android/repository/${CMDLINE_TOOLS_ZIP}" -o /tmp/cmdtools.zip \
 && unzip -q /tmp/cmdtools.zip -d ${ANDROID_HOME}/cmdline-tools \
 && mv ${ANDROID_HOME}/cmdline-tools/cmdline-tools ${ANDROID_HOME}/cmdline-tools/latest \
 && rm /tmp/cmdtools.zip \
 && chown -R vscode:vscode ${ANDROID_HOME}   # vscode runs sdkmanager in the next (vscode) step


# Bake the full mise toolchain + Android SDK packages into the image AS vscode, so
# the tools land in /home/vscode/.local/share/mise and persist into the image (that
# path is NOT a volume). post-create's `mise install` then becomes a fast idempotent
# no-op / drift safety net instead of reinstalling the whole toolchain.
#
# The seed mise.toml is a throwaway copy: the real workspace isn't mounted at build
# time, so we copy just the manifest to drive the install. The real workspace config takes
# over at runtime.
#
# Android SDK packages come from `mise run android-packages` —
# the single source of truth ([tasks.android-packages] in mise.toml). No longer
# duplicated inline here.
#
# playwright-browsers downloads Chromium into ~/.cache/ms-playwright (persists in the image)
# + its apt deps. Only the browser binaries are bakeable; the workspace-bound skills/config
# (playwright-workspace) run at post-create against the mounted repo. NOTE: baking the
# browsers means a Playwright version bump in mise.toml needs an image rebuild to refresh
# them (post-create's mise install won't redownload).
USER vscode
COPY --chown=vscode:vscode mise.toml /tmp/seed/mise.toml
WORKDIR /tmp/seed
RUN mise trust && mise install && mise run android-packages && mise run playwright-browsers

# Keep the SDK's x86_64 adb/fastboot (run via Rosetta) rather than swapping in
# the Debian arm64 build. Reason: the container's adb must MATCH the macOS host's
# adb version. Gradle's ddmlib kills any host adb server whose revision differs
# from the container's adb, and the container can't restart a server that lives
# on the host -- so a mismatch (arm64 Debian 34.x vs the Mac's latest) makes
# `./gradlew installDebug` kill the host server and hang. The SDK ships the same
# "latest" platform-tools the Mac has, so they match.
#
# These x86_64 binaries run under Rosetta because the amd64 glibc loader they
# need (/lib64/ld-linux-x86-64.so.2) is already installed by
# setup-amd64-multiarch.sh -- which is why the old "failed to open elf" error,
# the original reason for the arm64 swap, no longer occurs. Nothing to do here:
# sdkmanager already installed the x86_64 binaries above.
