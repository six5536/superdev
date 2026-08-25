#!/usr/bin/env bash
# RUN THIS ON THE DOCKER HOST — not inside the devcontainer.
# Supported hosts: macOS, Windows (Git Bash), Linux. See docs/ANDROID_DEBUGGING.md.
#
# One command to wire a phone to dev running in the container:
#   1. starts the adb server on the host so the container's adb CLI reaches the host's device
#      over `host.docker.internal` (the bridge in .devcontainer/devcontainer.json). Gradle's ddmlib
#      does NOT honor that bridge, so use Gradle only to build and `adb` for the device (install/logcat);
#   2. `adb reverse`s the dev-server ports so the DEVICE reaches them at http://localhost:<port>
#      (e.g. the sync API on :8081). No LAN exposure, no container rebuild.
#
# adb reverse forwards device → the adb server host's 127.0.0.1, so each port must answer on the host's
# localhost. Your editor's devcontainer port-forwarding already puts container ports there (VS Code
# auto-forwards listening ports; for others add `127.0.0.1:<port>:<port>` to appPort). That's why
# `adb reverse` is enough and socat isn't needed — the host→container hop is the editor's forward,
# the host→device hop is adb.
#
# WHY THIS SCRIPT IS macOS/Windows-SHAPED: there, Docker runs in a VM, so the container can never see
# a USB device and adb MUST live on the host. On a LINUX host the kernel is shared — pass the USB bus
# into the container instead and skip this script entirely (it will tell you how). Both are USB paths;
# wireless adb exists as a fallback for when there's no cable, not as the normal workflow.
#
# Prereqs:
#   - Android platform-tools (`adb`) on the host PATH, ideally version-matched with the container's
#     adb (37.0.0) to avoid protocol skew. On Windows use the native adb.exe from Git Bash.
#   - A device attached: USB (enable USB debugging) or wireless (set ADB_CONNECT, or `adb pair` first).
#
# Usage:
#   scripts/host/android-debug.sh                  # bridge + reverse the default ports (8081)
#   scripts/host/android-debug.sh 8081 9000        # reverse these ports instead
#   PORTS="8081 8082" scripts/host/android-debug.sh
#   ADB_CONNECT=192.168.88.50:5555 scripts/host/android-debug.sh   # connect a wireless device first
#   ADB_LISTEN=0.0.0.0 scripts/host/android-debug.sh               # required on a Linux host
#   SKIP_LINUX_HINT=1 ...                                          # silence the Linux advice
#
# Security: the adb server binds 127.0.0.1 by default, so it is NOT reachable from the LAN — only the
# container (via Docker Desktop's host.docker.internal → host loopback) and host-local processes. An
# adb server open on the network = full control of the attached device, so keep it on loopback. Only
# set ADB_LISTEN=0.0.0.0 if the container cannot see the device on loopback (the Linux case), and
# firewall tcp:5037 to the Docker bridge subnet when you do.
set -euo pipefail

PORTS="${*:-${PORTS:-8081}}"
ADB_PORT="${ANDROID_ADB_SERVER_PORT:-5037}"

# --- Host platform ------------------------------------------------------------
# Only used to pick a sane ADB_LISTEN default and to print platform-specific advice; every command
# below is portable. Git Bash / MSYS2 / Cygwin report MINGW64_NT-*, MSYS_NT-*, CYGWIN_NT-*.
case "$(uname -s)" in
  Darwin) HOST_OS=macos ;;
  Linux) HOST_OS=linux ;;
  MINGW* | MSYS* | CYGWIN*) HOST_OS=windows ;;
  *) HOST_OS=unknown ;;
esac

command -v adb >/dev/null || {
  echo "✗ adb not found on PATH — install the Android platform-tools on the host"
  [ "$HOST_OS" = windows ] && echo "  (Git Bash: add C:\\Users\\<you>\\AppData\\Local\\Android\\Sdk\\platform-tools to PATH)"
  exit 1
}

# On a Linux host this script is the wrong tool: `host.docker.internal` is absent unless you add
# `--add-host=host.docker.internal:host-gateway`, and even then it resolves to the bridge gateway
# (172.17.0.1) rather than host loopback — so a loopback-bound adb server is unreachable from the
# container. Run adb in the container instead (wirelessly, or over USB via a separate devcontainer
# config — NOT by editing the shared one, whose /dev/bus/usb mount would break macOS/Windows).
if [ "$HOST_OS" = linux ] && [ -z "${SKIP_LINUX_HINT:-}" ]; then
  cat <<'HINT'
