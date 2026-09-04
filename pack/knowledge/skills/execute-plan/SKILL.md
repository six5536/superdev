---
name: execute-plan
description: "Superdev process: drive /build unattended over the plan's work blocks, on the work's branch, until no block is ready."
---

<skill name="execute-plan" purpose="Deliver the Plan Unattended" input="the plan's id, when not handed off" user-input="$ARGUMENTS" output="every ready block built on the work's branch, and the deferred decisions put to the user in sequence">

<goal persona="delivery driver">
You drive the loop and decide nothing a phase owns. Take the plan given in the input above through `/build`, block by block, without stopping, until no block is ready.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="sokf_read" id="plan-{nnn}-{slug}" when="always" />
<tool_call name="sokf_read" id="issue-{nnn}-{slug}" when="if the plan delivers an issue" />
<tool_call name="sokf_read" id="development-procedure" when="always" />
</bootstrap_actions>

<process_actions>
<gate check="The plan is on file with `lifecycle: open`" on-fail="/scope — the plan is written there" />
<gate check="The plan's contract changes are settled and committed on the work's branch" on-fail="/scope — the go-ahead is the user's" />
<gate check="The working tree is on the work's branch" on-fail="stop; nothing unattended runs on the default branch" />
<step name="BEGIN THE RUN" task="`superdev run begin --next <the first step>`; a refusal names the run that owns the working tree — stop and report it" />
<loop until="no block is ready: every block is ticked, deferred, or blocked by a deferred decision">
<step name="PICK A BLOCK" task="Pick the first unticked block whose `Depends-on` blocks are all ticked, then `superdev run advance --next` naming it" />
<step name="BUILD THE BLOCK" task="Run `/build` for the block in a subagent — the driver's context holds plan state and the decision queue, nothing else — with `superdev run advance` at every real step forward" />
<step name="HANDLE A RETURN" task="A block failing its checks returns to `/build` at most twice, and the third failure defers the block and takes the next ready one. A gate returning to `/scope` is the user's: write the question into the plan's `Deferred decisions`, naming the block it blocks, and continue with the next ready block" />
</loop>
<step name="END THE RUN" task="`superdev run end`, then put the plan's deferred decisions to the user in sequence; record each answer under its question — a fresh `/execute-plan` resumes from the plan" />
</process_actions>

<rules>
<rule level="MUST NOT">commit or merge to the default branch; the run lives on the branch `/scope` created, and a human fast-forwards</rule>
<rule level="MUST NOT">answer a question whose gate returns to `/scope`; it becomes a deferred decision</rule>
<rule level="SHALL">drive the run through `superdev run begin`/`advance`/`end`, so the Stop hook holds the turn open and the watchdog bounds a stalled run</rule>
<rule level="SHALL">end rather than wait: a blocked run writes its questions into the plan and releases the working tree</rule>
</rules>
</skill>
