export type ProgressUpdate = { stage: string; activity?: string };

export async function withProgress<T>(
	ctx: any,
	options: { key: string; title: string; stage: string },
	run: (signal: AbortSignal, update: (value: ProgressUpdate) => void) => Promise<T>,
): Promise<T> {
	const controller = new AbortController();
	const started = Date.now();
	let stage = options.stage;
	let activity = "";
	let finishUi: ((value: "complete" | "cancel") => void) | undefined;
	let requestRender: (() => void) | undefined;
	const timer = setInterval(() => requestRender?.(), 1_000);
	timer.unref();
	ctx.ui.setStatus(options.key, `${options.title} · ${stage}`);

	const update = (value: ProgressUpdate) => {
		stage = value.stage;
		activity = value.activity ?? activity;
		ctx.ui.setStatus(options.key, `${options.title} · ${stage}`);
		requestRender?.();
	};

	const execution = run(controller.signal, update).then(
		(value) => { finishUi?.("complete"); return value; },
		(error) => { finishUi?.("complete"); throw error; },
	);

	try {
		if (ctx.hasUI !== false) {
			const selected = await ctx.ui.custom((tui: any, theme: any, _keybindings: any, done: (value: "complete" | "cancel") => void) => {
				finishUi = done;
				requestRender = () => tui.requestRender();
				return {
					render(width: number) {
						const elapsed = Math.floor((Date.now() - started) / 1_000);
						const clock = `${Math.floor(elapsed / 60)}:${String(elapsed % 60).padStart(2, "0")}`;
						const lines = [
							theme.fg("accent", options.title),
							theme.fg("text", `${stage} · ${clock}`),
						];
						if (activity) lines.push(theme.fg("muted", activity.slice(0, Math.max(1, width))));
						lines.push(theme.fg("dim", "Esc to stop and preserve partial work"));
						return lines;
					},
					invalidate() {},
					handleInput(data: string) { if (data === "\u001b") done("cancel"); },
				};
			});
			if (selected === "cancel") controller.abort();
		}
		return await execution;
	} finally {
		clearInterval(timer);
		finishUi = undefined;
		requestRender = undefined;
		ctx.ui.setStatus(options.key, undefined);
	}
}
