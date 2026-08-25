# Process dispatch guide

How a task gets routed to a process. Selection is recognition, not lookup — it runs on three signals: the request shape, situation triggers, and composition.

## 1. Request shape — what should be different when the task is done

Most requests declare their process in the verb and the deliverable:

| Request sounds like | Process | Notes |
|---|---|---|
| "Add / build / support X" | [feature-implementation](feature-implementation.md) | Pulls in [planning](planning.md) when scope warrants it |
| "X is broken / crashes / returns wrong Y" | [bug-fix](bug-fix.md) | Only once reproduced; until then it's debugging |
| "Why does X happen?" / symptom described, no ask | [debugging](debugging.md) | Deliverable is the **finding** — don't fix unasked |
| "Clean up / restructure / this is a mess" | [refactoring](refactoring.md) | Behavior-preserving by definition |
| "Make it faster / it's slow" | [performance-work](performance-work.md) | A measurement loop, not a patch loop |
| "Review this / look at my PR" | [code-review](code-review.md) | |
| "Is this safe / secure?" | [security-review](security-review.md) | Extends code-review with a threat model |
| "Write tests for X" | [testing](testing.md) | Also invoked as a step by most other processes |
| "How does this codebase work?" | [codebase-exploration](codebase-exploration.md) | Also the first step of most unfamiliar work |
| "Plan / design X first" | [planning](planning.md) | Output per `templates/plan.md` or `templates/design-doc.md` |
| "Commit / push / open a PR" | [commit-and-pr](commit-and-pr.md) | Only when asked |
| "Ship it / cut a release" | [release](release.md) | Has a built-in confirmation gate |
| "Can we use X? Would approach Y work?" | [spike-prototype](spike-prototype.md) | Only when the answer needs code; otherwise plain research |
| "Add / upgrade / remove package X" | [dependency-changes](dependency-changes.md) | |

## 2. Situation triggers — processes entered by events, not requests

Nobody asks for these; the situation invokes them mid-task, interrupting or splicing into whatever process is running. They only work if their triggers are watched for **while inside another process**.

| Trigger | Process | Priority |
|---|---|---|
| Evidence says my change broke something, or my approach is wrong | [mistake-recovery](mistake-recovery.md) | Immediately — interrupts everything |
| CI goes red on my push | [ci-failures](ci-failures.md) | Before any further pushes |
| A sync/merge conflicts — or merges suspiciously cleanly | [merge-conflicts](merge-conflicts.md) | Before building on the merged tree |
| A change invalidates a doc, example, or help text | [documentation-upkeep](documentation-upkeep.md) | Same commit as the change |

## 3. Composition — processes nest rather than compete

Selection is rarely "which one?" — it's "which one is the **spine**, and which get invoked as steps?" A single "add feature X" run typically threads through five:

```
codebase-exploration → planning → feature-implementation → testing → commit-and-pr
                                        │
                                        ├─ dependency-changes   (if a package is added)
                                        ├─ documentation-upkeep (if docs are invalidated)
                                        └─ mistake-recovery     (if something goes wrong)
```

Other common spines:

- **bug-fix** ← debugging (find the cause) → testing (regression test) → commit-and-pr
- **release** ← ci-failures (if the pipeline breaks) → documentation-upkeep (changelog, notes)
- **spike-prototype** → reports via `templates/investigation.md`, then restarts as feature-implementation if pursued

## Edge rules

- **Ambiguity resolves toward the least-committal process.** Unsure if it's a bug or intended behavior → debug and report before fixing. Unsure if a refactor is wanted → note it, don't do it. Escalating to a heavier process is cheap; un-doing an unwanted fix isn't.
- **Proportionality scales a process, never skips its spine.** A one-line fix gets no written plan, but still gets reproduce → fix → verify → report. Steps compress; they don't disappear.
- **Event triggers outrank the current process.** Mistake-recovery in particular preempts whatever was running; finish the recovery, then resume.
- **Every process ends with a report** — outcome first, verification stated honestly, deferred work named. If a run ends without one, the process isn't finished.

## Directory index

**Request-driven:** [bug-fix](bug-fix.md) · [code-review](code-review.md) · [codebase-exploration](codebase-exploration.md) · [commit-and-pr](commit-and-pr.md) · [debugging](debugging.md) · [dependency-changes](dependency-changes.md) · [feature-implementation](feature-implementation.md) · [performance-work](performance-work.md) · [planning](planning.md) · [refactoring](refactoring.md) · [release](release.md) · [security-review](security-review.md) · [spike-prototype](spike-prototype.md) · [testing](testing.md)

**Event-driven:** [ci-failures](ci-failures.md) · [documentation-upkeep](documentation-upkeep.md) · [merge-conflicts](merge-conflicts.md) · [mistake-recovery](mistake-recovery.md)

Document templates these processes produce live one level up in `templates/`.
