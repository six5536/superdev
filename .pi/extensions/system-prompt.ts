import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export default function (pi: ExtensionAPI) {
	pi.registerCommand("system-prompt", {
		description: "Write the current Pi system prompt to .pi/current-system-prompt.md",
		handler: async (args, ctx) => {
			const prompt = ctx.getSystemPrompt();
			const cwd = ctx.getSystemPromptOptions()?.cwd ?? process.cwd();
			const outputPath = join(cwd, ".pi", "current-system-prompt.md");

			mkdirSync(join(cwd, ".pi"), { recursive: true });
			writeFileSync(outputPath, prompt, "utf8");

			const message = `Current system prompt written to ${outputPath} (${prompt.length} chars)`;
			ctx.ui.notify(message, "info");

			if (args.trim() === "show") {
				console.log(`\n----- Pi system prompt (${prompt.length} chars) -----\n${prompt}\n----- end system prompt -----\n`);
			}
		},
	});
}
