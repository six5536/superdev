---
type: Plan
id: plan-022-decidable-findings-are-errors
title: A decidable finding is an error
description: Blocks closing the promised run-state fields, promoting the five findings the repository alone settles, scoping the edit-time hook off the two that span files, and holding the turn open while the knowledge carries an error.
lifecycle: done
links:
  - rel: implements
    to: issue-012-five-decidable-findings-only-warn
    note: The plan delivers the framed issue's two criteria under ADR-039.
---

# Plan: a decidable finding is an error

Request:
[issue-012][sokf:issue-012-five-decidable-findings-only-warn],
decided by
[ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate].

## Goal

The five findings the repository alone settles — a broken body link, a
missing `resource`, a missing `sources[].resource`, an index entry
naming a missing file, and a footnote label matching no
`sources[].id` — fail the run instead of warning, and the Stop hook
holds the turn open while the knowledge carries an error, so a finding
is read the turn it appears. The edit-time hook stops judging the two
findings only the whole tree settles, because it is handed one edited
file. ADR-039 settled the open question the issue carried, before this
plan was cut. Blocks 1 and 4 of the first cut were merged during
build: the hold cap and the hook that respects it are one deliverable.

## Contract changes

- contract-009-interface-run-state: `holds` and `HOLD_CAP`, already
  declared, are landed by block 1, which closes the `PENDING` the
  interface drift test reports until the code matches the declaration
  (ADR-038).

## Work blocks

### Block 1: The run state holds the turn open

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `holds` and `HOLD_CAP` land on the run state, as
  [contract-009][sokf:contract-009-interface-run-state] already declares
  them, together with the `hook run` behaviour that uses them: it refuses
  to end the turn while `validate` reports an error, naming the findings
  on stderr and incrementing `holds`; it resets `holds` once the
  knowledge is clean; it lets the turn end when the knowledge cannot be
  read or checked; and it stops holding at `HOLD_CAP`.

  First because the contract promises the two fields and the code does
  not, which the interface drift test reports as `PENDING` until this
  lands (ADR-038). The state and the hook ship together because a cap
  with nothing that holds is dead code, not a block — the first cut
  split them and clippy said so.
- Done-check: `every_declared_signature_exists_in_the_source` passes; a
  turn ending with the knowledge in error is held once and named; the
  same turn is held no more than `HOLD_CAP` times; unreadable knowledge
  ends the turn. The findings used here are the ones already fatal —
  block 2 is what adds the five, and a broken body link holds a turn
  only from then on.
- Cases:
  - unit: a state file with no `holds` key reads as zero, so a run armed
    by an older binary is not orphaned.
  - unit: `holds` round-trips through a write and a read.
  - integration: a Stop payload against a tree whose knowledge is in
    error exits 2 and names the finding — covers 1, 2.
  - integration: a hold count belongs to its session, so one session
    cannot spend another's cap.
  - integration: the same payload against a clean tree exits 0, and
    `holds` is back to zero.
  - integration: the `HOLD_CAP`-th hold reports and exits 0, so an
    unresolvable finding stalls nothing.
  - integration: unreadable knowledge exits 0, so the hook fails open.
  - integration: an armed run still continues as contract-009 says, so
    the two jobs of the hook do not interfere.

### Block 2: The five findings fail the run

- [x] Done — ticked by integrate at merge.
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

### Block 3: The edit-time hook stops judging what it cannot see

- [x] Done — ticked by integrate at merge.
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

### Block 4: The knowledge and the records settle

- [x] Done — ticked by integrate at merge.
- Depends-on: 2, 3.
- Change: the canonical knowledge and the pack mirror pass with the five
  enforced, the changelog carries the change, and the documentation the
  hooks are configured from says what holds a turn open.
- Done-check: `superdev validate` and `superdev validate pack` both pass,
  and the changelog names the new failure class.
- Cases:
  - integration: the live tree and the pack mirror validate clean.

<!-- sokf:links -->
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:contract-009-interface-run-state]: /knowledge/contracts/internal/active/contract-009-interface-run-state.md
[sokf:issue-012-five-decidable-findings-only-warn]: /knowledge/issues/done/issue-012-five-decidable-findings-only-warn.md
