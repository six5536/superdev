---
type: FeaturePlan
id: plan-023-feature-warnings-are-counted-not-listed
title: A warning is counted by default and listed on request — feature plan
description: Slices delivering `--warnings`, the shared default for the CLI and the hooks, and the counts `--json` has never carried.
lifecycle: done
---

# Feature plan: A warning is counted by default and listed on request

Request:
[issue-036-feature-request-validate-prints-warnings-by-default][sokf:issue-036-feature-request-validate-prints-warnings-by-default]

## Slices

### Slice 1: The report lists a warning only when asked

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `ValidateArgs` gains `--warnings`, which
  [contract-002][sokf:contract-002-cli-superdev] already declares, so this
  slice closes the pending element and the CLI drift test with it
  ([ADR-038][sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]).
  `Report::render_human` in `crates/lib/superdev-core/src/validate/sokf.rs`
  takes whether to list warnings; the counts in the summary line come from
  the findings and not from what was listed, so both stand whichever way
  the flag went. `run_validate` passes the flag through.
- Done-check: a bare `superdev validate` on this repository lists no
  warning and closes with `PASS (0 error(s), 5 warning(s))`;
  `superdev validate --warnings` lists the five; `cargo nextest run -E
  'test(cli_surface_matches_the_contract)'` passes.
- Cases:
  - unit: `render_human` without the flag lists every error, lists no
    warning, and states both counts — covers 1.
  - unit: `render_human` with the flag lists every finding of both
    severities — covers 2.
  - unit: a report of warnings alone renders `PASS` and a non-zero
    warning count without the flag, so a suppressed warning is still
    counted — covers 1.
  - unit: `passed()` is unchanged by the flag, on a report carrying both
    severities — covers 6.
  - e2e: `superdev validate` and `superdev validate --warnings` on this
    repository exit 0 alike and differ only in the listed lines — covers
    1, 2, 6.

### Slice 2: `--json` carries both counts and the same findings

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `Report::to_json` gains `errors` and `warnings` counts and the
  same listing switch `render_human` took, so the two renderers report one
  thing. `run_validate` passes the flag. The `sokf_snapshots` goldens are
  regenerated for the two added keys.
- Done-check: `superdev validate --json` carries `errors` and `warnings`
  and no warning in `findings`; `--json --warnings` lists them; the
  goldens' diff adds two keys per file and moves no finding, severity or
  verdict.
- Cases:
  - unit: `to_json` states both counts whether or not the flag was given,
    and lists warnings in `findings` only with it — covers 4.
  - unit: `to_json`'s counts and `render_human`'s summary agree on the
    same report, both ways round — covers 4.
  - unit: `passed` in the JSON is unchanged by the flag — covers 6.
  - e2e: `superdev validate --json` on this repository parses, states both
    counts, and lists no warning — covers 4.
  - manual: the top-level keys of `superdev validate --json` and of
    `--json --fix` match the `json:` block of
    [contract-002][sokf:contract-002-cli-superdev], key for key — covers
    5. Manual because binding the two by test is
    [I043][sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test],
    which the framed issue puts out of scope.

### Slice 3: The hooks default like the command line

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `hook_validate` in `crates/app/superdev/src/validate_cli.rs` and
  `knowledge_hold` in `crates/app/superdev/src/run.rs` render without
  listing warnings, so one rule governs whoever ran the check. Neither
  hook's verdict moves: the PostToolUse hook blocks on the same findings
  and the Stop hook holds on the same errors.
- Done-check: on a repository carrying one error and one warning, the
  PostToolUse hook's output names the error, names no warning, and states
  both counts; its exit code is 2 as before.
- Cases:
  - e2e: `superdev hook validate` on a knowledge tree with an error and a
    warning prints the error, prints no warning, states both counts, and
    exits 2 — covers 3, 6.
  - e2e: `superdev hook run`'s hold message lists no warning and states
    both counts — covers 3.
  - e2e: a tree whose only findings are warnings leaves the PostToolUse
    hook at exit 0, as today — covers 6.

<!-- sokf:links -->
[sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]: /knowledge/adrs/active/adr-038-a-contract-may-promise-what-is-not-built-yet.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:issue-036-feature-request-validate-prints-warnings-by-default]: /knowledge/issues/open/issue-036-feature-request-validate-prints-warnings-by-default.md
[sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]: /knowledge/issues/open/issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test.md
