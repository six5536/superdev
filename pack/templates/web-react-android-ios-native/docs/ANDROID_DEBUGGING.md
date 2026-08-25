# Android debugging from the devcontainer

The container has `adb` (platform-tools 37.0.0) but **no USB access on macOS or Windows** — Docker
runs in a VM there, so the USB bus is on the other side of a hypervisor. This doc covers the three
ways to get a real phone talking to a dev server running inside the container, and which one to pick
for your host OS.

| Option | Works on | Host adb server? | `adb reverse` targets | Use it when |
| ------ | -------- | ---------------- | --------------------- | ----------- |
| **A — host adb bridge** | macOS, Windows | yes | host loopback | **default** — plug in a cable and go |
| **B — USB passthrough** | Linux only | no | container loopback | you're on Linux; strictly simpler than A |
| **C — wireless adb** | any host | no | container loopback | no cable available, or testing on-device Wi-Fi behaviour |

**A is the default on macOS and Windows, B on Linux — both are USB, and USB is what you want.** It's
fast, it survives reboots, and it needs no repeated setup. Wireless (C) is a fallback, not a
recommendation: pairing has to be redone whenever the phone reboots or toggles wireless debugging,
`adb install` is materially slower over Wi-Fi, and the connection drops when the phone sleeps or roams.
Reach for it when you have no cable or you're specifically testing Wi-Fi behaviour.

`adb reverse tcp:8081 tcp:8081` makes the **device** reach `http://localhost:8081`. It always
forwards to the loopback of whichever machine runs the adb server — that is the single fact that
decides the shape of each option below.

