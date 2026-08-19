#!/bin/sh
set -e

# Shared build and runtime dependencies for the dev container.
# adb/fastboot talk to Android devices; bubblewrap and socat back agent
# sandboxing; the rest is general plumbing (downloads, archives, JSON).
apt-get update && apt-get install -y --no-install-recommends \
    jq \
    bubblewrap \
    socat \
    curl \
    unzip \
    git \
    ca-certificates \
    adb \
    fastboot \
 && apt-get clean -y \
 && rm -rf /var/lib/apt/lists/*
