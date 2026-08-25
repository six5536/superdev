// CLI automation wrapper for Android emulator/device development workflow.

import { Command } from "commander";
import { existsSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { execSync } from "node:child_process";
import { pass, fail, info, warn, repoRoot, screenshotDir, timestamp, run, capture, tryCapture, stream } from "./lib.ts";

// --- Configuration -----------------------------------------------------------

const PACKAGE = "com.{{superdev:project-compact}}.app";
const ACTIVITY = ".MainActivity";
const ROOT = repoRoot();
const ANDROID_DIR = resolve(ROOT, "apps", "android-native");

// --- Helpers -----------------------------------------------------------------

function checkAdb(): void {
  if (!capture("which adb")) {
    fail("adb not found. Install Android SDK platform-tools.");
    process.exit(1);
  }
}

// Every adb call goes to whichever server scripts/adb-env.sh selected. When that's the host bridge
// (macOS/Windows) and the bridge isn't running, adb hangs for 30s+ rather than failing: Docker
// Desktop's gateway ACCEPTS the TCP connection and then goes silent, so adb waits on a handshake that
// never arrives. A connect probe can't detect this — it succeeds instantly — so bound adb itself and
// translate the timeout into something actionable.
const ADB_TIMEOUT_S = 8;

function adbServerLabel(): string {
  const host = process.env.ANDROID_ADB_SERVER_ADDRESS;
  return host ? `the adb server at ${host}:${process.env.ANDROID_ADB_SERVER_PORT ?? "5037"}` : "the local adb server";
}

function failAdbTimeout(): never {
  fail(`adb timed out after ${ADB_TIMEOUT_S}s talking to ${adbServerLabel()}.`);
  if (process.env.ANDROID_ADB_SERVER_ADDRESS) {
    info("Start the bridge on the host:      scripts/host/android-debug.sh");
    info("Or use a container-local server:   env -u ANDROID_ADB_SERVER_ADDRESS -u ANDROID_ADB_SERVER_PORT adb ...");
    info("Details:                           docs/ANDROID_DEBUGGING.md");
  }
  process.exit(1);
}

/** `adb devices` bounded by ADB_TIMEOUT_S; exits with guidance if the server is unreachable. */
function adbDevices(args = ""): string {
  const { code, stdout } = tryCapture(`timeout ${ADB_TIMEOUT_S} adb devices ${args}`.trim());
  if (code === 124) failAdbTimeout();
  return stdout;
}

function checkDevice(): void {
  const lines = adbDevices()
    .split("\n")
    .filter((l) => l.endsWith("device"));
  if (lines.length === 0) {
    fail("No Android device/emulator connected. Start an emulator or connect a device.");
    process.exit(1);
  }
  if (lines.length > 1) {
    warn(`Multiple devices connected (${lines.length}). Using first device.`);
    warn("Set ANDROID_SERIAL to target a specific device.");
  }
}

function findApk(config = "debug"): string | null {
  const apkDir = resolve(ANDROID_DIR, "app", "build", "outputs", "apk", config);
  if (!existsSync(apkDir)) return null;
  return capture(`find "${apkDir}" -name "*.apk" -type f 2>/dev/null | head -1`) || null;
}

// --- Commands ----------------------------------------------------------------

function cmdBuild(config: string): void {
  info(`Building Android app (${config})...`);
  const task = config === "release" ? "assembleRelease" : "assembleDebug";
  const code = run(`./gradlew ${task}`, { cwd: ANDROID_DIR, tail: 20 });
  if (code !== 0) {
    fail("Build failed");
    process.exit(1);
  }
  pass(`Build succeeded (${config})`);
}

function cmdInstall(config: string): void {
  checkAdb();
  checkDevice();
  const apk = findApk(config);
  if (!apk) {
    fail("No APK found. Run 'npm run android -- build' first.");
    process.exit(1);
  }
  info(`Installing ${apk}`);
  // `adb install` reports failures (signature mismatch, insufficient space, downgrade) on stdout
  // AND with a non-zero exit, so the status must be checked — reporting success regardless hides
  // the failure and sends you looking for the bug in the app instead of the install.
  const code = run(`adb install -r "${apk}"`);
  if (code !== 0) {
    fail("Install failed — the app on the device is unchanged.");
    info("Signature mismatch? The installed build was signed with a different debug keystore:");
    info(`  adb uninstall ${PACKAGE}   # WARNING: erases app data (profiles, sessions)`);
    process.exit(code);
  }
  pass("App installed");
}

function cmdLaunch(): void {
  checkAdb();
  checkDevice();
  info(`Launching ${PACKAGE}/${ACTIVITY}`);
  run(`adb shell am start -n "${PACKAGE}/${ACTIVITY}"`);
  pass("App launched");
}

function cmdTerminate(): void {
  checkAdb();
  checkDevice();
  info(`Terminating ${PACKAGE}`);
  run(`adb shell am force-stop "${PACKAGE}"`);
  pass("App terminated");
}

function cmdClear(): void {
  checkAdb();
  checkDevice();
  info(`Clearing app data for ${PACKAGE}`);
  run(`adb shell pm clear "${PACKAGE}"`);
  pass("App data cleared");
}

function cmdScreenshot(path?: string): void {
  checkAdb();
  checkDevice();
  const dest = path ?? join(screenshotDir(), `android-${timestamp()}.png`);
  info(`Taking screenshot → ${dest}`);
  try {
    const buf = execSync("adb exec-out screencap -p", { maxBuffer: 50 * 1024 * 1024 });
    writeFileSync(dest, buf);
    pass(`Screenshot saved: ${dest}`);
  } catch {
    fail("Screenshot failed");
    process.exit(1);
  }
}

function cmdLogs(opts: { filter?: string }): void {
  checkAdb();
  checkDevice();
  run("adb logcat -c");
  info("Streaming logs (Ctrl-C to stop)...");
  const tag = opts.filter ?? "{{superdev:project-pascal}}";
  stream("adb", ["logcat", "-s", tag]);
}

function cmdUidump(): void {
  checkAdb();
  checkDevice();
  info("Dumping UI hierarchy...");
  const remote = "/sdcard/window_dump.xml";
  run(`adb shell uiautomator dump "${remote}"`);
  run(`adb shell cat "${remote}"`);
  run(`adb shell rm -f "${remote}"`);
}

function cmdAppinfo(): void {
  checkAdb();
  checkDevice();
  info(`App info for ${PACKAGE}`);
  run(`adb shell dumpsys package "${PACKAGE}" | head -80`);
}

function cmdRun(config: string): void {
  cmdBuild(config);
  cmdInstall(config);
  cmdLaunch();
}

// --- Debug Server Commands ---------------------------------------------------

const DEBUG_SERVER = "http://localhost:8081";

function cmdDebugPing(): void {
  info("Pinging debug server...");
  const result = capture(`curl -s ${DEBUG_SERVER}/debug/ping`);
  if (!result) {
    fail("Debug server not reachable. Is the app running in debug mode?");
    process.exit(1);
  }
  console.log(result);
}

function cmdDebugState(key?: string): void {
  const path = key ? `/debug/state/${key}` : "/debug/state";
  const result = capture(`curl -s ${DEBUG_SERVER}${path}`);
  if (!result) {
    fail("Debug server not reachable.");
    process.exit(1);
  }
  console.log(result);
}

function cmdDebugLogs(): void {
  const result = capture(`curl -s ${DEBUG_SERVER}/debug/logs`);
  if (!result) {
    fail("Debug server not reachable.");
    process.exit(1);
  }
  console.log(result);
}

function cmdDebugNavigate(route: string): void {
  info(`Navigating to ${route}...`);
  const result = capture(`curl -s -X POST ${DEBUG_SERVER}/debug/navigate -H "Content-Type: application/json" -d '{"route":"${route}"}'`);
  if (!result) {
    fail("Debug server not reachable.");
    process.exit(1);
  }
  console.log(result);
}

function cmdDebugAction(type: string, payload?: string): void {
  const body = payload ? `{"type":"${type}","payload":${payload}}` : `{"type":"${type}","payload":{}}`;
  const result = capture(`curl -s -X POST ${DEBUG_SERVER}/debug/action -H "Content-Type: application/json" -d '${body}'`);
  if (!result) {
    fail("Debug server not reachable.");
    process.exit(1);
  }
  console.log(result);
}

function cmdDebugScreenshot(path?: string): void {
  const dest = path ?? join(screenshotDir(), `android-debug-${timestamp()}.png`);
  info(`Capturing debug screenshot → ${dest}`);
  const code = run(`curl -s -o "${dest}" ${DEBUG_SERVER}/debug/screenshot`);
  if (code !== 0) {
    fail("Debug screenshot failed.");
    process.exit(1);
  }
  pass(`Debug screenshot saved: ${dest}`);
}

function cmdTest(): void {
  info("Running Android tests...");
  const code = run("./gradlew test", { cwd: ANDROID_DIR, tail: 30 });
  if (code !== 0) {
    fail("Tests failed");
    process.exit(1);
  }
  pass("Tests passed");
}

// --- Program -----------------------------------------------------------------

const program = new Command().name("android").description("Android device/emulator automation for {{superdev:project-name}}").version("0.3.0");

program
  .command("build")
  .description("Build the Android app")
  .argument("[config]", "debug or release", "debug")
  .action((config: string) => cmdBuild(config));

program
  .command("install")
  .description("Install APK on device/emulator")
  .argument("[config]", "debug or release", "debug")
  .action((config: string) => cmdInstall(config));

program
  .command("launch")
  .description("Launch the app")
  .action(() => cmdLaunch());

program
  .command("terminate")
  .description("Force-stop the app")
  .action(() => cmdTerminate());

program
  .command("clear")
  .description("Clear app data")
  .action(() => cmdClear());

program
  .command("screenshot")
  .description("Take a screenshot")
  .argument("[path]", "output file path (default: tmp/screenshots/)")
  .action((path?: string) => cmdScreenshot(path));

program
  .command("logs")
  .description("Stream logcat")
  .option("--filter <tag>", "logcat tag (default: {{superdev:project-pascal}})")
  .action((opts: { filter?: string }) => cmdLogs(opts));

program
  .command("uidump")
  .description("Dump UI hierarchy (XML)")
  .action(() => cmdUidump());

program
  .command("appinfo")
  .description("Show package info")
  .action(() => cmdAppinfo());

program
  .command("run")
  .description("Build + install + launch")
  .argument("[config]", "debug or release", "debug")
  .action((config: string) => cmdRun(config));

program
  .command("test")
  .description("Run unit tests (JVM + Robolectric Compose UI tests; no device needed)")
  .action(() => cmdTest());

program
  .command("devices")
  .description("List connected devices")
  .action(() => {
    checkAdb();
    console.log(adbDevices("-l"));
  });

program
  .command("emulators")
  .description("List available AVDs")
  .action(() => {
    if (!capture("which emulator")) {
      fail("emulator command not found. Check your ANDROID_HOME/SDK setup.");
      process.exit(1);
    }
    run("emulator -list-avds");
  });

// Debug server commands
program
  .command("debug-ping")
  .description("Ping the debug server")
  .action(() => cmdDebugPing());

program
  .command("state")
  .description("Get app state from debug server")
  .argument("[key]", "specific state key (e.g. routes, audio)")
  .action((key?: string) => cmdDebugState(key));

program
  .command("debug-logs")
  .description("Get logs from debug server")
  .action(() => cmdDebugLogs());

program
  .command("navigate")
  .description("Navigate app via debug server")
  .argument("<route>", "route to navigate to")
  .action((route: string) => cmdDebugNavigate(route));

program
  .command("debug-action")
  .description("Trigger an action via debug server")
  .argument("<type>", "action type")
  .argument("[payload]", "JSON payload")
  .action((type: string, payload?: string) => cmdDebugAction(type, payload));

program
  .command("debug-screenshot")
  .description("Capture screenshot via debug server")
  .argument("[path]", "output file path")
  .action((path?: string) => cmdDebugScreenshot(path));

program.parse();
