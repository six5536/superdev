// Paired evidence for SOKF-routed reads: every case runs twice, once through
// Pi 0.85.1's built-in read against the physical file and once through the
// routed tool against the equivalent SOKF argument, and the two results must
// agree exactly. The pinned test dependency is imported directly, so this
// harness fails rather than skips when Pi cannot load.

import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
	createReadTool,
	createReadToolDefinition,
	DEFAULT_MAX_BYTES,
	DEFAULT_MAX_LINES,
} from "@earendil-works/pi-coding-agent";

import sokf, { findRepository, prepareReadPath } from "../../../.pi/extensions/sokf.ts";

const here = dirname(fileURLToPath(import.meta.url));
const repository = resolve(here, "../../..");

const MANIFEST = 'sokf: "0.1"\nname: paired-fixture\n';
const CONCEPT = `---
type: Reference
id: paired-a
title: Paired A
description: The concept both halves of the harness read.
---

# Body

First line of the body.
Second line of the body.
`;

/** What one execution produced: a result, or the error it threw. */
type Outcome = { ok: true; value: any } | { ok: false; error: Error };

async function capture(run: () => Promise<any>): Promise<Outcome> {
	try {
		return { ok: true, value: await run() };
	} catch (error) {
		return { ok: false, error: error as Error };
	}
}

/**
 * `value` with `target` rewritten to `spelling` in every string it carries.
 *
 * The walk is structural rather than a JSON round-trip: Pi sets `details`
 * to an explicit `undefined` on an untruncated read, and serializing would
 * drop that key and hide a real difference in result shape.
 */
function rewritePaths(value: unknown, target: string, spelling: string): unknown {
	if (typeof value === "string") return value.split(target).join(spelling);
	if (Array.isArray(value)) return value.map((item) => rewritePaths(item, target, spelling));
	if (value === null || typeof value !== "object") return value;
	const out: Record<string, unknown> = {};
	// `Object.keys` keeps a key whose value is `undefined`, which is exactly
	// the distinction being preserved.
	for (const key of Object.keys(value)) {
		out[key] = rewritePaths((value as Record<string, unknown>)[key], target, spelling);
	}
	return out;
}

/**
 * The two outcomes are the same, once the routed half's virtual argument is
 * rewritten to the physical argument the built-in half was given.
 *
 * Both halves are handed the same physical spelling wherever one exists, so
 * this rewrite is a no-op except for a `sokf:<id>` argument — the one
 * unavoidable virtual-versus-physical difference. It is applied to the
 * routed half alone, so a real divergence still fails.
 */
function assertSame(builtin: Outcome, routed: Outcome, asked: string, physical: string, what: string) {
	const normalize = (value: unknown) => (asked === physical ? value : rewritePaths(value, asked, physical));
	if (builtin.ok !== routed.ok) {
		const detail = builtin.ok ? routed : builtin;
		throw new Error(`${what}: one half ${builtin.ok ? "threw" : "succeeded"}: ${(detail as any).error ?? ""}`);
	}
	if (!builtin.ok && !routed.ok) {
		assert.equal(normalize(routed.error.message), builtin.error.message, `${what}: error text`);
		assert.equal(routed.error.constructor.name, builtin.error.constructor.name, `${what}: error kind`);
		return;
	}
	assert.deepEqual(normalize((routed as any).value), (builtin as any).value, `${what}: result`);
}

function sandbox(): { root: string; knowledge: string } {
	const root = mkdtempSync(join(tmpdir(), "sokf-pi-paired-"));
	mkdirSync(join(root, ".git"));
	const knowledge = join(root, "knowledge");
	mkdirSync(join(knowledge, "notes"), { recursive: true });
	writeFileSync(join(knowledge, "manifest.sokf.yaml"), MANIFEST);
	writeFileSync(join(knowledge, "a.md"), CONCEPT);
	return { root, knowledge };
}

