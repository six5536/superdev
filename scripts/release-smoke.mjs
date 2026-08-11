#!/usr/bin/env node
// Behavioural smoke test for a compiled superdev binary: check --version,
// --help, and usage-error exit codes. Release CI runs it against each built
// artifact its runner can execute; locally run `npm run smoke` after
// `cargo build --release -p superdev`.
//
// The project is a scaffold, so this exercises only the plumbing the release
// pipeline depends on. Extend it as real verbs land.

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";

const bin =
  process.argv[2] ??
  join("target", "release", process.platform === "win32" ? "superdev.exe" : "superdev");
if (!existsSync(bin)) {
  fail(`no binary at ${bin} — build first: cargo build --release -p superdev`);
}

function fail(message) {
  console.error(`release-smoke: ${message}`);
  process.exit(1);
}

/** Run the binary, asserting the exit status; returns the spawn result. */
function run(args, expectStatus) {
  const r = spawnSync(bin, args, { encoding: "utf8" });
  if (r.error) {
    fail(`failed to run ${bin}: ${r.error.message}`);
  }
  if (r.status !== expectStatus) {
    console.error(r.stdout);
    console.error(r.stderr);
    fail(`\`superdev ${args.join(" ")}\` exited ${r.status}, expected ${expectStatus}`);
  }
  return r;
}

const version = run(["--version"], 0).stdout.trim();
if (!/^superdev \d+\.\d+\.\d+/.test(version)) {
  fail(`unexpected --version output: ${version}`);
}

const help = run(["--help"], 0).stdout;
if (!help.includes("Usage:")) {
  fail(`--help printed no usage:\n${help}`);
}

const completions = run(["completions", "bash"], 0).stdout;
if (!completions.includes("superdev")) {
  fail("completions bash printed nothing recognisable");
}

// A usage error must exit 2 — the launcher smoke relies on this code too.
run(["--definitely-not-a-flag"], 2);

console.log(`release-smoke OK: ${version} (${bin})`);