⚠ Linux host detected — you do not need this script; use USB passthrough instead (Option B).
  This script exists because macOS/Windows run Docker in a VM. On Linux the kernel is shared, so give
  the container the USB bus and run adb in there — same cable, fewer moving parts, nothing on a socket.

  Use a SEPARATE devcontainer config so the shared one keeps working for macOS/Windows contributors
  (its /dev/bus/usb source path does not exist on those hosts). Create
  .devcontainer/linux-usb/devcontainer.json reusing ../Dockerfile, adding:

    "runArgs": ["--device-cgroup-rule=c 189:* rmw"],
    "mounts": ["source=/dev/bus/usb,target=/dev/bus/usb,type=bind"]

  VS Code offers it in the "Reopen in Container" picker. scripts/adb-env.sh detects the mount and
  leaves ANDROID_ADB_SERVER_* unset, so adb uses its own local server automatically.
  Host udev rules still gate device access: https://github.com/M0Rf30/android-udev-rules

  See docs/ANDROID_DEBUGGING.md. SKIP_LINUX_HINT=1 silences this. Continuing in 5s — Ctrl-C to bail.
HINT
  sleep 5
fi

# adb's server listener only supports two modes — it refuses a specific bind IP ("listening on
# specified hostname currently unsupported"):
#   loopback (default)  : -L tcp:PORT     → 127.0.0.1, NOT reachable from the LAN  (our default)
#   all interfaces      : -a              → 0.0.0.0, reachable from the LAN (firewall tcp:PORT!)
# Loopback is preferred and works on macOS/Windows: the container reaches it via Docker Desktop's
# host.docker.internal, which proxies to the host's loopback. A Linux host has no such proxy, so
# default to all-interfaces there — the one case where the LAN warning applies.
if [ -z "${ADB_LISTEN:-}" ]; then
  if [ "$HOST_OS" = linux ]; then ADB_LISTEN=0.0.0.0; else ADB_LISTEN=127.0.0.1; fi
fi
if [ "$ADB_LISTEN" = "0.0.0.0" ]; then
  LISTEN_ARGS=(-a)
  WHERE="all interfaces (0.0.0.0:${ADB_PORT}) — reachable from the LAN"
  echo "⚠ adb is listening on ALL interfaces: anyone who can reach tcp:${ADB_PORT} owns the device."
  echo "  Restrict it to the Docker bridge, e.g.:"
  echo "    sudo iptables -I INPUT ! -s 172.16.0.0/12 -p tcp --dport ${ADB_PORT} -j DROP"
else
  ADB_LISTEN=127.0.0.1
  LISTEN_ARGS=(-L "tcp:${ADB_PORT}")
  WHERE="loopback (127.0.0.1:${ADB_PORT}) — not on the LAN, container reaches it via host.docker.internal"
fi
PROBE=127.0.0.1 # both modes bind loopback too

echo "▸ host: ${HOST_OS}"
echo "▸ starting adb server on ${WHERE}…"
adb kill-server 2>/dev/null || true
# `server nodaemon` runs the server in the foreground; backgrounded so the script can continue.
adb "${LISTEN_ARGS[@]}" server nodaemon >/tmp/gt-adb.log 2>&1 &
SERVER_PID=$!
trap 'echo; echo "▸ stopping adb server"; kill "$SERVER_PID" 2>/dev/null || true; adb kill-server 2>/dev/null || true' EXIT INT TERM

# Portable TCP probe. `nc` is absent on a stock Git Bash and its -z flag varies between the
# openbsd/nmap builds, so prefer bash's /dev/tcp (built in on macOS, Linux and Git Bash bash).
port_open() {
  if (exec 3<>"/dev/tcp/$1/$2") 2>/dev/null; then
    return 0
  fi
  command -v nc >/dev/null 2>&1 && nc -z "$1" "$2" 2>/dev/null
}

# Wait for the server to accept connections WITHOUT an adb client (a client would race-spawn its own
# loopback-only server on the same port and ours would fail to bind).
for ((i = 0; i < 50; i++)); do
  if port_open "$PROBE" "$ADB_PORT"; then break; fi
  sleep 0.2
done
if ! port_open "$PROBE" "$ADB_PORT"; then
  echo "✗ adb server didn't come up. /tmp/gt-adb.log:"
  tail -20 /tmp/gt-adb.log 2>/dev/null | sed 's/^/    /'
  exit 1
fi

if [ -n "${ADB_CONNECT:-}" ]; then
  echo "▸ connecting wireless device $ADB_CONNECT (pair first if needed: adb pair <ip:port>)…"
  adb connect "$ADB_CONNECT"
fi

echo "▸ waiting for a device (plug in USB and approve the prompt, or set ADB_CONNECT)…"
adb wait-for-device
echo "▸ device: $(adb devices | awk 'NR==2{print $1" ("$2")"}')"

for p in $PORTS; do
  adb reverse "tcp:$p" "tcp:$p"
  echo "▸ reverse   device:$p → host localhost:$p"
done

echo
echo "✓ ready. In the app, point the server URL at http://localhost:8081 (and other ports as needed)."
echo "  Container side still works too: 'adb devices' and 'npm run android -- install' use this bridge."
echo "  Ctrl-C to stop the adb server."
adb reverse --list || true

wait "$SERVER_PID"