export default async function paired() {
	const tools = new Map<string, any>();
	const handlers = new Map<string, any>();
	sokf({
		registerTool(tool: any) {
			tools.set(tool.name, tool);
		},
		on(name: string, handler: any) {
			handlers.set(name, handler);
		},
		sendMessage() {},
	} as any);
	const read = tools.get("read");
	assert.ok(read, "the adapter registered no read tool");

	// The routed registration advertises only what it accepts: no overview
	// address and no section-qualified address.
	assert.ok(!read.description.includes("sokf:<id>#"), "read still advertises section addresses");
	assert.ok(!/sokf: for an overview/.test(read.description), "read still advertises the overview address");

	// Pi owns presentation: the renderers are its own, not a copy.
	const pi = createReadToolDefinition(repository);
	assert.equal(read.renderCall, pi.renderCall, "the routed read replaced Pi's call renderer");
	assert.equal(read.renderResult, pi.renderResult, "the routed read replaced Pi's result renderer");
	assert.deepEqual(read.parameters, pi.parameters, "the routed read replaced Pi's schema");

	const { root, knowledge } = sandbox();
	try {
		const ctx = { cwd: root, ui: { notify() {} } };
		const nested = { cwd: join(knowledge, "notes"), ui: { notify() {} } };
		const physical = join(knowledge, "a.md");

		// Bulk fixtures for the truncation boundaries Pi owns.
		const many = `${Array.from({ length: DEFAULT_MAX_LINES + 500 }, (_, i) => `line ${i + 1}`).join("\n")}\n`;
		writeFileSync(join(knowledge, "many.md"), many);
		const heavy = `${Array.from({ length: 400 }, (_, i) => `${i} ${"x".repeat(200)}`).join("\n")}\n`;
		writeFileSync(join(knowledge, "heavy.md"), heavy);
		// The oversized line is quoted back in a notice that names the path, so
		// this file is read by identity as well as by path.
		writeFileSync(
			join(knowledge, "wide.md"),
			`---\ntype: Reference\nid: paired-wide\n---\n${"y".repeat(DEFAULT_MAX_BYTES + 1000)}\nsecond\n`,
		);

		// Each case names the routed argument and the physical argument that
		// addresses the same bytes, so the two halves differ only where a
		// virtual identity has no physical spelling.
		const builtin = createReadTool(root);
		const cases: Array<{ what: string; argument: string; physical: string; params?: any; ctx?: any }> = [
			{ what: "whole file by identity", argument: "sokf:paired-a", physical: "knowledge/a.md" },
			{ what: "whole file by physical path", argument: "knowledge/a.md", physical: "knowledge/a.md" },
			{ what: "one leading @", argument: "@knowledge/a.md", physical: "@knowledge/a.md" },
			{ what: "absolute physical path", argument: join(knowledge, "a.md"), physical: join(knowledge, "a.md") },
			{ what: "offset and limit", argument: "sokf:paired-a", physical: "knowledge/a.md", params: { offset: 2, limit: 3 } },
			{ what: "offset alone", argument: "sokf:paired-a", physical: "knowledge/a.md", params: { offset: 5 } },
			{ what: "limit past the end", argument: "sokf:paired-a", physical: "knowledge/a.md", params: { limit: 999 } },
			{ what: "offset out of range", argument: "sokf:paired-a", physical: "knowledge/a.md", params: { offset: 9000 } },
			{ what: "missing file", argument: "knowledge/notes/gone.md", physical: "knowledge/notes/gone.md" },
			{ what: "line truncation", argument: "knowledge/many.md", physical: "knowledge/many.md" },
			{ what: "line truncation with offset", argument: "knowledge/many.md", physical: "knowledge/many.md", params: { offset: 3 } },
			{ what: "byte truncation", argument: "knowledge/heavy.md", physical: "knowledge/heavy.md" },
			{ what: "one oversized line", argument: "knowledge/wide.md", physical: "knowledge/wide.md" },
			{ what: "oversized line by identity", argument: "sokf:paired-wide", physical: "knowledge/wide.md" },
			// Landing the window on the oversized line is what produces Pi's
			// first-line-exceeds-limit notice — the one model-visible read
			// notice that quotes the path it was given. Without an offset the
			// frontmatter leads and Pi truncates by bytes instead, so these two
			// rows are the only proof that routed spelling is restored.
			{ what: "oversized first line", argument: "knowledge/wide.md", physical: "knowledge/wide.md", params: { offset: 5 } },
			{
				what: "oversized first line by identity",
				argument: "sokf:paired-wide",
				physical: "knowledge/wide.md",
				params: { offset: 5 },
			},
			{ what: "nested working directory", argument: "sokf:paired-a", physical: "../a.md", ctx: nested },
			{ what: "nested relative path", argument: "../a.md", physical: "../a.md", ctx: nested },
		];

		for (const testCase of cases) {
			const where = testCase.ctx ?? ctx;
			const params = { ...(testCase.params ?? {}) };
			const before = await capture(() =>
				builtin.execute("builtin", { ...params, path: testCase.physical }, undefined, undefined, where),
			);
			const after = await capture(() =>
				read.execute("routed", { ...params, path: testCase.argument }, undefined, undefined, where),
			);
			assertSame(before, after, testCase.argument, testCase.physical, testCase.what);
		}

		// Cancellation is refused before any target is opened, in Pi's words.
		const aborted = AbortSignal.abort();
		const cancelledBuiltin = await capture(() =>
			builtin.execute("builtin", { path: "knowledge/a.md" }, aborted, undefined, ctx),
		);
		const cancelledRouted = await capture(() =>
			read.execute("routed", { path: "sokf:paired-a" }, aborted, undefined, ctx),
		);
		assertSame(cancelledBuiltin, cancelledRouted, "sokf:paired-a", "knowledge/a.md", "cancellation");

		// A copied routed excerpt is the source, byte for byte, so it works
		// unchanged as an exact-replacement anchor. A whole read is the file;
		// a limited read is its first lines, once Pi's own continuation notice
		// — which is presentation, not content — is set aside.
		const source = readFileSync(physical, "utf8");
		const whole = await read.execute("routed", { path: "sokf:paired-a" }, undefined, undefined, ctx);
		assert.equal(whole.content[0].text, source, "a whole routed read is not the source bytes");

		const excerpt = await read.execute("routed", { path: "sokf:paired-a", limit: 6 }, undefined, undefined, ctx);
		const notice = /\n\n\[\d+ more lines in file\. Use offset=\d+ to continue\.\]$/;
		assert.match(excerpt.content[0].text, notice, "the limited read carried no continuation notice");
		const copied = excerpt.content[0].text.replace(notice, "");
		assert.ok(copied.startsWith("---\ntype: Reference"), "the routed excerpt dropped frontmatter");
		assert.equal(copied, source.split("\n").slice(0, 6).join("\n"), "the routed excerpt is not the source range");
		// The anchor is usable as written: it occurs exactly once in the file.
		assert.equal(source.split(copied).length - 1, 1, "the routed excerpt is not a unique anchor");

		// Rendered knowledge is refused, and the message names the tool that
		// serves it.
		for (const refused of ["sokf:", "sokf:paired-a#Body"]) {
			const outcome = await capture(() =>
				read.execute("routed", { path: refused }, undefined, undefined, ctx),
			);
			assert.equal(outcome.ok, false, `${refused} was not refused`);
			assert.match((outcome as any).error.message, /sokf_search/, `${refused} named no alternative`);
		}

		// Path preparation is proven against the value Pi hands its own read
		// operation, not against a reading of Pi's source.
		const variants = [
			"knowledge/a.md",
			"@knowledge/a.md",
			`${root}/knowledge/a.md`,
			"knowledge/a\u00A0b.md",
			"knowledge/Shot 3.15 PM.md",
			"knowledge/e\u0301cran.md",
			"knowledge/d'ete.md",
			"knowledge/d'e\u0301te\u0301 3.15 PM.md",
			`~${"/"}nonexistent-sokf-fixture.md`,
		];
		for (const variant of variants) {
			let observed: string | undefined;
			const probe = createReadTool(root, {
				operations: {
					access: async (path: string) => {
						observed = path;
						throw new Error("stop");
					},
					readFile: async () => Buffer.alloc(0),
					detectImageMimeType: async () => undefined,
				},
			});
			await capture(() => probe.execute("probe", { path: variant }, undefined, undefined, ctx));
			assert.equal(await prepareReadPath(variant, root), observed, `path preparation for ${variant}`);
		}

		// A contained symlink reads its target, including one outside
		// `knowledge/`; an escaping one is refused before access.
		mkdirSync(join(root, "vendor"));
		writeFileSync(join(root, "vendor/shared.md"), "shared body\n");
		symlinkSync(join(root, "vendor/shared.md"), join(knowledge, "linked.md"));
		const linked = await read.execute("routed", { path: "knowledge/linked.md" }, undefined, undefined, ctx);
		assert.equal(linked.content[0].text, "shared body\n", "a contained symlink did not read its target");

		// The escape target is a second checkout, so this is also the
		// cross-checkout refusal: a routed path may not reach another
		// repository's knowledge, and is refused before the file is opened.
		const outside = mkdtempSync(join(tmpdir(), "sokf-pi-escape-"));
		mkdirSync(join(outside, ".git"));
		mkdirSync(join(outside, "knowledge"));
		writeFileSync(join(outside, "knowledge/secret.md"), "secret\n");
		symlinkSync(join(outside, "knowledge/secret.md"), join(knowledge, "escaping.md"));
		const escaped = await capture(() =>
			read.execute("routed", { path: "knowledge/escaping.md" }, undefined, undefined, ctx),
		);
		assert.equal(escaped.ok, false, "a routed path reached another checkout");
		assert.equal(readFileSync(join(outside, "knowledge/secret.md"), "utf8"), "secret\n");
		rmSync(outside, { recursive: true, force: true });

		// An ordinary path outside the knowledge tree stays Pi's alone.
		writeFileSync(join(root, "plain.md"), "plain\n");
		const plain = await read.execute("routed", { path: "plain.md" }, undefined, undefined, ctx);
		assert.equal(plain.content[0].text, "plain\n", "an ordinary read was not delegated unchanged");

		await handlers.get("session_shutdown")?.({}, ctx);
	} finally {
		rmSync(root, { recursive: true, force: true });
	}

	// Discovery ranks its markers: any `.git` on the walk wins, and the
	// configuration marker still selects a managed non-Git root.
	const ranked = mkdtempSync(join(tmpdir(), "sokf-pi-roots-"));
	try {
		const configured = join(ranked, "configured");
		mkdirSync(join(configured, ".superdev"), { recursive: true });
		writeFileSync(join(configured, ".superdev/config.toml"), "");
		mkdirSync(join(configured, "nested/deeper"), { recursive: true });
		assert.equal(findRepository(join(configured, "nested/deeper")), configured, "the config marker lost its root");

		// A `.git` below the configuration marker still wins.
		const git = join(configured, "nested");
		mkdirSync(join(git, ".git"));
		assert.equal(findRepository(join(configured, "nested/deeper")), git, "a nearer .git did not win");

		// And a `.git` above it wins too, so precedence is by marker and not
		// by depth.
		const above = join(ranked, "above");
		mkdirSync(join(above, ".git"), { recursive: true });
		const belowConfig = join(above, "inner");
		mkdirSync(join(belowConfig, ".superdev"), { recursive: true });
		writeFileSync(join(belowConfig, ".superdev/config.toml"), "");
		mkdirSync(join(belowConfig, "leaf"));
		assert.equal(findRepository(join(belowConfig, "leaf")), above, "a higher .git did not win");

		// A linked worktree carries `.git` as a pointer file rather than a
		// directory, and it selects the worktree from the root and from a
		// descendant — not the shared Git directory it points at.
		const shared = join(ranked, "shared-git");
		mkdirSync(join(shared, "worktrees/linked"), { recursive: true });
		const worktree = join(ranked, "linked-worktree");
		mkdirSync(join(worktree, "knowledge/notes"), { recursive: true });
		writeFileSync(join(worktree, ".git"), `gitdir: ${join(shared, "worktrees/linked")}\n`);
		assert.equal(findRepository(worktree), worktree, "a .git pointer file lost its worktree");
		assert.equal(
			findRepository(join(worktree, "knowledge/notes")),
			worktree,
			"a .git pointer file lost its worktree from a descendant",
		);
	} finally {
		rmSync(ranked, { recursive: true, force: true });
	}

	console.log("SOKF_PI_PAIRED_PASS");
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
	await paired();
}
