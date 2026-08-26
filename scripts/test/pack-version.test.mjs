// The pack version lives in two files that must agree: `/pack/pack.toml`, which
// the pack itself declares, and `DEFAULT_PACK.rev` in the binary, which names
// the tag that content was cut as. A release sets both or neither — a binary
// whose pin names a rev holding different content is the failure this module
// exists to make unreachable.

import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";

import {
  readPackVersion,
  nextPatch,
  packTag,
  setPackVersion,
  plannedPackVersion,
  prereleaseOf,
  isBehind,
  refusalFor,
} from "../pack-version.mjs";

const PACK_TOML = `# a comment the rewrite must not disturb
format      = 1
name        = "superdev-assets"
version     = "0.4.2"
description = "superdev's stock skills, templates and scaffolds"
`;

const SOURCE_RS = `//! source.rs

pub const DEFAULT_PACK: DefaultPack = DefaultPack {
    source: "github:six5536/superdev",
    rev: "assets-v0.4.2",
};

pub const PACK_TAG_PREFIX: &str = "assets-v";
`;

/// A tree carrying the two files a pack release rewrites, and nothing else.
function fixture({ packToml = PACK_TOML, sourceRs = SOURCE_RS } = {}) {
  const root = mkdtempSync(join(tmpdir(), "packver-"));
  const source = join(root, "crates/lib/superdev-core/src/pack/source.rs");
  mkdirSync(join(root, "pack"), { recursive: true });
  mkdirSync(dirname(source), { recursive: true });
  writeFileSync(join(root, "pack/pack.toml"), packToml);
  writeFileSync(source, sourceRs);
  return root;
}

const cleanup = (root) => rmSync(root, { recursive: true, force: true });

test("reads the version the pack declares", () => {
  const root = fixture();
  assert.equal(readPackVersion(root), "0.4.2");
  cleanup(root);
});

test("the next patch is the default bump, and the tag is the version prefixed", () => {
  assert.equal(nextPatch("0.4.2"), "0.4.3");
  assert.equal(nextPatch("0.9.0"), "0.9.1");
  // Ten follows nine: the tag series is ordered by the numbers, not the string.
  assert.equal(nextPatch("0.10.9"), "0.10.10");
  assert.equal(packTag("0.4.3"), "assets-v0.4.3");
});

test("setting the version moves the manifest and the pin together", () => {
  const root = fixture();

  setPackVersion(root, "1.0.0");

  assert.equal(readPackVersion(root), "1.0.0");
  const source = readFileSync(
    join(root, "crates/lib/superdev-core/src/pack/source.rs"),
    "utf8",
  );
  assert.match(source, /rev: "assets-v1\.0\.0"/);
  // The prefix constant is a different `assets-v` and must be left alone.
  assert.match(source, /PACK_TAG_PREFIX: &str = "assets-v"/);
  cleanup(root);
});

test("nothing else in either file moves", () => {
  const root = fixture();

  setPackVersion(root, "1.0.0");

  const toml = readFileSync(join(root, "pack/pack.toml"), "utf8");
  assert.equal(toml, PACK_TOML.replace('"0.4.2"', '"1.0.0"'));
  const source = readFileSync(
    join(root, "crates/lib/superdev-core/src/pack/source.rs"),
    "utf8",
  );
  assert.equal(source, SOURCE_RS.replace('"assets-v0.4.2"', '"assets-v1.0.0"'));
  cleanup(root);
});

test("setting the version twice is setting it once", () => {
  const root = fixture();

  setPackVersion(root, "1.0.0");
  const once = readFileSync(join(root, "pack/pack.toml"), "utf8");
  setPackVersion(root, "1.0.0");

  assert.equal(readFileSync(join(root, "pack/pack.toml"), "utf8"), once);
  cleanup(root);
});

// A rename or a reformat that this module's patterns no longer match must stop
// the release, not sail past it leaving the pin naming the previous content.
test("a pin the pattern cannot find is an error, not a silent skip", () => {
  const root = fixture({ sourceRs: "pub const SOMETHING_ELSE: u8 = 1;\n" });

  assert.throws(() => setPackVersion(root, "1.0.0"), /DEFAULT_PACK/);
  cleanup(root);
});

test("a manifest with no version is an error", () => {
  const root = fixture({ packToml: 'format = 1\nname = "x"\n' });

  assert.throws(() => setPackVersion(root, "1.0.0"), /pack\.toml/);
  cleanup(root);
});

test("a version that is not three numbers is refused", () => {
  const root = fixture();

  assert.throws(() => setPackVersion(root, "1.0"), /1\.0/);
  assert.throws(() => setPackVersion(root, "v1.0.0"), /v1\.0\.0/);
  cleanup(root);
});

