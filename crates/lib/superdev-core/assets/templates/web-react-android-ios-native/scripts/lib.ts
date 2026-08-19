// Shared helpers for CLI automation scripts.

import { execSync, spawn, type SpawnOptions } from "node:child_process";
import { existsSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";

// --- Colors -----------------------------------------------------------------

const FMT = {
  red: "\x1b[0;31m",
  green: "\x1b[0;32m",
  yellow: "\x1b[0;33m",
  cyan: "\x1b[0;36m",
  bold: "\x1b[1m",
  nc: "\x1b[0m",
} as const;

export function pass(msg: string): void {
  console.log(`${FMT.green}${FMT.bold}✓ ${msg}${FMT.nc}`);
}

export function fail(msg: string): void {
  console.error(`${FMT.red}${FMT.bold}✗ ${msg}${FMT.nc}`);
}

export function info(msg: string): void {
  console.log(`${FMT.cyan}→ ${msg}${FMT.nc}`);
}

export function warn(msg: string): void {
  console.log(`${FMT.yellow}⚠ ${msg}${FMT.nc}`);
}

// --- Paths ------------------------------------------------------------------

export function repoRoot(): string {
  return execSync("git rev-parse --show-toplevel", { encoding: "utf-8" }).trim();
}

export function ensureDir(dir: string): void {
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
}

export function screenshotDir(): string {
  const dir = resolve(repoRoot(), "tmp", "screenshots");
  ensureDir(dir);
  return dir;
}

export function timestamp(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`;
}

// --- Execution --------------------------------------------------------------

/** Run a command synchronously, inherit stdio, return exit code. */
export function run(cmd: string, opts?: { cwd?: string; tail?: number }): number {
  try {
    const output = execSync(cmd, {
      cwd: opts?.cwd,
      encoding: "utf-8",
      stdio: opts?.tail ? "pipe" : "inherit",
    });
    if (opts?.tail && output) {
      const lines = output.split("\n");
      console.log(lines.slice(-opts.tail).join("\n"));
    }
    return 0;
  } catch (err: unknown) {
    if (err && typeof err === "object" && "status" in err) {
      const e = err as { status: number | null; stdout?: string };
      if (opts?.tail && e.stdout) {
        const lines = (e.stdout as string).split("\n");
        console.log(lines.slice(-opts.tail).join("\n"));
      }
      return (e.status as number) ?? 1;
    }
    return 1;
  }
}

/** Run a command capturing stdout AND the exit code — capture() collapses every failure to "". */
export function tryCapture(cmd: string, opts?: { cwd?: string }): { code: number; stdout: string } {
  try {
    const stdout = execSync(cmd, { cwd: opts?.cwd, encoding: "utf-8", stdio: ["pipe", "pipe", "pipe"] });
    return { code: 0, stdout: stdout.trim() };
  } catch (err: unknown) {
    const e = (err ?? {}) as { status?: number | null; stdout?: string };
    return { code: e.status ?? 1, stdout: (e.stdout ?? "").trim() };
  }
}

/** Run a command and capture stdout. Returns empty string on failure. */
export function capture(cmd: string, opts?: { cwd?: string }): string {
  try {
    return execSync(cmd, { cwd: opts?.cwd, encoding: "utf-8", stdio: ["pipe", "pipe", "pipe"] }).trim();
  } catch {
    return "";
  }
}

/** Spawn a long-running process (e.g. log streaming) with inherited stdio. */
export function stream(cmd: string, args: string[], opts?: SpawnOptions): void {
  const child = spawn(cmd, args, { stdio: "inherit", ...opts });
  child.on("error", (err) => {
    fail(`Failed to start: ${cmd} — ${err.message}`);
    process.exit(1);
  });
  child.on("exit", (code) => {
    process.exit(code ?? 0);
  });
}
