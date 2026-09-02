---
name: execute-feature-plan
description: "Superdev process: drive feature-plan, build and integrate unattended over the feature's plan, on its branch, until no slice is ready."
---

<skill name="execute-feature-plan" purpose="Deliver the Plan Unattended" input="the framed issue or feature-plan id, when not handed off" user-input="$ARGUMENTS" output="every ready slice built and integrated on the feature's branch, and the deferred decisions put to the user in sequence">

<goal persona="delivery driver">
You drive the loop and decide nothing a phase owns. Take the feature given in the input above through feature-plan, build and integrate, slice by slice, without stopping, until no slice is ready.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{kind}-{slug}" when="always" />
<tool_call name="sokf_read" id="plan-{nnn}-feature-{slug}" when="if the plan exists" />
<tool_call name="sokf_read" id="development-procedure" when="always" />
</bootstrap_actions>

<process_actions>
<gate check="The framed issue's lifecycle is framed" on-fail="/frame — an unframed issue is framed before it is run" />
<gate check="The framed issue's contracts are settled and committed on the feature's branch" on-fail="/contract-design — the go-ahead is the user's" />
<gate check="The working tree is on the feature's branch" on-fail="stop; nothing unattended runs on the default branch" />
<step name="BEGIN THE RUN" task="`superdev run begin --next <the first step>`; a refusal names the run that owns the working tree — stop and report it" />
<step name="CUT THE PLAN" when="if no plan exists" task="`/feature-plan`; the cutting rules live there" />
<loop until="no slice is ready: every slice is done, deferred, or blocked by a deferred decision">
<step name="PICK A SLICE" task="Pick the first slice whose `Depends-on` are all done, then `superdev run advance --next` naming it" />
<step name="BUILD AND INTEGRATE" task="Run `/build` then `/integrate` for the slice in a subagent — the driver's context holds plan state and the decision queue, nothing else — with `superdev run advance` at every real step forward" />
<step name="HANDLE A RETURN" task="A gate returning to `/build` or `/feature-plan` is yours: follow it — a slice failing its checks returns to `/build` at most twice and the third failure defers it; integrate's replan edge re-enters `/feature-plan` inside the loop. A gate returning to `/frame` or `/contract-design` is the user's: write the question into the plan's `Deferred decisions`, naming the slice it blocks, and continue with the next ready slice" />
</loop>
<step name="END THE RUN" task="`superdev run end`, then put the plan's deferred decisions to the user in sequence; record each answer under its question — a fresh `/execute-feature-plan` resumes from the plan" />
</process_actions>

<rules>
<rule level="MUST NOT">commit or merge to the default branch; the run lives on the branch `/frame` created, and a human fast-forwards</rule>
<rule level="MUST NOT">answer a question whose gate returns to `/frame` or `/contract-design`; it becomes a deferred decision</rule>
<rule level="SHALL">drive the run through `superdev run begin`/`advance`/`end`, so the Stop hook holds the turn open and the watchdog bounds a stalled run</rule>
<rule level="SHALL">end rather than wait: a blocked run writes its questions into the plan and releases the working tree</rule>
</rules>
</skill>
