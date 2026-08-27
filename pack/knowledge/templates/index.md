# Templates

Copy-verbatim skeletons for the documents the development process produces.
Read one with `aokf_read` (id `template-<name>`), strip the frontmatter, and
fill in the angle-bracket placeholders.

## Planning & design

* [Spec](spec.md) - what done looks like from outside — behaviour, acceptance criteria, UI states, edge cases, out of scope. Filed as a draft concept in `knowledge/specs/`, tagged done at accept.
* [Feature Plan](feature-plan.md) - the feature's slice list — per slice a done-check, the assigned test-plan cases, and a done marker. Produced by the feature-plan phase.
* [Ad-hoc Plan](adhoc-plan.md) - ad-hoc implementation plan for one-off work outside the feature workflow — context, goal, ordered steps, files affected, testing, and risks.
* [Interface Contract](interface-contract.md) - the interfaces build codes against — data model and API, module boundaries, key flows, and cross-cutting concerns — each in its native language, or TypeSpec.
* [ADR](adr.md) - architecture decision record — context, the decision, options considered, and consequences. Filed as a Decision concept in `knowledge/decisions/`.
* [Test Plan](test-plan.md) - scope, risks driving the plan, automated and manual cases, regression coverage, and exit criteria. Appended to the spec concept.

## Change delivery

* [Commit Message](commit-message.md) - conventional-commit shape — typed summary line, why-not-what body, and breaking-change footer.
* [PR Description](pr-description.md) - summary, motivation, grouped changes, test plan, and notes for reviewers.
* [Code Review](code-review.md) - verdict first, findings ranked by severity with concrete failure scenarios, and what was checked and found fine.
* [Security Review](security-review.md) - risk verdict, scope and threat model, findings with attack scenarios, and areas checked sound.

## Release & migration

* [Changelog](changelog.md) - Keep-a-Changelog skeleton — Unreleased plus per-release Added/Changed/Fixed sections and compare links.
* [Release Notes](release-notes.md) - headline, highlights, breaking changes with migration steps, fixes, and the upgrade command.
* [Migration Guide](migration-guide.md) - old-to-new steps with per-step verification, behavioural differences, rollback, and troubleshooting.

## Reports & analysis

* [Bug Report](bug-report.md) - symptom, environment, exact repro steps, expected vs actual, root cause, and regression risk. One of the three shapes the issue tracker holds.
* [Feature Request](feature-request.md) - motivation, proposed behaviour, alternatives considered and scope. One of the three shapes the issue tracker holds.
* [Chore](chore.md) - the surfaces scoped mechanical work reaches and the check that says it is done. One of the three shapes the issue tracker holds.
* [Investigation](investigation.md) - conclusion-first write-up — question, evidence with sources, ruled-out hypotheses, and recommendation.
* [Postmortem](postmortem.md) - blameless incident write-up — impact, timeline, root cause, and typed action items.
* [Status Update](status-update.md) - TL;DR, done since last update, in progress, blockers with recommended defaults, and next.

## Project files

* [README](readme.md) - install, quick start, usage, configuration, and the development loop for a project front page.

## Knowledge concepts

Skeletons for the knowledge concepts, mirroring the knowledge index —
each carries the concept's frontmatter, ready to fill and file.

* [Project Overview](project-overview.md) - what the project is, for whom, and its current status.
* [Glossary](glossary.md) - the domain terms the project's code and docs assume, one definition each.
* [Constraints & Non-Goals](constraints-non-goals.md) - what the project deliberately does not do, and the accepted limitations.
* [Backlog](backlog.md) - ideas under consideration and ideas decided against, with the reasoning.
* [Architecture](architecture.md) - the system's layers, the key subsystems, and how they fit together.
* [Architectural Rules](architectural-rules.md) - the invariants behind the architecture, each with its reason.
* [Software Components](software-components.md) - the deliverables and the CI/CD that builds them.
* [Configuration](configuration.md) - the config files and stores, their shape, and what lives outside the repo.
* [API Contracts](api-contracts.md) - the public surfaces, their contracts, and the stability promises.
* [Error Handling](error-handling.md) - the error taxonomy or exit codes, and the failure-reporting rules.
* [Directory Structure](directory-structure.md) - what lives where in the repository.
* [Technology Stack](technology-stack.md) - languages, dependencies with their reasons, and the pinned toolchain.
* [Visual System](visual-system.md) - the design tokens the UI is built against — palette, type roles, spacing, signature, and component library.
* [Coding Standards](coding-standards.md) - the behavioural rules, prose rules, and per-language conventions.
* [Security Requirements](security-requirements.md) - the vulnerability policy in brief, and the design's guarantees.
* [Dependency Policy](dependency-policy.md) - when a dependency may be added and how its version is chosen.
* [Testing Strategy](testing-strategy.md) - the test layers, the key choices behind them, and where they run.
* [Development Procedure](development-procedure.md) - setup, the change workflow, and what must pass before a PR.
* [Development Commands](development-commands.md) - the everyday command set and the traps in it.
* [Issue Tracker](issue-tracker.md) - where issues live, the filing conventions, and the triage labels.
* [Definition of Done](definition-of-done.md) - what a change must satisfy before it merges.
* [Release Procedure](release-procedure.md) - how a release is cut, the gates, and the irreversible steps.
