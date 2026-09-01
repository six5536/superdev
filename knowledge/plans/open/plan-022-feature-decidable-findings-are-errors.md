---
type: FeaturePlan
id: plan-022-feature-decidable-findings-are-errors
title: A decidable finding is an error — feature plan
description: Slices closing the promised run-state fields, promoting the five findings the repository alone settles, scoping the edit-time hook off the two that span files, and holding the turn open while the knowledge carries an error.
lifecycle: open
links:
  - rel: implements
    to: issue-012-feature-request-five-decidable-findings-only-warn
    note: The plan delivers the framed issue's two criteria under ADR-039.
---

# Feature plan: a decidable finding is an error

Request:
[issue-012][sokf:issue-012-feature-request-five-decidable-findings-only-warn],
decided by
[ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate].

## Slices

### Slice 1: The run state carries the hold count

- [ ] Not started.
- Depends-on: none.
- Change: `holds` and `HOLD_CAP` land on the run state, as
  [contract-009][sokf:contract-009-interface-run-state] already declares
  them. Nothing reads them yet. First because the contract promises them
  and the code does not, which the interface drift test reports as
  `PENDING` until this lands (ADR-038).
- Done-check: `every_declared_signature_exists_in_the_source` passes,
  and a state file written before this change still reads.
- Cases:
  - unit: a state file with no `holds` key reads as zero, so a run armed
    by an older binary is not orphaned.
  - unit: `holds` round-trips through a write and a read.

### Slice 2: The five findings fail the run

- [ ] Not started.
- Depends-on: none.
- Change: the five findings the repository alone settles become errors
  in `validate::sokf` — a broken body link, a missing `resource`, a
  missing `sources[].resource`, an index entry naming a missing file,
  and a footnote label matching no `sources[].id`. The non-core `rel`
  stays a warning, being the one the repository cannot settle. The
  golden snapshots move with them.
- Done-check: a tree carrying one of each exits 1 rather than 0, and
  the only warning the document check can still emit is the `rel` one.
- Cases:
  - unit: each of the five is reported as an error, naming the file and
    the target — covers 1, 2.
  - unit: a non-core `rel` is still a warning, so the tier is split by
    decidability and not emptied — covers 2.
  - golden: the document-check snapshots carry the new severities.

### Slice 3: The edit-time hook stops judging what it cannot see

- [ ] Not started.
- Depends-on: 2.
- Change: `hook validate` no longer blocks on the two findings only the
  whole tree settles — a broken body link and an index entry naming a
  missing file — because it is handed one edited file and cannot see
  whether the target arrives in the next edit. It still reports them.
  Every other error blocks as it does today.
- Done-check: editing a governed file to add a link to a file that does
  not exist exits 0, and the same file with a malformed `type` exits 2.
- Cases:
  - integration: a new broken body link in an edited concept exits 0.
  - integration: a new index entry naming a missing file exits 0.
  - integration: a missing `resource` in the same file still exits 2, so
    the hook was scoped and not disarmed.

### Slice 4: The turn does not end on an unresolved finding

- [ ] Not started.
- Depends-on: 1, 3.
- Change: `hook run` refuses to end the turn while `validate` reports an
  error, naming the findings on stderr and incrementing `holds`. It
  resets `holds` once the knowledge is clean, lets the turn end when the
  knowledge cannot be read or checked, and stops holding at `HOLD_CAP`.
  This is what keeps the two slice 3 stopped blocking on from being
  ignored.
- Done-check: a turn ending with a broken body link is held once and
  named; the same turn is held no more than `HOLD_CAP` times; a knowledge
  that cannot be read ends the turn.
- Cases:
  - integration: a Stop payload against a tree carrying a broken link
    exits 2 and names the finding — covers 1, 2.
  - integration: the same payload against a clean tree exits 0, and
    `holds` is back to zero.
  - integration: the `HOLD_CAP`-th hold reports and exits 0, so an
    unresolvable finding stalls nothing.
  - integration: unreadable knowledge exits 0, so the hook fails open.
  - integration: an armed run still continues as contract-009 says, so
    the two jobs of the hook do not interfere.

### Slice 5: The knowledge and the records settle

- [ ] Not started.
- Depends-on: 2, 4.
- Change: the canonical knowledge and the pack mirror pass with the five
  enforced, the changelog carries the change, and the documentation the
  hooks are configured from says what holds a turn open.
- Done-check: `superdev validate` and `superdev validate pack` both pass,
  and the changelog names the new failure class.
- Cases:
  - integration: the live tree and the pack mirror validate clean.

## Deferred decisions

- None. ADR-039 settled the open question the issue carried, before this
  plan was cut.

<!-- sokf:links -->
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:contract-009-interface-run-state]: /knowledge/contracts/internal/active/contract-009-interface-run-state.md
[sokf:issue-012-feature-request-five-decidable-findings-only-warn]: /knowledge/issues/open/issue-012-feature-request-five-decidable-findings-only-warn.md
