import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { createReadTool, createEditTool, createWriteTool } from "@earendil-works/pi-coding-agent";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import sokf from "../../../.pi/extensions/sokf.ts";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

function snapshot(root: string): unknown[] {
	return readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))
		.map((entry) => [entry.name, entry.isDirectory()
			? snapshot(join(root, entry.name)) : readFileSync(join(root, entry.name))]);
}

export default async function () {
	const tools = new Map<string, any>();
	const handlers = new Map<string, any>();
	const messages: Array<{ message: any; options: any }> = [];
	sokf({
		registerTool(tool: any) {
			tools.set(tool.name, tool);
		},
		on(name: string, handler: any) {
			handlers.set(name, handler);
		},
		sendMessage(message: any, options: any) {
			messages.push({ message, options });
		},
	} as any);

	for (const name of ["sokf_search", "sokf_graph", "sokf_overview"]) {
		if (!tools.has(name)) throw new Error(`SOKF Pi adapter did not register ${name}`);
	}
	// The extension serves what a file tool cannot, and nothing else: a
	// session keeps Pi's own file tools on physical paths.
	for (const name of ["read", "edit", "write"]) {
		if (tools.has(name)) throw new Error(`SOKF Pi adapter registered ${name}, which it must not`);
	}
	if (tools.size !== 3) {
		throw new Error(`SOKF Pi adapter registered ${tools.size} tools rather than 3`);
	}

	const projectContext = { cwd: repository, ui: { notify() {} } };
	const search = await tools
		.get("sokf_search")
		.execute("search", { query: "safe mutation", limit: 1 }, undefined, undefined, projectContext);
	if (search.content[0]?.type !== "text") throw new Error("SOKF search did not return text content");
	const graph = await tools
		.get("sokf_graph")
		.execute("graph", { id: "architecture" }, undefined, undefined, projectContext);
	// A traversal reaches a source file without a second lookup, so every
	// concept the graph names carries its repository-relative path.
	if (!graph.content[0]?.text?.includes("knowledge/architecture.md")) {
		throw new Error("SOKF graph did not carry the repository-relative path of a concept");
	}
	const overview = await tools
		.get("sokf_overview")
		.execute("overview", {}, undefined, undefined, projectContext);
	if (overview.content[0]?.type !== "text") throw new Error("SOKF overview did not return text content");

	const sandbox = mkdtempSync(join(tmpdir(), "sokf-pi-adapter-"));
	try {
		mkdirSync(join(sandbox, ".git"));
		mkdirSync(join(sandbox, "knowledge"));
		writeFileSync(join(sandbox, "knowledge", "manifest.sokf.yaml"), 'sokf: "0.1"\nname: smoke\n');
		writeFileSync(join(sandbox, "knowledge", "a.md"), "---\ntype: T\nid: alpha\n---\nAlpha.\n");
		const context = { cwd: join(sandbox, "knowledge"), ui: { notify() {} } };
		const turnEnd = handlers.get("turn_end");
		// Pi's own tools return physical bytes, including frontmatter, and
		// accept an unchanged read excerpt as an edit anchor.
		const path = "knowledge/a.md";
		const bytes = readFileSync(join(sandbox, path), "utf8");
		const read = await createReadTool(sandbox).execute("read", { path });
		assert.equal(read.content[0]?.type, "text");
		assert.equal((read.content[0] as any).text, bytes);
		await createEditTool(sandbox).execute("edit", {
			path, edits: [{ oldText: bytes, newText: bytes.replace("Alpha.", "Changed.") }],
		});
		assert.equal(readFileSync(join(sandbox, path), "utf8"), bytes.replace("Alpha.", "Changed."));
		const linked = "---\ntype: T\nid: gamma\n---\n[Alpha](a.md)\n";
		await createWriteTool(sandbox).execute("write", { path: "knowledge/c.md", content: linked });
		assert.equal(readFileSync(join(sandbox, "knowledge/c.md"), "utf8"), linked,
			"write repaired knowledge before turn end");
		await turnEnd({}, context);
		assert.match(readFileSync(join(sandbox, "knowledge/c.md"), "utf8"), /\[sokf:alpha\]/);
		assert.equal(messages.length, 0);
		const valid = snapshot(sandbox);
		await turnEnd({}, context);
		assert.deepEqual(snapshot(sandbox), valid, "a clean turn changed the tree");
		assert.equal(messages.length, 0);

		// Written the way anything else writes: directly, with no SOKF tool
		// involved. The turn-end check is what makes the knowledge consistent,
		// whatever wrote it.
		writeFileSync(
			join(sandbox, "knowledge", "b.md"),
			"---\ntype: T\nid: beta\nlinks:\n  - rel: depends-on\n    to: missing\n---\nBeta.\n",
		);

		// `b.md` links to a concept that does not exist, so the tree fails
		// validation and every turn end reports it.
		await turnEnd({}, context);
		const failing = snapshot(sandbox);
		await turnEnd({}, context);
		await turnEnd({}, context);
		assert.deepEqual(snapshot(sandbox), failing, "follow-up state was persisted");
		const triggering = messages.filter(({ options }) => options.triggerTurn);
		if (triggering.length !== 1) {
			throw new Error(
				`The first report of a sequence triggers exactly one turn; got ${triggering.length}`,
			);
		}
		if (messages.length !== 3) {
			throw new Error(`Every failing turn end reports; got ${messages.length} of 3`);
		}
		if (messages.slice(1).some(({ options }) => options.deliverAs !== "nextTurn")) {
			throw new Error("A later report must be visible and non-triggering");
		}

		// The run is unconditional: a turn that called none of this extension's
		// tools still validates, so a write through `bash` is covered. Fixing
		// the tree by hand and ending a turn must clear the sequence.
		const before = messages.length;
		writeFileSync(join(sandbox, "knowledge", "b.md"), "---\ntype: T\nid: beta\n---\nBeta.\n");
		await turnEnd({}, context);
		if (messages.length !== before) {
			throw new Error("A turn end on a valid tree must send no message");
		}

		// The sequence reset, so the next failure triggers again rather than
		// staying silent for the rest of the session.
		writeFileSync(
			join(sandbox, "knowledge", "b.md"),
			"---\ntype: T\nid: beta\nlinks:\n  - rel: depends-on\n    to: missing\n---\nBeta.\n",
		);
		await turnEnd({}, context);
		if (messages.at(-1)?.options.triggerTurn !== true) {
			throw new Error("A clean run must reset the sequence so a later failure triggers again");
		}
	} finally {
		await handlers.get("session_shutdown")?.({}, projectContext);
		rmSync(sandbox, { recursive: true, force: true });
	}
}
