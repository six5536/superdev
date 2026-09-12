/** Authoritative workflow and Git mutation commands do not run through model bash. */
export function modelMayNotRun(command: string): boolean {
	// The retired v2 verbs stay listed: an obsolete instruction must be blocked
	// here rather than reach a service that would refuse it less clearly.
	return /\bsuperdev\s+workflow\s+(?:apply|start|resume|bind|cancel|transition|abandon|commit|activity-start|activity-finish)\b/.test(command)
		|| /\bgit\s+(?:add|commit|update-ref|reset|switch|checkout|merge|rebase|cherry-pick|branch|tag|stash|clean|restore|rm|mv)\b/.test(command);
}

/** Missing project policy is a refusal, never implicit automatic acceptance. */
export function requiresHumanAcceptance(value: boolean | undefined): boolean {
	if (value === undefined) throw new Error("ACCEPT status omitted the configured human-acceptance policy");
	return value;
}
