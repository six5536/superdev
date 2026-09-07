#!/usr/bin/env node

import { access, readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const mapPath = resolve(root, "knowledge/documentation.md");
const map = await readFile(mapPath, "utf8");

const requiredSurfaces = [
	"readme",
	"contributor-guide",
	"canonical-knowledge",
	"cli-reference",
	"changelog",
	"documentation-site",
];
for (const surface of requiredSurfaces) {
	if (!map.includes(`### Surface: ${surface}\n`)) {
		throw new Error(`documentation map is missing surface ${surface}`);
	}
}

for (const path of ["README.md", "CONTRIBUTING.md", "CHANGELOG.md", "knowledge/development-commands.md"]) {
	await access(resolve(root, path));
	if (!map.includes(`/${path}`)) throw new Error(`documentation map does not declare /${path}`);
}

const blocks = map.split(/^### Surface: /m).slice(1);
const fields = [
	"Audience and purpose",
	"Kind",
	"Authored sources",
	"Generated outputs",
	"Source of truth",
	"Triggers",
	"Generation command",
	"Verification command",
	"Generated output",
	"Publication owner",
];
for (const block of blocks) {
	const name = block.split("\n", 1)[0];
	for (const field of fields) {
		if (!block.includes(`- ${field}: `)) throw new Error(`${name} is missing ${field}`);
	}
}

console.log(`documentation: ${blocks.length} surface(s) declared and checked`);
