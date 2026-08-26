// The binary release workflow must not run for a content release. Nothing in
// the workflow says `assets-v` — it is the `v*` filter not matching that keeps
// content tags out, which is easy to undo by widening the filter and hard to
// notice until a content release spends twenty minutes cross-building five
// binaries and publishing them. This holds the two apart.

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { packTag } from "../pack-version.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");

/** The tag patterns `release.yml` triggers on. */
function tagFilters() {
  const workflow = readFileSync(join(root, ".github/workflows/release.yml"), "utf8");
  const line = workflow.match(/^\s*tags:\s*\[(.*)\]\s*$/m);
  assert.ok(line, "release.yml must trigger on a tags filter");
  return line[1].split(",").map((p) => p.trim().replace(/^["']|["']$/g, ""));
}

/** GitHub's tag globs: `*` is any run of characters bar a slash. */
function matches(pattern, ref) {
  const source = pattern
    .split("*")
    .map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
    .join("[^/]*");
  return new RegExp(`^${source}$`).test(ref);
}

test("the glob translation is right about what it is asserting", () => {
  assert.ok(matches("v*", "v1.2.3"));
  assert.ok(matches("v*", "v1.2.3-rc.1"));
  assert.ok(!matches("v*", "assets-v1.2.3"));
  assert.ok(!matches("v*", "v1/2"));
});

test("a binary tag triggers the release workflow", () => {
  const filters = tagFilters();
  for (const tag of ["v1.2.3", "v0.2.0-rc.1"]) {
    assert.ok(
      filters.some((f) => matches(f, tag)),
      `${tag} must match one of ${filters.join(", ")}`,
    );
  }
});

test("a content tag triggers nothing", () => {
  const filters = tagFilters();
  for (const version of ["1.2.3", "0.1.0", "10.0.0"]) {
    const tag = packTag(version);
    assert.ok(
      !filters.some((f) => matches(f, tag)),
      `${tag} must match none of ${filters.join(", ")} — a content release runs no build`,
    );
  }
});
