# Writing Agent Briefs

An agent brief is what an issue concept's body becomes when the issue moves to `ready-for-agent`. It is the authoritative specification an AFK agent will work from: the original request and triage notes are context — the brief is the contract. It fills the ISSUE-FORMAT.md structure (`/to-plan` carries it) — leave the `# Blocked by` section intact; this file is the guidance on filling the rest well.

## Principles

### Durability over precision

The issue may sit in `ready-for-agent` for days or weeks. The codebase will change in the meantime. Write the brief so it stays useful even as files are renamed, moved, or refactored.

- **Do** describe interfaces, types, and behavioral contracts
- **Do** name specific types, function signatures, or config shapes that the agent should look for or modify
- **Don't** reference file paths — they go stale
- **Don't** reference line numbers
- **Don't** assume the current implementation structure will remain the same

### Behavioral, not procedural

Describe **what** the system should do, not **how** to implement it. The agent will explore the codebase fresh and make its own implementation decisions.

- **Good:** "The `SkillConfig` type should accept an optional `schedule` field of type `CronExpression`"
- **Bad:** "Open src/types/skill.ts and add a schedule field on line 42"
- **Good:** "When a user runs `/triage` with no arguments, they should see a summary of issues needing attention"
- **Bad:** "Add a switch statement in the main handler function"

### Complete acceptance criteria

The agent needs to know when it's done. Every agent brief must have concrete, testable acceptance criteria. Each criterion should be independently verifiable.

- **Good:** "An issue tagged needs-triage appears in the attention list with its one-line summary"
- **Bad:** "Triage should work correctly"

### Explicit scope boundaries

State what is out of scope. This prevents the agent from gold-plating or making assumptions about adjacent features.

## Brief sections

Within the issue concept's body (ISSUE-FORMAT.md defines the frame):

```markdown
# What to build

**Current behaviour:** what happens now. For bugs, the broken
behaviour; for enhancements, the status quo the feature builds on.

**Desired behaviour:** what should happen after the agent's work is
complete. Be specific about edge cases and error conditions.

**Key interfaces:**
- `TypeName` — what needs to change and why
- `functionName()` return type — what it currently returns vs what it
  should return

# Acceptance criteria

- [ ] Specific, testable criterion 1
- [ ] Specific, testable criterion 2

# Out of scope

- Thing that should NOT be changed or addressed in this issue
- Adjacent feature that might seem related but is separate
```

## Examples

### Good agent brief (bug)

```markdown
# What to build

**Current behaviour:** When a skill description exceeds 1024
characters, it is truncated at exactly 1024 characters regardless of
word boundaries, producing descriptions that end mid-word ("Use when
the user wants to confi").

**Desired behaviour:** Truncation breaks at the last word boundary
before 1024 characters and appends "..." to indicate truncation.

**Key interfaces:**
- The `SkillMetadata` type's `description` field — no type change, but
  the validation/processing logic that populates it must respect word
  boundaries
- Any function that reads SKILL.md frontmatter and extracts the
  description

# Acceptance criteria

- [ ] Descriptions under 1024 chars are unchanged
- [ ] Descriptions over 1024 chars are truncated at the last word
      boundary before 1024 chars
- [ ] Truncated descriptions end with "..."
- [ ] The total length including "..." does not exceed 1024 chars

# Out of scope

- Changing the 1024 char limit itself
- Multi-line description support
```

### Good agent brief (enhancement)

```markdown
# What to build

**Current behaviour:** When a feature request is rejected, the issue
keeps its `wontfix` tag but no reasoning is recorded. Future similar
requests require the maintainer to recall the prior decision.

**Desired behaviour:** Rejected requests carry their reasoning in the
issue body and under the backlog concept's "Decided against" section,
and triage's prior-rejection check surfaces a match when a new issue
resembles one.

**Key interfaces:**
- The backlog concept's "Decided against" entry shape: the idea, the
  decision, the reasoning, in one bullet
- Triage step 1(b) reads those entries and reports any resemblance

# Acceptance criteria

- [ ] Rejecting an enhancement records the reasoning in the issue body
      and adds a "Decided against" entry
- [ ] A new issue resembling a recorded rejection is surfaced during
      triage with a pointer to the entry
- [ ] An already-implemented request produces no entry (it was built,
      not rejected)

# Out of scope

- Automated matching (the maintainer confirms the match)
- Reopening previously rejected features
- Bug reports (only enhancement rejections are recorded)
```

### Bad agent brief

```markdown
# What to build

The triage thing is broken. Look at the main file and fix it.
The function around line 150 has the issue.

Files to change:
- src/triage/handler.ts (line 150)
- src/types.ts (line 42)
```

This is bad because:
- Vague description ("the triage thing is broken")
- References file paths and line numbers that will go stale
- No acceptance criteria
- No scope boundaries
- No description of current vs desired behaviour
