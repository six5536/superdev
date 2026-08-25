<superdev-process>

The superdev development process is as follows.

## 1. Development Process

```mermaid
flowchart TD
    S{{"Start: new project OR existing project"}} --> F["1 · Frame"]
    F --> SP["2 · Spec"]
    SP --> ID["3 · Interface Design"]
    ID --> PL["4 · Plan"]
    PL --> BU["5 · Build"]
    BU --> VE["6 · Verify"]
    VE -->|"fails"| BU
    VE -->|"passes"| IN["7 · Integrate"]
    IN -->|"slices remain"| PL
    IN -->|"last slice"| AC["8 · Accept"]
    AC -->|"gaps"| PL
    AC -->|"next feature"| F
    AC --> DN(["Done"])
```

---

## 2. Phases

Each phase is a skill; its checklist is `.claude/skills/<phase>/SKILL.md`.

1. Frame — state the problem, the user, and the constraints. New
   project: also choose the tech stack and the visual system.
2. Spec — observable behaviour, acceptance criteria, and the test
   plan.
3. Interface Design — decide the interfaces that are expensive to
   change; record each decision as an ADR.
4. Plan — cut the spec into slices small enough to build and verify in
   one pass; the slice list is the plan.
5. Build — implement one slice: tests first, then the code that passes
   them, committed.
6. Verify — check the slice against its done-check and the test plan.
   Failures return to Build.
7. Integrate — merge the slice; update the changelog, the
   knowledgebase, and the plan.
8. Accept — check the whole feature on the real target against the
   acceptance criteria; update the user documentation; file gaps as
   issues. Gaps return to Plan.

---

## 3. Documents

The process reads and writes the knowledgebase (`knowledge/`):

- Specs: `knowledge/specs/Snnn-<feature-slug>.md` — behaviour,
  acceptance criteria, and test plan; tagged `done` at accept.
- Plans: `knowledge/plans/Pnnn-<slug>.md` — the slice list; tagged
  `done` at the last integrate.
- Decisions: `knowledge/decisions/Dnnn-<slug>.md` — ADRs; permanent.
- Issues: `knowledge/issues/Innn-<slug>.md` — gaps and tickets.
- Templates: `knowledge/templates/` — the document skeletons the
  phases use.

</superdev-process>
