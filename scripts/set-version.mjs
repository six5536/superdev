#!/usr/bin/env node
// Set one version across the whole project in lockstep: the Cargo workspace
// version (and the internal superdev-core dep), every package.json under
// packages/, the launcher's pinned optionalDependencies, and this repo's own
// skills pin in .superdev/.
//
// Usage: node scripts/set-version.mjs <version>

import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("usage: node scripts/set-version.mjs <semver>");
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

// Cargo.toml: workspace package version + internal superdev-core dependency pin.
const cargoPath = join(root, "Cargo.toml");
let cargo = readFileSync(cargoPath, "utf8");
cargo = cargo.replace(/^version = "[^"]*"$/m, `version = "${version}"`);
cargo = cargo.replace(
  /(superdev-core = \{ path = "crates\/lib\/superdev-core", version = ")[^"]*(" \})/,
  `$1${version}$2`,
);
writeFileSync(cargoPath, cargo);

// Every packages/*/package.json: version, plus the launcher's optionalDependencies.
const pkgsDir = join(root, "packages");
for (const name of readdirSync(pkgsDir)) {
  const p = join(pkgsDir, name, "package.json");
  let json;
  try {
    json = JSON.parse(readFileSync(p, "utf8"));
  } catch {
    continue;
  }
  json.version = version;
  if (json.optionalDependencies) {
    for (const dep of Object.keys(json.optionalDependencies)) {
      if (dep.startsWith("@six5536/superdev-")) {
        json.optionalDependencies[dep] = version;
      }
    }
  }
  writeFileSync(p, JSON.stringify(json, null, 2) + "\n");
}

// This repo is itself a superdev-managed repo, and the skills component's
// version comes from CARGO_PKG_VERSION. Left unbumped, the pin in .superdev
// falls behind the compiled registry and `superdev status` reports drift.
for (const [file, re] of [
  [".superdev/config.toml", /(\[skills\][^[]*?version = ")[^"]*(")/],
  [".superdev/lock.toml", /(\[components\.skills\][^[]*?version = ")[^"]*(")/],
]) {
  const p = join(root, file);
  const text = readFileSync(p, "utf8");
  if (!re.test(text)) {
    console.error(`could not find the skills version in ${file}`);
    process.exit(1);
  }
  writeFileSync(p, text.replace(re, `$1${version}$2`));
}

// Lockfiles record the workspace members' own versions, so they go stale on a
// bump. Left stale, `cargo publish --locked` fails and `npm ci` refuses to
// install. Refresh both rather than leave that for the release to discover.
const run = (cmd, args) => {
  try {
    execFileSync(cmd, args, { cwd: root, stdio: "inherit" });
  } catch {
    console.error(`\nfailed: ${cmd} ${args.join(" ")}`);
    console.error("The manifests were updated but the lockfiles are now stale.");
    process.exit(1);
  }
};

run("cargo", ["update", "--workspace", "--offline"]);
run("npm", ["install", "--package-lock-only", "--ignore-scripts", "--silent"]);

console.log(
  `set version to ${version} across Cargo workspace, packages/, .superdev/, and both lockfiles`,
);
