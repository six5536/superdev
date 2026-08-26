// The pack's version, and the two files that must agree about it.
//
// `/pack/pack.toml` is what the pack declares about itself; `DEFAULT_PACK.rev`
// in the binary names the tag that content was cut as. A binary whose pin names
// a rev holding different content is the mismatch ADR-008 exists to remove, so
// both are written by one call and a pattern that no longer matches is an
// error rather than a skip.
//
// The pack carries a version series of its own, apart from the binary's: a
// content release must not consume a binary version number.

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/** What a pack release tag is called — `PACK_TAG_PREFIX` in the Rust source. */
export const TAG_PREFIX = "assets-v";

const MANIFEST = "pack/pack.toml";
const SOURCE = "crates/lib/superdev-core/src/pack/source.rs";

/** The `version` line in `pack.toml`, whose value is aligned with its siblings. */
const MANIFEST_VERSION = /^(version\s*=\s*")([^"]*)(")/m;

/** `DEFAULT_PACK`'s `rev`, and not the `PACK_TAG_PREFIX` constant below it. */
const DEFAULT_PACK_REV = /(const DEFAULT_PACK: DefaultPack = DefaultPack \{[^}]*?rev: ")([^"]*)(")/;

/** The tag a pack version is released as. */
export function packTag(version) {
  return `${TAG_PREFIX}${version}`;
}

/** The version after this one, bumping the patch — the default content release. */
export function nextPatch(version) {
  const [major, minor, patch] = parse(version);
  return `${major}.${minor}.${patch + 1}`;
}

/** The version `/pack/pack.toml` declares. */
export function readPackVersion(root) {
  const manifest = readFileSync(join(root, MANIFEST), "utf8");
  const found = manifest.match(MANIFEST_VERSION)?.[2];
  if (found === undefined) {
    throw new Error(`could not find a version in ${MANIFEST}`);
  }
  return found;
}

/**
 * Set the pack's version in the manifest and the binary's pin, together.
 *
 * Returns the tag the version is released as, which is what the caller tags.
 */
export function setPackVersion(root, version) {
  parse(version);

  const manifestPath = join(root, MANIFEST);
  const manifest = readFileSync(manifestPath, "utf8");
  if (!MANIFEST_VERSION.test(manifest)) {
    throw new Error(`could not find a version in ${MANIFEST}`);
  }

  const sourcePath = join(root, SOURCE);
  const source = readFileSync(sourcePath, "utf8");
  if (!DEFAULT_PACK_REV.test(source)) {
    throw new Error(`could not find DEFAULT_PACK's rev in ${SOURCE}`);
  }

  // Both reads and both checks first: a half-written pair is the state this
  // module exists to prevent, so nothing is written until both can be.
  writeFileSync(manifestPath, manifest.replace(MANIFEST_VERSION, `$1${version}$3`));
  writeFileSync(sourcePath, source.replace(DEFAULT_PACK_REV, `$1${packTag(version)}$3`));
  return packTag(version);
}

/** The three numbers a pack version is, refusing anything else. */
function parse(version) {
  const parts = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!parts) {
    throw new Error(`pack version must be MAJOR.MINOR.PATCH, not \`${version}\``);
  }
  return parts.slice(1, 4).map(Number);
}
