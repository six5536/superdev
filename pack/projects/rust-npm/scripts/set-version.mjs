#!/usr/bin/env node
// Set one version across the whole project in lockstep: the Cargo workspace
// version (and the internal core dep), every package.json under packages/,
// and the launcher's pinned optionalDependencies — then refresh both
// lockfiles, which record the members' own versions and go stale on a bump.
//
// Usage: node scripts/set-version.mjs <version>

import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const SLUG = "{{superdev:project-slug}}";

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("usage: node scripts/set-version.mjs <semver>");
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

// Cargo.toml: workspace package version + internal core dependency pin.
const cargoPath = join(root, "Cargo.toml");
let cargo = readFileSync(cargoPath, "utf8");
cargo = cargo.replace(/^version = "[^"]*"$/m, `version = "${version}"`);
cargo = cargo.replace(
  new RegExp(`(${SLUG}-core = \\{ path = "crates/lib/${SLUG}-core", version = ")[^"]*(" \\})`),
  `$1${version}$2`,
);
writeFileSync(cargoPath, cargo);

// Every packages/*/package.json: version, plus the launcher's optionalDependencies.
for (const name of readdirSync(join(root, "packages"))) {
  const p = join(root, "packages", name, "package.json");
  let json;
  try {
    json = JSON.parse(readFileSync(p, "utf8"));
  } catch {
    continue;
  }
  json.version = version;
  if (json.optionalDependencies) {
    for (const dep of Object.keys(json.optionalDependencies)) {
      if (dep.startsWith(`${SLUG}-`)) {
        json.optionalDependencies[dep] = version;
      }
    }
  }
  writeFileSync(p, JSON.stringify(json, null, 2) + "\n");
}

// Left stale, `cargo publish --locked` fails and `npm ci` refuses to install.
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

console.log(`set version to ${version} across the Cargo workspace, packages/, and both lockfiles`);
