#!/usr/bin/env node
// Cut a release commit and its two tags. Deliberately stops before pushing: the
// push is what triggers an irreversible publish, so it stays a human action.
//
// Usage: node scripts/release.mjs <version> [pack-version]
//
//   1. refuse a dirty working tree
//   2. work out the pack version, refusing one that is malformed or backwards
//   3. refuse either tag if it exists
//   4. refuse a version with no CHANGELOG section
//   5. set the version everywhere (including lockfiles)
//   6. set the pack version and the pin that must name it
//   7. verify it landed consistently
//   8. commit, then tag the commit twice
//
// The binary and its content are cut from one commit so `DEFAULT_PACK.rev`
// always names a rev whose `/pack/` is what this binary embedded — there is no
// second step for a human to forget (ADR-008). The pack keeps a version series
// of its own; with none given it takes the version `pack.toml` declares while
// that is unreleased, and the next patch once it is out. A candidate binary
// cuts a candidate content tag, which `update` does not move stable pins to. A
// content release with no binary is `scripts/release-pack.mjs`.
//
// Checks 1 to 4 run before anything is written, so a failure leaves the tree
// untouched.

import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import {
  readPackVersion,
  packTag,
  setPackVersion,
  plannedPackVersion,
  prereleaseOf,
  refusalFor,
  coreOf,
  TAG_PREFIX,
} from "./pack-version.mjs";

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

// --- 2. Work out the pack version, and refuse a bad one before writing ------
// The pack has a version series of its own, so it comes from the tree rather
// than from the binary version. Everything here is settled before step 5
// writes anything: an argument this rejects must not leave a half-set tree.
fetchTags();
const declared = readPackVersion(root);
// Has the version the tree declares been released? If not, that is the one to
// cut, so the first release cuts the `pack.toml` the repo already carries.
const coreReleased = run("git", ["tag", "--list", `${TAG_PREFIX}${coreOf(declared)}`]).trim() !== "";
const packVersion = choosePackVersion();
const packTagName = packTag(packVersion);

function choosePackVersion() {
  const given = process.argv[3]?.replace(/^(assets-)?v/, "");
  const prerelease = prereleaseOf(version);
  if (given === undefined || given === "") {
    return plannedPackVersion({ declared, coreReleased, prerelease });
  }
  try {
    packTag(given);
  } catch (e) {
    fail(e.message);
  }
  const refusal = refusalFor({ candidate: given, declared, coreReleased });
  if (refusal) {
    fail(refusal);
  }
  // A candidate binary must not cut a release-numbered content tag: `update`
  // moves stable pins onto anything spelled as a three-number release. Nor the
  // converse — a release whose pin names a candidate tag gives every repo it
  // sets up a pin that only comes forward once some later release covers it.
  if (prerelease && !prereleaseOf(given)) {
    fail(`${tag} is a prerelease, so its content tag must be one too — try ${given}-${prerelease}`);
  }
  if (!prerelease && prereleaseOf(given)) {
    fail(`${tag} is a release, so its content tag must be one too — not ${packTag(given)}`);
  }
  return given;
}

// Which versions are already out is a question about tags, and a clone can be
// missing them — `--no-tags`, or simply not fetched since a colleague cut one.
// Deciding from a stale view re-cuts a version that exists, which only shows
// up as a rejected push once the commit and both tags are made. Best effort:
// a release without a reachable remote is still a release worth preparing.
function fetchTags() {
  try {
    run("git", ["fetch", "--tags", "--quiet"]);
  } catch {
    console.warn("warning: could not fetch tags; deciding from this clone's own");
  }
}

// --- 3. Neither tag may already exist ---------------------------------------
for (const existing of [tag, packTagName]) {
  if (run("git", ["tag", "--list", existing]).trim() !== "") {
    fail(`tag ${existing} already exists`);
  }
}

// --- 4. CHANGELOG must have a section for this version ----------------------
// This is the gate that makes "update the changelog" non-optional; the release
// workflow extracts the same section for the GitHub release notes.
const changelog = readFileSync(join(root, "CHANGELOG.md"), "utf8");
if (!new RegExp(`^## \\[${version.replace(/[.\\+]/g, "\\$&")}\\]`, "m").test(changelog)) {
  fail(
    `CHANGELOG.md has no "## [${version}]" section.\n` +
      "  Add one (promote [Unreleased] if that is where the notes are) and retry.",
  );
}

// --- 5. Set the version everywhere ------------------------------------------
step(`setting version to ${version}`);
run("node", [join(root, "scripts/set-version.mjs"), version], { stdio: "inherit" });

// --- 6. Set the pack version and the pin that must name it ------------------
step(`setting the pack version to ${packVersion}`);
setPackVersion(root, packVersion);
console.log(`  pack/pack.toml and DEFAULT_PACK.rev now name ${packTagName}`);

// --- 7. Verify it landed consistently ---------------------------------------
step("verifying version consistency");
run("node", [join(root, "scripts/verify-version.mjs"), version], { stdio: "inherit" });

// --- 8. Commit, then tag that one commit twice ------------------------------
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
