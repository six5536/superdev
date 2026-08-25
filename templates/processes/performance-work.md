# Process: Performance work

## 1. Define the target before touching code

- Turn "slow" into a number: which operation, measured how, currently X, needs to be Y. Without a target, optimization has no finish line.
- Confirm the workload that matters: realistic data sizes and shapes, not toy inputs — code that's fast on 10 rows can be quadratic on 10,000.

## 2. Measure the baseline

- Build a repeatable measurement: a benchmark script, a timed test, a profiler run — with fixed inputs and enough runs to see variance.
- Record the baseline numbers and the exact command that produced them. Every later claim of improvement is relative to this.

## 3. Profile — never optimize from intuition

- Use a profiler (or targeted timing) to find where time/memory actually goes. The hot spot is routinely not where it "obviously" is.
- Distinguish the cost categories: algorithmic complexity, I/O waits, allocation churn, redundant work (cache misses, repeated computation). The fix differs for each.

## 4. Change one thing at a time

- Apply the single highest-leverage fix suggested by the profile — usually algorithm/data-structure choice or eliminating repeated work, rarely micro-tweaks.
- Re-measure with the same command after each change. Keep changes that pay measurably; revert ones that don't, even if they "should" help.
- Watch for what the optimization costs: readability, memory-for-speed trades, new invariants (a cache that must be invalidated). Note these in the change.

## 5. Guard the win

- Verify behavior is unchanged: the full test suite, plus output-equivalence on the benchmark inputs.
- Keep the benchmark in the repo where the project has a home for one, so regressions are catchable.
- Stop at the target. Past it, further optimization is complexity spent on nothing.

## 6. Report

- Before/after numbers with the measurement command, what the profile showed, what was changed and why it worked, and any trade-offs introduced.
