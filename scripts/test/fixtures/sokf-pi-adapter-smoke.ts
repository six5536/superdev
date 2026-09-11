import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import sokf from "../../../.pi/extensions/sokf.ts";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

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

	for (const name of ["read", "edit", "write", "sokf_search", "sokf_graph"]) {
		if (!tools.has(name)) throw new Error(`SOKF Pi adapter did not register ${name}`);
	}

	const projectContext = { cwd: repository, ui: { notify() {} } };
	// A routed read returns the source exactly: the first line of a concept
	// file is its frontmatter fence, not a rendered heading.
	const read = await tools
		.get("read")
		.execute("read", { path: "sokf:architecture", limit: 1 }, undefined, undefined, projectContext);
	if (read.content[0]?.type !== "text" || !read.content[0].text.startsWith("---")) {
		throw new Error("SOKF routed read did not return exact source through Pi's read");
	}
	const search = await tools
		.get("sokf_search")
		.execute("search", { query: "safe mutation", limit: 1 }, undefined, undefined, projectContext);
	if (search.content[0]?.type !== "text") throw new Error("SOKF search did not return text content");
	await tools
		.get("sokf_search")
		.execute("search-again", { query: "safe mutation", limit: 1 }, undefined, undefined, projectContext);
	await tools
		.get("sokf_search")
		.execute("search-third", { query: "safe mutation", limit: 1 }, undefined, undefined, projectContext);
	for (const refused of ["sokf:not-a-concept", "sokf:", "sokf:architecture#Approach"]) {
		await tools
			.get("read")
			.execute("refused", { path: refused }, undefined, undefined, projectContext)
			.then(
				() => {
					throw new Error(`SOKF read accepted \`${refused}\` instead of refusing it`);
				},
				() => undefined,
			);
	}

	const sandbox = mkdtempSync(join(tmpdir(), "sokf-pi-adapter-"));
	try {
		mkdirSync(join(sandbox, ".git"));
		mkdirSync(join(sandbox, "knowledge"));
		writeFileSync(join(sandbox, "knowledge", "manifest.sokf.yaml"), 'sokf: "0.1"\nname: smoke\n');
		writeFileSync(join(sandbox, "knowledge", "a.md"), "---\ntype: T\nid: alpha\n---\nOld.\n");
		const context = { cwd: join(sandbox, "knowledge"), ui: { notify() {} } };

		const edit = await tools
			.get("edit")
			.execute(
				"edit",
				{ path: "sokf:alpha", edits: [{ oldText: "Old.", newText: "New." }] },
				undefined,
				undefined,
				context,
			);
		if (!edit.details?.patch || !readFileSync(join(sandbox, "knowledge", "a.md"), "utf8").includes("New.")) {
			throw new Error("SOKF virtual edit did not preserve Pi's edit details");
		}

		await tools.get("write").execute(
			"write",
			{
				path: "knowledge/b.md",
				content: "---\ntype: T\nid: beta\nlinks:\n  - rel: depends-on\n    to: missing\n---\nBeta.\n",
			},
			undefined,
			undefined,
			context,
		);
		if (!readFileSync(join(sandbox, "knowledge", "b.md"), "utf8").includes("Beta.")) {
			throw new Error("SOKF physical write was not routed from a subdirectory");
		}

		const turnEnd = handlers.get("turn_end");
		await turnEnd({}, context);
		await turnEnd({}, context);
		await turnEnd({}, context);
		if (messages.filter(({ options }) => options.triggerTurn).length !== 2) {
			throw new Error("SOKF repair feedback was not bounded at two turns");
		}
		if (messages.at(-1)?.options.deliverAs !== "nextTurn") {
			throw new Error("A persistent SOKF validation failure was not deferred for manual continuation");
		}
	} finally {
		await handlers.get("session_shutdown")?.({}, projectContext);
		rmSync(sandbox, { recursive: true, force: true });
	}
}
