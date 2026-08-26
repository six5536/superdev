#!/usr/bin/env node
// Cut a content release: the pack alone, with no binary build.
//
// Usage: node scripts/release-pack.mjs [pack-version]
//
// With no version it bumps the patch. Like the binary release it stops before
// pushing, because the push is what publishes.
//
//   1. refuse a dirty working tree
//   2. refuse a tag that exists
//   3. refuse a version with no CHANGELOG section
//   4. set pack.toml's version and DEFAULT_PACK.rev together
//   5. commit and tag
//
// Checks 1 to 3 run before anything is written, so a failure leaves the tree
// untouched.
//
// The tag is `assets-vA.B.C`, which the release workflow's `v*` filter does not
// match, so no five-platform build runs — the point of the feature (ADR-008).

import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import {
  readPackVersion,
  nextPatch,
  packTag,
  setPackVersion,
  TAG_PREFIX,
} from "./pack-version.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const run = (cmd, args, opts = {}) =>
  execFileSync(cmd, args, { cwd: root, encoding: "utf8", ...opts });
const step = (msg) => console.log(`\n\u2192 ${msg}`);
const fail = (msg) => {
  console.error(`\npack release aborted: ${msg}`);
  process.exit(1);
};

const version = process.argv[2]?.replace(/^(assets-)?v/, "") ?? nextPatch(readPackVersion(root));
const tag = packTag(version);

// --- 1. Clean working tree --------------------------------------------------
if (run("git", ["status", "--porcelain"]).trim() !== "") {
  fail("working tree is dirty; commit or stash first");
}

// --- 2. Tag must not already exist -----------------------------------------
if (run("git", ["tag", "--list", tag]).trim() !== "") {
  fail(`tag ${tag} already exists`);
}

// --- 3. CHANGELOG must have a section for this content release --------------
// A content release has no binary release to be described by, so it carries
// its own section. The binary release writes no second section: the pack tag
// it cuts comes off the same commit its own section already describes.
const changelog = readFileSync(join(root, "CHANGELOG.md"), "utf8");
if (!new RegExp(`^## \\[${tag.replace(/[.\\+]/g, "\\$&")}\\]`, "m").test(changelog)) {
  fail(
    `CHANGELOG.md has no "## [${tag}]" section.\n` +
      "  Add one saying what the content change is and retry.",
  );
}

// --- 4. Set the pack version and the pin it must agree with -----------------
step(`setting the pack version to ${version}`);
setPackVersion(root, version);
console.log(`  pack/pack.toml and DEFAULT_PACK.rev now name ${tag}`);

// --- 5. Commit and tag ------------------------------------------------------
step("committing and tagging");
run("git", ["add", "-A"], { stdio: "inherit" });
run("git", ["commit", "-m", `chore(release): ${tag}`], { stdio: "inherit" });
run("git", ["tag", "-a", tag, "-m", tag], { stdio: "inherit" });

console.log(`
Prepared ${tag}.

  Review:  git show ${tag}
  Publish: git push --follow-tags

No workflow runs for a content tag and nothing is published to a registry.
Pushing it is what lets \`superdev update\` find the release, which it does by
asking this repository for its newest \`${TAG_PREFIX}\` tag.`);
