#!/usr/bin/env bash
# Sourced by mise (`_.source` in mise.toml) in every shell INSIDE the container.
# Decides where the container's `adb` CLI should look for its server.
#
# Why this is a runtime probe and not a static value: the answer depends on the HOST OS, and nothing
# inside the container can see that directly — mise's `{{ os() }}` evaluates to `linux` here no matter
# whether the host is macOS, Windows or Linux, so it cannot be used to branch.
#
# Why it lives in mise rather than devcontainer.json `containerEnv`: mise can conditionally OMIT a
# variable, which containerEnv cannot. That matters because adb rejects an empty value
# (`no host in ':5037'`) — the var has to be absent, not blank. Note the corollary: mise can only
# add or change vars, never unset one that containerEnv already set, which is exactly why these two
# no longer live there.
#
# The discriminator is whether the container has direct USB access:
#
#   /dev/bus/usb present  → the device is ours (Linux host + the .devcontainer/linux-usb config).
#                           Leave the vars unset so adb runs its own local server.
#   /dev/bus/usb absent   → macOS/Windows, where Docker is in a VM and adb must live on the host.
#                           Point at the host bridge started by scripts/host/android-debug.sh.
#
# A filesystem test, deliberately — no DNS lookup, because this runs on every shell spawn and a probe
# of `host.docker.internal` would put a resolver round-trip (and its timeout) in that path.
#
# Overriding: the values below defer to anything already exported, and `.mise.local.toml` layers over
# mise.toml. But note that .mise.local.toml sits in the MOUNTED WORKSPACE — the host and the container
# read the same file, so it is not a per-machine escape hatch (see docs/ANDROID_DEBUGGING.md for the
# ANDROID_HOME case where that already bites). For a one-off, e.g. wireless adb on a macOS host wanting
# a local server despite there being no /dev/bus/usb:
#   env -u ANDROID_ADB_SERVER_ADDRESS -u ANDROID_ADB_SERVER_PORT adb devices
#
# See docs/ANDROID_DEBUGGING.md.

if [ ! -d /dev/bus/usb ]; then
  # Docker Desktop proxies host.docker.internal to the host's loopback, where android-debug.sh binds.
  export ANDROID_ADB_SERVER_ADDRESS="${ANDROID_ADB_SERVER_ADDRESS:-host.docker.internal}"
  export ANDROID_ADB_SERVER_PORT="${ANDROID_ADB_SERVER_PORT:-5037}"
fi
