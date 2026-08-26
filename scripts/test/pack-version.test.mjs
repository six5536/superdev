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

import { readPackVersion, nextPatch, packTag, setPackVersion } from "../pack-version.mjs";

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
