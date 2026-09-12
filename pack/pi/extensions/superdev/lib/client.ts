import { randomBytes } from "node:crypto";
import type { ExtensionContext } from "@earendil-works/pi-coding-agent";
import { pinService } from "./service-pin.ts";
import { runPinnedSuperdev } from "./service-exec.ts";

export const steps = ["select-issue", "interview-issue", "write-issue", "check-issue", "approve-issue",
	"write-plan", "interview-plan", "update-plan", "check-plan", "approve-plan", "handoff"] as const;
export type ScopeStep = typeof steps[number];
export const stages = ["implementation", "verification", "review", "acceptance"] as const;
export type ExecutionStage = typeof stages[number];
export type Checkpoint = { completed_blocks: number[]; unfinished: string; evidence: string[] };
/** One assessment's own report, bound to the candidate it judged. */
export type AssessmentReport = {
	stage: ExecutionStage;
	candidate?: string;
	findings: string;
	/** Why the report cannot stand as a verdict, when it cannot. */
	void?: string;
};
export type WorkerRecord = { session: string; session_file?: string; anchor?: string };
export type ScopeRecord = {
	id: string; revision: number; checkout: string; issue: string; plan?: string;
	phase: "scope" | "build" | "accept" | "done" | "abandoned";
	scope_step: ScopeStep; default_branch: string; work_branch?: string;
	steps: Array<{ step: ScopeStep; outcome: "completed" | "skipped"; note: string }>;
	discussion?: string; recovery?: string;
	issue_approval?: { original: { hash: string }; compatible: string[] };
	plan_approval?: { original: { hash: string }; compatible: string[] };
	pending_publication?: unknown; pending_build_start?: unknown;
	execution_mode?: "current" | "worker";
	worker_session?: WorkerRecord;
	stage?: ExecutionStage;
	checkpoint: Checkpoint;
	retries: Record<string, number>;
	candidate?: string;
	assessment?: AssessmentReport;
};
export type WorkflowStatus = {
	defaultBranch?: string; defaultBranchDiagnostic?: string;
	currentBranch?: string; currentBranchDiagnostic?: string;
	/** Absent when the manifest is unreadable; absence is never automatic acceptance. */
	humanAcceptanceRequired?: boolean;
	/** Fingerprint of branch, HEAD, and every pending change. Absent when unreadable. */
	worktreeState?: string;
	workflows: Array<{ record: ScopeRecord; approval: { executable: boolean; reason?: string } }>;
	claim?: { workflow: string; session: string; child_pid?: number };
};

/** Only this controller holds the capability. Model tools supply intent, not credentials. */
export class WorkflowClient {
	private readonly authority = randomBytes(32).toString("hex");
	private pin?: Promise<Awaited<ReturnType<typeof pinService>>>;
	constructor(private readonly launcher = "superdev") {}

	async status(cwd: string, signal?: AbortSignal): Promise<WorkflowStatus> {
		const result = await this.exec(["workflow", "status", "--json"], cwd, undefined, signal);
		if (!Array.isArray(result.workflows)) throw new Error("Workflow status omitted local records");
		return result as WorkflowStatus;
	}

	async apply(request: Record<string, unknown>, ctx: ExtensionContext, signal?: AbortSignal): Promise<ScopeRecord> {
		const result = await this.exec(["workflow", "apply", "--session", ctx.sessionManager.getSessionId(),
			"--owner-pid", String(process.pid)], ctx.cwd, JSON.stringify(request), signal);
		if (!result.record || typeof result.record.id !== "string" || !Number.isSafeInteger(result.record.revision)) {
			throw new Error("Workflow mutation omitted its local record");
		}
		return result.record as ScopeRecord;
	}

	private async exec(args: string[], cwd: string, input?: string, signal?: AbortSignal) {
		this.pin ??= pinService(this.launcher, cwd).catch((error) => { this.pin = undefined; throw error; });
		const service = await this.pin;
		const result = await runPinnedSuperdev(service.path, service.digest, args, cwd, signal,
			input === undefined ? undefined : { SUPERDEV_UI_AUTHORITY: this.authority }, input);
		if (result.code !== 0) throw new Error(result.stderr.trim() || `Workflow service exited ${result.code}`);
		let response;
		try { response = JSON.parse(result.stdout); }
		catch (error) { throw new Error("Workflow service returned invalid JSON", { cause: error }); }
		if (response.protocol !== "superdev-workflow/v3" || !response.result || typeof response.result !== "object") {
			throw new Error("Workflow service returned an incompatible protocol");
		}
		return response.result;
	}

	async dispose() { if (this.pin) (await this.pin).dispose(); this.pin = undefined; }
}