// --- Choosing the version a release cuts ------------------------------------

test("a version the tree declares but has never released is what gets cut", () => {
  // pack.toml says 0.1.0 and no such tag exists: the first release cuts the
  // version the tree already claims rather than skipping past it, which is
  // also how a hand-edited pack.toml gets the version its author chose.
  assert.equal(
    plannedPackVersion({ declared: "0.1.0", coreReleased: false }),
    "0.1.0",
  );
});

test("once that version is released the patch moves on", () => {
  assert.equal(
    plannedPackVersion({ declared: "0.1.0", coreReleased: true }),
    "0.1.1",
  );
});

test("a prerelease binary cuts a prerelease content tag", () => {
  // `update` takes only three-number releases, so a candidate's content must
  // not be spelled as one: an rc that cut `assets-v0.1.1` would move every
  // stable user's pin onto release-candidate content.
  assert.equal(
    plannedPackVersion({ declared: "0.1.0", coreReleased: false, prerelease: "rc.1" }),
    "0.1.0-rc.1",
  );
  assert.equal(
    plannedPackVersion({ declared: "0.1.0", coreReleased: true, prerelease: "rc.1" }),
    "0.1.1-rc.1",
  );
});

test("a second candidate moves the suffix, not the version", () => {
  assert.equal(
    plannedPackVersion({ declared: "0.1.0-rc.1", coreReleased: false, prerelease: "rc.2" }),
    "0.1.0-rc.2",
  );
});

test("the release after a candidate drops the suffix and keeps the version", () => {
  // 0.1.0-rc.1 is tagged but 0.1.0 is not, so the stable release is 0.1.0.
  assert.equal(
    plannedPackVersion({ declared: "0.1.0-rc.1", coreReleased: false }),
    "0.1.0",
  );
});

test("the prerelease a binary version carries, if any", () => {
  assert.equal(prereleaseOf("0.3.0"), "");
  assert.equal(prereleaseOf("0.3.0-rc.1"), "rc.1");
  assert.equal(prereleaseOf("1.0.0-alpha.2"), "alpha.2");
});

// --- Refusing to go backwards -----------------------------------------------

test("a version below the one the tree declares is behind it", () => {
  assert.ok(isBehind("0.0.5", "0.1.0"));
  assert.ok(isBehind("0.1.0", "0.10.0"));
  assert.ok(!isBehind("0.1.0", "0.1.0"));
  assert.ok(!isBehind("0.2.0", "0.1.0"));
  // A candidate and its release share a version, so neither is behind the
  // other: cutting 0.1.0 with 0.1.0-rc.1 declared is the normal path.
  assert.ok(!isBehind("0.1.0", "0.1.0-rc.1"));
});

// --- Prereleases through the writing path -----------------------------------

test("a prerelease version is written to both files", () => {
  const root = fixture();

  const tag = setPackVersion(root, "1.0.0-rc.1");

  assert.equal(tag, "assets-v1.0.0-rc.1");
  assert.equal(readPackVersion(root), "1.0.0-rc.1");
  assert.match(
    readFileSync(join(root, "crates/lib/superdev-core/src/pack/source.rs"), "utf8"),
    /rev: "assets-v1\.0\.0-rc\.1"/,
  );
  cleanup(root);
});

test("the tag helper refuses what the release cannot cut", () => {
  assert.throws(() => packTag("1.2"), /1\.2/);
  assert.throws(() => packTag(""), /MAJOR/);
  assert.throws(() => packTag("v1.2.3"), /v1\.2\.3/);
});

// --- What a release refuses -------------------------------------------------

test("a version behind what the tree declares is refused", () => {
  assert.match(
    refusalFor({ candidate: "0.0.5", declared: "0.1.0", coreReleased: false }),
    /behind pack\.toml's 0\.1\.0/,
  );
});

test("a candidate whose release is already out is refused", () => {
  // `isBehind` compares the numbers alone, so this one gets past it: 0.2.0-rc.1
  // and 0.2.0 share a version. What makes it backwards is 0.2.0 being cut.
  assert.match(
    refusalFor({ candidate: "0.2.0-rc.1", declared: "0.2.0", coreReleased: true }),
    /already released/,
  );
});

test("a candidate for a version not yet out is the normal path", () => {
  assert.equal(
    refusalFor({ candidate: "0.1.0-rc.1", declared: "0.1.0", coreReleased: false }),
    "",
  );
  assert.equal(
    refusalFor({ candidate: "0.2.0", declared: "0.1.0", coreReleased: true }),
    "",
  );
});
