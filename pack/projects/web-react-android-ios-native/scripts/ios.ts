// CLI automation wrapper for iOS simulator development workflow.

import { Command } from "commander";
import { existsSync, readdirSync } from "node:fs";
import { resolve, join } from "node:path";
import { pass, fail, info, warn, repoRoot, screenshotDir, timestamp, run, capture, stream } from "./lib.ts";

// --- Configuration -----------------------------------------------------------

const BUNDLE_ID = "com.{{superdev:project-compact}}.app";
const SCHEME = "{{superdev:project-pascal}}";
const ROOT = repoRoot();
const IOS_DIR = resolve(ROOT, "apps", "ios-native");
const DEFAULT_SIM = "iPhone 17 Pro";

// --- Helpers -----------------------------------------------------------------

function getBootedSim(): string {
  const json = capture("xcrun simctl list devices booted -j");
  if (!json) return "";
  try {
    const data = JSON.parse(json) as {
      devices: Record<string, Array<{ udid: string; state: string }>>;
    };
    for (const runtime of Object.values(data.devices)) {
      for (const dev of runtime) {
        if (dev.state === "Booted") return dev.udid;
      }
    }
  } catch {
    /* ignore */
  }
  return "";
}

function ensureBooted(): string {
  let udid = getBootedSim();
  if (!udid) {
    warn(`No simulator booted. Booting ${DEFAULT_SIM}...`);
    cmdBoot(DEFAULT_SIM);
    udid = getBootedSim();
    if (!udid) {
      fail("Failed to boot simulator");
      process.exit(1);
    }
  }
  return udid;
}

function hasXcodeproj(): boolean {
  try {
    return readdirSync(IOS_DIR).some((f) => f.endsWith(".xcodeproj"));
  } catch {
    return false;
  }
}

function findAppBundle(): string | null {
  // Prefer .app bundle from xcodebuild (DerivedData) over bare SPM executable
  const derived = resolve(IOS_DIR, "DerivedData");
  const found = findFirst(derived, "{{superdev:project-pascal}}.app", "Debug-iphonesimulator");
  if (found) return found;

  const home = process.env["HOME"] ?? "";
  const xcodeDerived = resolve(home, "Library", "Developer", "Xcode", "DerivedData");
  const xcodeFound = findFirst(xcodeDerived, "{{superdev:project-pascal}}.app", "Debug-iphonesimulator");
  if (xcodeFound) return xcodeFound;

  // Fallback to SPM bare executable (won't install on simulator but may be useful for swift build)
  const spmApp = resolve(IOS_DIR, ".build", "debug", "{{superdev:project-pascal}}App");
  if (existsSync(spmApp)) return spmApp;

  return null;
}

function findFirst(base: string, name: string, pathContains?: string): string | null {
  if (!existsSync(base)) return null;
  const cmd = pathContains ? `find "${base}" -name "${name}" -path "*${pathContains}*" -type d 2>/dev/null | head -1` : `find "${base}" -name "${name}" -type d 2>/dev/null | head -1`;
  return capture(cmd) || null;
}

// --- Commands ----------------------------------------------------------------

function cmdBuild(config: string): void {
  info(`Building iOS app (${config})...`);
  if (hasXcodeproj()) {
    const xcConfig = config === "release" ? "Release" : "Debug";
    info(`Using xcodebuild (scheme: ${SCHEME})`);
    const code = run(`xcodebuild -scheme "${SCHEME}" -configuration "${xcConfig}" ` + `-destination "platform=iOS Simulator,name=${DEFAULT_SIM}" ` + `-derivedDataPath "${resolve(IOS_DIR, "DerivedData")}" build`, { cwd: IOS_DIR, tail: 20 });
    if (code !== 0) {
      fail("Build failed");
      process.exit(1);
    }
  } else {
    info("Using swift build (SPM)");
    const code = run("swift build", { cwd: IOS_DIR, tail: 20 });
    if (code !== 0) {
      fail("Build failed");
      process.exit(1);
    }
  }
  pass(`Build succeeded (${config})`);
}

function cmdBoot(sim: string): void {
  const udid = getBootedSim();
  if (udid) {
    warn(`Simulator already booted (${udid})`);
    return;
  }
  info(`Booting simulator: ${sim}`);
  run(`xcrun simctl boot "${sim}"`);
  run("open -a Simulator");
  pass(`Simulator booted: ${sim}`);
}

function cmdShutdown(): void {
  info("Shutting down booted simulator...");
  run("xcrun simctl shutdown booted");
  pass("Simulator shut down");
}

function cmdInstall(): void {
  const udid = ensureBooted();
  const appPath = findAppBundle();
  if (!appPath) {
    fail("No .app bundle found. Run 'npm run ios -- build' first.");
    process.exit(1);
  }
  info(`Installing ${appPath}`);
  const code = run(`xcrun simctl install "${udid}" "${appPath}"`);
  if (code !== 0) {
    fail("Install failed. Is the path a valid .app bundle?");
    process.exit(1);
  }
  pass("App installed");
}

