// Unified entry point routing to platform-specific scripts.

import { Command } from "commander";
import { resolve } from "node:path";
import { pass, fail, repoRoot, run } from "./lib.ts";

const ROOT = repoRoot();
const SCRIPTS = resolve(ROOT, "scripts");

// --- Helpers -----------------------------------------------------------------

function ios(args: string): number {
  return run(`node "${resolve(SCRIPTS, "ios.ts")}" ${args}`.trim());
}

function android(args: string): number {
  return run(`node "${resolve(SCRIPTS, "android.ts")}" ${args}`.trim());
}

function web(cmd: string): number {
  return run(`npm run ${cmd}`, { cwd: resolve(ROOT, "apps", "web") });
}

// --- Program -----------------------------------------------------------------

const program = new Command().name("dev").description("Unified dev CLI for {{superdev:project-name}}").version("0.3.0");

// The subcommand defaults to `run` (build + install + launch) so the `npm run dev:ios` /
// `dev:android` package scripts, which pass no subcommand, start the inner dev loop instead of
// failing on a missing argument.
program
  .command("ios")
  .description("Route to iOS script")
  .argument("[command]", "ios subcommand (build, run, test, etc.)", "run")
  .argument("[args...]", "additional arguments")
  .allowUnknownOption()
  .action((cmd: string, args: string[]) => {
    const code = ios([cmd, ...args].join(" "));
    process.exit(code);
  });

program
  .command("android")
  .description("Route to Android script")
  .argument("[command]", "android subcommand (build, run, test, etc.)", "run")
  .argument("[args...]", "additional arguments")
  .allowUnknownOption()
  .action((cmd: string, args: string[]) => {
    const code = android([cmd, ...args].join(" "));
    process.exit(code);
  });

program
  .command("web")
  .description("Route to web npm scripts")
  .argument("[command]", "dev | build | test | preview | lint | format", "dev")
  .action((cmd: string) => {
    const valid = ["dev", "build", "test", "preview", "lint", "format"];
    if (!valid.includes(cmd)) {
      fail(`Unknown web command: ${cmd}. Valid: ${valid.join(", ")}`);
      process.exit(1);
    }
    const code = web(cmd);
    process.exit(code);
  });

program
  .command("all")
  .description("Run a command on all platforms")
  .argument("[command]", "command to run on all platforms", "test")
  .argument("[args...]", "additional arguments")
  .action((cmd: string, args: string[]) => {
    const rest = args.join(" ");
    let exitCode = 0;

    console.log("\x1b[1m=== iOS ===\x1b[0m");
    if (ios([cmd, rest].join(" ").trim()) !== 0) exitCode = 1;

    console.log("\n\x1b[1m=== Android ===\x1b[0m");
    if (android([cmd, rest].join(" ").trim()) !== 0) exitCode = 1;

    console.log("\n\x1b[1m=== Web ===\x1b[0m");
    if (web(cmd) !== 0) exitCode = 1;

    if (exitCode === 0) pass(`All platforms: ${cmd} succeeded`);
    else fail("Some platforms failed");
    process.exit(exitCode);
  });

program.parse();