The container picks its adb target automatically, so on macOS/Windows Option A needs no setup inside
the container at all — see [How the container finds adb](#how-the-container-finds-adb).

---

## Option A — host adb bridge (macOS, Windows)

Run on the host, not in the container:

```bash
scripts/host/android-debug.sh              # reverse :8081
scripts/host/android-debug.sh 8081 9000    # reverse specific ports
ADB_CONNECT=192.168.88.50:5555 scripts/host/android-debug.sh   # wireless device
```

It starts an adb server bound to `127.0.0.1`, waits for a device, then reverses the requested ports.
The container's `adb` CLI reaches that server via `ANDROID_ADB_SERVER_ADDRESS=host.docker.internal`
(set by mise — see [below](#how-the-container-finds-adb)), which Docker Desktop proxies to host loopback.

Because reverse lands on **host** loopback, each port must answer there — your editor's devcontainer
port forwarding does that (VS Code auto-forwards listening ports; for anything it misses, add
`127.0.0.1:<port>:<port>` to `appPort`). That is why no `socat` is needed: host→container is the
editor's forward, host→device is adb.

**Gradle caveat.** Gradle's bundled ddmlib device monitor ignores `ANDROID_ADB_SERVER_ADDRESS`
(and `ADB_SERVER_SOCKET`), so `./gradlew installDebug` cannot see the device. Build with Gradle,
install with adb:

```bash
npm run android -- build      # gradle assembleDebug
npm run android -- install    # adb install -r (goes over the bridge)
npm run android -- logs
```

**Windows specifics.** Run the script from Git Bash against the native `adb.exe`
(`…/AppData/Local/Android/Sdk/platform-tools` on PATH). Keep adb on Windows proper — an adb server
inside a WSL2 distro is *not* what `host.docker.internal` resolves to. Getting USB into a WSL2 distro
would additionally require [usbipd-win](https://github.com/dorssel/usbipd-win); Option A avoids that.

**Security.** An adb server reachable over the network means full control of the attached device.
The script binds loopback by default and only opens all interfaces when you ask for it.

---

## Option B — USB passthrough into the container (Linux only)

On a Linux host the kernel is shared, so the container can own the device directly: no host adb
server, no port-forward hop, and nothing exposed on a socket.

The catch is that this is the one option that has to change the **container definition**, and its
config is not portable — `source=/dev/bus/usb` has no meaning on a macOS or Windows host (the path
does not exist there and is not a shared path), so putting it in the committed
`.devcontainer/devcontainer.json` breaks everyone else. Do **not** edit the default config. Add a
second, committed config instead and let VS Code offer it in the "Reopen in Container" picker:

```
.devcontainer/devcontainer.json          # default, OS-neutral — leave alone
.devcontainer/linux-usb/devcontainer.json # Linux + USB, reuses ../Dockerfile
```

The Linux variant is the default config plus:

```jsonc
"runArgs": ["--device-cgroup-rule=c 189:* rmw"],
"mounts": ["source=/dev/bus/usb,target=/dev/bus/usb,type=bind"],
```

and **minus** `ANDROID_ADB_SERVER_ADDRESS`/`ANDROID_ADB_SERVER_PORT`, or adb keeps dialing a host
server that isn't there. Blanking them does not work — adb rejects an empty value with
`cannot connect to daemon at tcp::5037: no host in ':5037'` — so they must be absent.

Bind-mounting the whole `/dev/bus/usb` directory (rather than `--device`) plus the cgroup rule for
major 189 is what survives unplug/replug — a `--device` entry is resolved once at container create.

Host-side [udev rules](https://github.com/M0Rf30/android-udev-rules) still gate access to the device
node for non-root users.

Then everything happens in the container, and reverse points at container loopback — where the dev
server actually listens:

```bash
adb devices
adb reverse tcp:8081 tcp:8081
```

Why Option A does not work well here: `host.docker.internal` does not resolve on Docker Engine unless
you add `--add-host=host.docker.internal:host-gateway`, and even then it points at the bridge gateway
(`172.17.0.1`) rather than host loopback — so a loopback-bound adb server is unreachable from the
container, forcing `ADB_LISTEN=0.0.0.0` and a LAN-exposed adb. The script warns and defaults to
all-interfaces if you run it on Linux anyway; firewall `tcp:5037` to the bridge subnet if you do.

---

## Option C — wireless adb from inside the container (fallback)

Only worth it when you have no cable, or when you're deliberately testing behaviour over Wi-Fi. The
costs are real: re-pairing after each phone reboot or wireless-debugging toggle, noticeably slower
`adb install`, and drops when the phone sleeps or roams between APs.

Container→LAN egress is NAT'd on all three platforms, so the container can reach the phone's LAN IP
directly. On the phone enable **Developer options → Wireless debugging**, then in the container:

```bash
# adb must use its OWN local server, so drop the container's host-bridge vars per invocation:
alias adbl='env -u ANDROID_ADB_SERVER_ADDRESS -u ANDROID_ADB_SERVER_PORT adb'

adbl pair <phone-ip>:<pairing-port>    # code shown on the phone
adbl connect <phone-ip>:5555
adbl reverse tcp:8081 tcp:8081
```

`env -u` is the mechanism that works, and it needs no config change. Two things that do **not** work:
blanking the vars (`no host in ':5037'`), and `-H 127.0.0.1` — adb treats an explicit `-H` as a remote
server and refuses to spawn a daemon there (`* cannot start server on remote host`), so it only helps
when a server is already listening.

mDNS discovery (`adb mdns services`) will not cross the NAT, so use the explicit IP. Pairing generally
has to be redone after the phone reboots or toggles wireless debugging.

---

## How the container finds adb

`ANDROID_ADB_SERVER_ADDRESS`/`_PORT` are set by **mise**, not `devcontainer.json`, from a one-line
probe in `scripts/adb-env.sh`:

```toml
# mise.toml
[env]
_.source = "{{ config_root }}/scripts/adb-env.sh"
```

```bash
# scripts/adb-env.sh
if [ ! -d /dev/bus/usb ]; then
  export ANDROID_ADB_SERVER_ADDRESS="${ANDROID_ADB_SERVER_ADDRESS:-host.docker.internal}"
  export ANDROID_ADB_SERVER_PORT="${ANDROID_ADB_SERVER_PORT:-5037}"
fi
```

No `/dev/bus/usb` means Docker is in a VM and adb must live on the host (Option A). Its presence means
the USB bus was passed in and adb runs locally (Option B). A filesystem test rather than a DNS probe of
`host.docker.internal`, because this runs on every shell spawn and a resolver round-trip — plus its
timeout — has no business in that path.

Three constraints drove this, all verified against adb 37.0.0 and mise 2026.7.11:

- **mise cannot branch on the host OS.** `{{ os() }}` evaluates *inside* the container, so it is
  `linux` whether the host is macOS, Windows or Linux. Anything host-dependent has to be probed at
  runtime; templating can't do it.
- **`containerEnv` cannot omit a variable, only set one** — and adb rejects a blank value outright
  (`cannot connect to daemon at tcp::5037: no host in ':5037'`). Option B needs the vars *absent*,
  which is why they had to leave `devcontainer.json` for somewhere that can express "sometimes".
- **mise can add or change vars but never unset one.** So leaving them in `containerEnv` as well would
  silently override the probe. They live in exactly one place.

**Overriding.** `.mise.local.toml` (gitignored) layers over `mise.toml`, but mind the
trap: it lives in the **mounted workspace**, so it is not per-machine — the container and the host both
read the same file. Setting a host-shaped value there leaks it into the container — e.g. an
`ANDROID_HOME = "~/Library/Android/sdk"` meant for a macOS host resolves to a nonexistent path inside
the container (and isn't tilde-expanded), leaving `ANDROID_HOME` and `ANDROID_SDK_ROOT` disagreeing.

So use `.mise.local.toml` only for values valid in *both* places, and prefer a runtime probe (as
`adb-env.sh` does) for anything that genuinely differs. For one-off overrides:

```bash
env -u ANDROID_ADB_SERVER_ADDRESS -u ANDROID_ADB_SERVER_PORT adb devices   # force a local server
adb -H other-host -P 5037 devices                                          # force a specific server
```

`-H` overrides the env var but only selects an *already-running* server — adb refuses to start a daemon
on any explicitly-given host (`* cannot start server on remote host`), so it cannot stand in for `env -u`.

Option B additionally needs `runArgs` and `mounts`, which have no conditional form at all
(`${localEnv:…}` doesn't help: an empty mount source is a malformed mount, not an omitted one). That is
why it goes in a separate `.devcontainer/linux-usb/devcontainer.json` rather than the default config,
whose `/dev/bus/usb` source path doesn't even exist on a macOS or Windows host.

---

## Ports

`:8081` is the Android native debug server and the only port reversed by default. `:5173` is the Vite dev server — pass it
too when you want a device to load the web app with live reload:

```bash
scripts/host/android-debug.sh 8081 5173     # Option A, on the host
adb reverse tcp:5173 tcp:5173               # Options B and C, in the container
```

Both servers bind `0.0.0.0` inside the container (`server.host`/`preview.host` in
`apps/web/vite.config.ts`), with `strictPort` so they never silently move off the port the reverse
points at. Pass extra ports positionally or via `PORTS="8081 5173"`.
