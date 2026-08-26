#!/usr/bin/env node
// Cut a release commit and its two tags. Deliberately stops before pushing: the
// push is what triggers an irreversible publish, so it stays a human action.
//
// Usage: node scripts/release.mjs <version> [pack-version]
//
//   1. refuse a dirty working tree
//   2. refuse either tag if it exists
//   3. refuse a version with no CHANGELOG section
//   4. set the version everywhere (including lockfiles)
//   5. set the pack version and the pin that must name it
//   6. verify it landed consistently
//   7. commit, then tag the commit twice
//
// The binary and its content are cut from one commit so `DEFAULT_PACK.rev`
// always names a rev whose `/pack/` is what this binary embedded — there is no
// second step for a human to forget (ADR-008). The pack keeps a version series
// of its own; with none given it takes the next patch. A content release with
// no binary is `scripts/release-pack.mjs`.
//
// Checks 1 to 3 run before anything is written, so a failure leaves the tree
// untouched.

import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { readPackVersion, nextPatch, packTag, setPackVersion } from "./pack-version.mjs";

const version = process.argv[2]?.replace(/^v/, "");
const root = join(dirname(fileURLToPath(import.meta.url)), "..");

if (!version || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("usage: node scripts/release.mjs <semver> [pack-version]");
  process.exit(1);
}

const tag = `v${version}`;
const run = (cmd, args, opts = {}) =>
  execFileSync(cmd, args, { cwd: root, encoding: "utf8", ...opts });
const step = (msg) => console.log(`\n\u2192 ${msg}`);

const fail = (msg) => {
  console.error(`\nrelease aborted: ${msg}`);
  process.exit(1);
};

// --- 1. Clean working tree --------------------------------------------------
if (run("git", ["status", "--porcelain"]).trim() !== "") {
  fail("working tree is dirty; commit or stash first");
}

// --- 2. Neither tag may already exist ---------------------------------------
// The pack version is its own series, so it is read from the tree rather than
// derived from the binary version; with none given the patch moves on.
const packVersion = process.argv[3]?.replace(/^(assets-)?v/, "") ?? nextPatch(readPackVersion(root));
const packTagName = packTag(packVersion);
for (const existing of [tag, packTagName]) {
  if (run("git", ["tag", "--list", existing]).trim() !== "") {
    fail(`tag ${existing} already exists`);
  }
}

// --- 3. CHANGELOG must have a section for this version ----------------------
// This is the gate that makes "update the changelog" non-optional; the release
// workflow extracts the same section for the GitHub release notes.
const changelog = readFileSync(join(root, "CHANGELOG.md"), "utf8");
if (!new RegExp(`^## \\[${version.replace(/[.\\+]/g, "\\$&")}\\]`, "m").test(changelog)) {
  fail(
    `CHANGELOG.md has no "## [${version}]" section.\n` +
      "  Add one (promote [Unreleased] if that is where the notes are) and retry.",
  );
}

// --- 4. Set the version everywhere ------------------------------------------
step(`setting version to ${version}`);
run("node", [join(root, "scripts/set-version.mjs"), version], { stdio: "inherit" });

// --- 5. Set the pack version and the pin that must name it ------------------
step(`setting the pack version to ${packVersion}`);
setPackVersion(root, packVersion);
console.log(`  pack/pack.toml and DEFAULT_PACK.rev now name ${packTagName}`);

// --- 6. Verify it landed consistently ---------------------------------------
step("verifying version consistency");
run("node", [join(root, "scripts/verify-version.mjs"), version], { stdio: "inherit" });

// --- 7. Commit, then tag that one commit twice ------------------------------
step("committing and tagging");
run("git", ["add", "-A"], { stdio: "inherit" });
run("git", ["commit", "-m", `chore(release): ${tag}`], { stdio: "inherit" });
for (const cut of [tag, packTagName]) {
  run("git", ["tag", "-a", cut, "-m", cut], { stdio: "inherit" });
}

console.log(`
Prepared ${tag}, carrying the content release ${packTagName}.

  Review:  git show ${tag}
  Publish: git push --follow-tags

Pushing triggers the release workflow, which publishes to npm and crates.io.
Those publishes cannot be undone. ${packTagName} rides along on the same commit
and runs no workflow of its own; it is what \`superdev update\` moves a default
pin to.`);