function cmdLaunch(): void {
  const udid = ensureBooted();
  info(`Launching ${BUNDLE_ID}`);
  run(`xcrun simctl launch "${udid}" "${BUNDLE_ID}"`);
  pass("App launched");
}

function cmdTerminate(): void {
  const udid = ensureBooted();
  info(`Terminating ${BUNDLE_ID}`);
  run(`xcrun simctl terminate "${udid}" "${BUNDLE_ID}"`);
  pass("App terminated");
}

function cmdErase(): void {
  info("Erasing booted simulator...");
  run("xcrun simctl erase booted");
  pass("Simulator erased");
}

function cmdScreenshot(path?: string): void {
  const udid = ensureBooted();
  const dest = path ?? join(screenshotDir(), `ios-${timestamp()}.png`);
  info(`Taking screenshot → ${dest}`);
  run(`xcrun simctl io "${udid}" screenshot "${dest}"`);
  pass(`Screenshot saved: ${dest}`);
}

function cmdLogs(opts: { filter?: string }): void {
  const udid = ensureBooted();
  info("Streaming logs (Ctrl-C to stop)...");
  const predicate = opts.filter ? `subsystem == "${BUNDLE_ID}" AND category == "${opts.filter}"` : `subsystem == "${BUNDLE_ID}"`;
  stream("xcrun", ["simctl", "spawn", udid, "log", "stream", "--predicate", predicate]);
}

function cmdAppinfo(): void {
  const udid = ensureBooted();
  info(`App info for ${BUNDLE_ID}`);
  run(`xcrun simctl appinfo "${udid}" "${BUNDLE_ID}"`);
}

function cmdContainer(): void {
  const udid = ensureBooted();
  const result = capture(`xcrun simctl get_app_container "${udid}" "${BUNDLE_ID}" data`) || capture(`xcrun simctl get_app_container "${udid}" "${BUNDLE_ID}"`);
  if (!result) {
    fail("Could not find app container. Is the app installed?");
    process.exit(1);
  }
  console.log(result);
}

function cmdRun(config: string): void {
  cmdBuild(config);
  cmdInstall();
  cmdLaunch();
}

// --- Debug Server Commands ---------------------------------------------------

const DEBUG_SERVER = "http://localhost:8080";

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
  const dest = path ?? join(screenshotDir(), `ios-debug-${timestamp()}.png`);
  info(`Capturing debug screenshot → ${dest}`);
  const code = run(`curl -s -o "${dest}" ${DEBUG_SERVER}/debug/screenshot`);
  if (code !== 0) {
    fail("Debug screenshot failed.");
    process.exit(1);
  }
  pass(`Debug screenshot saved: ${dest}`);
}

function cmdTest(): void {
  if (hasXcodeproj()) {
    info("Running tests via xcodebuild...");
    run(`xcodebuild test -scheme "${SCHEME}" ` + `-destination "platform=iOS Simulator,name=${DEFAULT_SIM}" ` + `-derivedDataPath "${resolve(IOS_DIR, "DerivedData")}"`, { cwd: IOS_DIR, tail: 30 });
  } else {
    info("Running tests via swift test...");
    run("swift test", { cwd: IOS_DIR });
  }
}

// --- Program -----------------------------------------------------------------

const program = new Command().name("ios").description("iOS simulator automation for {{superdev:project-name}}").version("0.3.0");

program
  .command("build")
  .description("Build the iOS app")
  .argument("[config]", "debug or release", "debug")
  .action((config: string) => cmdBuild(config));

program
  .command("boot")
  .description("Boot iOS simulator")
  .argument("[simulator]", "simulator name", DEFAULT_SIM)
  .action((sim: string) => cmdBoot(sim));

program
  .command("shutdown")
  .description("Shut down booted simulator")
  .action(() => cmdShutdown());

program
  .command("install")
  .description("Install app on booted simulator")
  .action(() => cmdInstall());

program
  .command("launch")
  .description("Launch app on booted simulator")
  .action(() => cmdLaunch());

program
  .command("terminate")
  .description("Terminate running app")
  .action(() => cmdTerminate());

program
  .command("erase")
  .description("Erase booted simulator")
  .action(() => cmdErase());

program
  .command("screenshot")
  .description("Take a screenshot")
  .argument("[path]", "output file path (default: tmp/screenshots/)")
  .action((path?: string) => cmdScreenshot(path));

program
  .command("logs")
  .description("Stream app logs")
  .option("--filter <category>", "filter by log category")
  .action((opts: { filter?: string }) => cmdLogs(opts));

program
  .command("appinfo")
  .description("Show app info on simulator")
  .action(() => cmdAppinfo());

program
  .command("container")
  .description("Print app data container path")
  .action(() => cmdContainer());

program
  .command("run")
  .description("Build + install + launch")
  .argument("[config]", "debug or release", "debug")
  .action((config: string) => cmdRun(config));

program
  .command("test")
  .description("Run tests")
  .action(() => cmdTest());

program
  .command("devices")
  .description("List available simulators")
  .action(() => {
    run("xcrun simctl list devices available");
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
