#!/usr/bin/env node
// Assert that one version is used consistently everywhere: the Cargo
// workspace, the internal core pin, and every package.json under packages/
// (including the launcher's optionalDependencies).
//
// Usage: node scripts/verify-version.mjs [expected-version]
//
// With no argument it only checks internal consistency. With one, it also
// checks everything matches that version — this is how the release workflow
// verifies a tag against the tree.

import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const SLUG = "{{superdev:project-slug}}";

const expected = process.argv[2]?.replace(/^v/, "");
const root = join(dirname(fileURLToPath(import.meta.url)), "..");

/** Every place a version is written, as { where, version } records. */
const found = [];
const problems = [];

const record = (where, version) => {
  if (version === undefined) {
    problems.push(`could not read a version from ${where}`);
    return;
  }
  found.push({ where, version });
};

// --- Cargo.toml: workspace version + the internal core pin ------------------
const cargo = readFileSync(join(root, "Cargo.toml"), "utf8");
record("Cargo.toml [workspace.package] version", cargo.match(/^version = "([^"]*)"$/m)?.[1]);
record(
  `Cargo.toml ${SLUG}-core dependency pin`,
  cargo.match(new RegExp(`${SLUG}-core = \\{ path = "crates/lib/${SLUG}-core", version = "([^"]*)" \\}`))?.[1],
);

// --- packages/*/package.json + the launcher's optionalDependencies ----------
for (const name of readdirSync(join(root, "packages"))) {
  const path = join(root, "packages", name, "package.json");
  let json;
  try {
    json = JSON.parse(readFileSync(path, "utf8"));
  } catch {
    continue;
  }
  record(`packages/${name}/package.json version`, json.version);
  for (const [dep, range] of Object.entries(json.optionalDependencies ?? {})) {
    if (dep.startsWith(`${SLUG}-`)) {
      record(`packages/${name}/package.json optionalDependencies["${dep}"]`, range);
    }
  }
}

// --- Verdict ----------------------------------------------------------------
const versions = [...new Set(found.map((f) => f.version))];
const target = expected ?? versions[0];

if (versions.length !== 1) {
  problems.push(`inconsistent versions across the tree: ${versions.join(", ")}`);
}
for (const { where, version } of found) {
  if (version !== target) {
    problems.push(`${where}: found ${version}, expected ${target}`);
  }
}

if (problems.length > 0) {
  console.error("version check failed:");
  for (const p of problems) console.error(`  - ${p}`);
  console.error("\nRun `npm run set-version <version>` to fix.");
  process.exit(1);
}

console.log(`version ${target} is consistent across ${found.length} locations`);
