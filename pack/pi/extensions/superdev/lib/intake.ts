import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { Type } from "typebox";
import { StringEnum } from "@earendil-works/pi-ai";

/** Shared intake UI; review queues use the same choices and discussion vocabulary. */
export function registerIntakeTools({ pi, workflowStatus, runSuperdev, authority, runtime, here }: any) {
    pi.registerTool({
        name: "superdev_ask",
        label: "Discuss workflow intent",
        description: "Ask one intake question with choices and a recommendation; allow a typed answer, chat, or cancellation",
        parameters: Type.Object({
            question: Type.String(),
            choices: Type.Array(Type.String(), { minItems: 1, maxItems: 8 }),
            recommendation: Type.String(),
        }),
        executionMode: "sequential",
        execute: async (_id: string, input: any, _signal: AbortSignal, _update: unknown, ctx: any) => {
            if (!ctx.hasUI) return result({ status: "human-input-required", ...input });
            const answer = await ctx.ui.select(`${input.question}\nRecommendation: ${input.recommendation}`, [...input.choices, "Type another answer", "Discuss in chat", "Cancel"]);
            if (!answer || answer === "Cancel") return result({ status: "cancelled" });
            if (answer === "Discuss in chat") return result({ status: "discuss", question: input.question });
            const text = answer === "Type another answer" ? await ctx.ui.input(input.question) : answer;
            return result(text?.trim() ? { status: "answered", answer: text.trim() } : { status: "cancelled" });
        },
    });
    pi.registerTool({
        name: "superdev_file_issue",
        label: "Create issue",
        description: "Confirm, allocate, validate and commit a bug, feature, or chore on the default branch without starting a workflow",
        parameters: Type.Object({
            title: Type.String(),
            description: Type.String(),
            kind: StringEnum(["bug", "feature", "chore"] as const),
        }),
        executionMode: "sequential",
        execute: async (_id: string, input: any, _signal: AbortSignal, _update: unknown, ctx: any) => {
            const status = await workflowStatus(ctx.cwd);
            if (runtime.modifyingBusy || status.busy || status.owner) throw new Error("Pause the existing workflow and explicitly return to the default branch before filing another issue.");
            if (!ctx.hasUI) return result({ status: "human-input-required" });
            if (!(await ctx.ui.confirm(`Create ${input.kind} on ${status.defaultBranch}?`, `${input.title}\n\n${input.description}`))) return result({ status: "cancelled" });
            return result(await runSuperdev(["file", "--human-approved", "--kind", "issue", "--issue-kind", input.kind,
                "--title", input.title, "--description", input.description, "--default-branch", status.defaultBranch], ctx.cwd, authority));
        },
    });
    for (const name of ["issue", "file"]) pi.registerCommand(name, {
        description: name === "issue" ? "Create a canonical issue on the default branch" : "Alias for /issue",
        handler: async (args: string, ctx: any) => {
            if (!ctx.isIdle()) return ctx.ui.notify("Issue capture requires an idle session", "warning");
            const skill = await readFile(resolve(here, "../../skills/issue/SKILL.md"), "utf8");
            pi.sendUserMessage(`${skill}\n\nUser: ${args}`);
        },
    });
}

function result(value: unknown) {
    return { content: [{ type: "text", text: JSON.stringify(value) }], details: value };
}
