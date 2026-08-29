# Templates

Copy-verbatim skeletons for the documents the development process produces.
Read one with `sokf_read` (id `template-<name>`), strip the frontmatter, and
fill in the angle-bracket placeholders.

## Planning & design

* [Spec][sokf:template-spec] - what done looks like from outside — behaviour, acceptance criteria, UI states, edge cases, out of scope. Filed as a draft concept in `knowledge/specs/`, tagged done at accept.
* [Feature Plan][sokf:template-feature-plan] - the feature's slice list — per slice a done-check, the assigned test-plan cases, and a done marker. Produced by the feature-plan phase.
* [Ad-hoc Plan][sokf:template-adhoc-plan] - ad-hoc implementation plan for one-off work outside the feature workflow — context, goal, ordered steps, files affected, testing, and risks.
* [ADR][sokf:template-adr] - architecture decision record — context, the decision, options considered, and consequences. Filed as a Decision concept in `knowledge/decisions/`.
* [Test Plan][sokf:template-test-plan] - scope, risks driving the plan, automated and manual cases, regression coverage, and exit criteria. Appended to the spec concept.

## Contracts

One document per contract — the interface contract private to a feature, the rest promised outward.

* [Interface Contract][sokf:template-contract-interface] - the interfaces build codes against — data model and API, module boundaries, key flows, and cross-cutting concerns — each in its native language, or TypeSpec.
* [CLI Contract][sokf:template-contract-cli] - one command-line surface, its behaviour, exit codes and stability promise.
* [REST Contract][sokf:template-contract-rest] - one HTTP API in TypeSpec, its authentication, errors and stability promise.
* [GraphQL Contract][sokf:template-contract-graphql] - one graph in SDL, its endpoint, errors, limits and deprecation policy.
* [RPC Contract][sokf:template-contract-rpc] - one RPC service in its IDL, its transport, errors and wire compatibility.
* [MCP Contract][sokf:template-contract-mcp] - one MCP server, the tools it exposes, its errors and stability promise.
* [Library Contract][sokf:template-contract-library] - one published library, its exported API, errors and stability promise.
* [UI Contract][sokf:template-contract-ui] - the routes, the screens and their states, and what is promised not to move.
* [Event Contract][sokf:template-contract-events] - one published message stream, its payloads, delivery guarantees and stability promise.
* [Data Contract][sokf:template-contract-data] - the persisted store, its schema, the constraints it holds and how it migrates.
* [Configuration Contract][sokf:template-contract-config] - the settings a deployer supplies, where they come from, which source wins.
* [File Format Contract][sokf:template-contract-file-format] - one file others read or write, its shape, compatibility rules and stability promise.
* [Deployment Contract][sokf:template-contract-deployment] - what is published, what the runtime must provide, and how it starts and stops.
* [Authorisation Contract][sokf:template-contract-authz] - the principals, the role and scope vocabulary, the permissions and the boundary every surface enforces.
* [Telemetry Contract][sokf:template-contract-telemetry] - the metrics, the log shape and the traces operators build on.

## Change delivery

* [Commit Message][sokf:template-commit-message] - conventional-commit shape — typed summary line, why-not-what body, and breaking-change footer.
* [PR Description][sokf:template-pr-description] - summary, motivation, grouped changes, test plan, and notes for reviewers.
* [Code Review][sokf:template-code-review] - verdict first, findings ranked by severity with concrete failure scenarios, and what was checked and found fine.
* [Security Review][sokf:template-security-review] - risk verdict, scope and threat model, findings with attack scenarios, and areas checked sound.

## Release & migration

* [Changelog][sokf:template-changelog] - Keep-a-Changelog skeleton — Unreleased plus per-release Added/Changed/Fixed sections and compare links.
* [Release Notes][sokf:template-release-notes] - headline, highlights, breaking changes with migration steps, fixes, and the upgrade command.
* [Migration Guide][sokf:template-migration-guide] - old-to-new steps with per-step verification, behavioural differences, rollback, and troubleshooting.

## Reports & analysis

* [Bug Report][sokf:template-bug-report] - symptom, environment, exact repro steps, expected vs actual, root cause, and regression risk. One of the three shapes the issue tracker holds.
* [Feature Request][sokf:template-feature-request] - motivation, proposed behaviour, alternatives considered and scope. One of the three shapes the issue tracker holds.
* [Chore][sokf:template-chore] - the surfaces scoped mechanical work reaches and the check that says it is done. One of the three shapes the issue tracker holds.
* [Investigation][sokf:template-investigation] - conclusion-first write-up — question, evidence with sources, ruled-out hypotheses, and recommendation.
* [Postmortem][sokf:template-postmortem] - blameless incident write-up — impact, timeline, root cause, and typed action items.
* [Status Update][sokf:template-status-update] - TL;DR, done since last update, in progress, blockers with recommended defaults, and next.

## Project files

* [README][sokf:template-readme] - install, quick start, usage, configuration, and the development loop for a project front page.

## Knowledge concepts

Skeletons for the knowledge concepts, mirroring the knowledge index —
each carries the concept's frontmatter, ready to fill and file.

* [Project Overview][sokf:template-project-overview] - what the project is, for whom, and its current status.
* [Glossary][sokf:template-glossary] - the domain terms the project's code and docs assume, one definition each.
* [Constraints & Non-Goals][sokf:template-constraints-non-goals] - what the project deliberately does not do, and the accepted limitations.
* [Backlog][sokf:template-backlog] - ideas under consideration and ideas decided against, with the reasoning.
* [Architecture][sokf:template-architecture] - the system's layers, the key subsystems, and how they fit together.
* [Architectural Rules][sokf:template-architectural-rules] - the invariants behind the architecture, each with its reason.
* [Software Components][sokf:template-software-components] - the deliverables and the CI/CD that builds them.
* [Configuration][sokf:template-configuration] - the config files and stores, their shape, and what lives outside the repo.
* [Error Handling][sokf:template-error-handling] - the error taxonomy or exit codes, and the failure-reporting rules.
* [Directory Structure][sokf:template-directory-structure] - what lives where in the repository.
* [Technology Stack][sokf:template-technology-stack] - languages, dependencies with their reasons, and the pinned toolchain.
* [Visual System][sokf:template-visual-system] - the design tokens the UI is built against — palette, type roles, spacing, signature, and component library.
* [Coding Standards][sokf:template-coding-standards] - the behavioural rules, prose rules, and per-language conventions.
* [Security Requirements][sokf:template-security-requirements] - the vulnerability policy in brief, and the design's guarantees.
* [Dependency Policy][sokf:template-dependency-policy] - when a dependency may be added and how its version is chosen.
* [Testing Strategy][sokf:template-testing-strategy] - the test layers, the key choices behind them, and where they run.
* [Development Procedure][sokf:template-development-procedure] - setup, the change workflow, and what must pass before a PR.
* [Development Commands][sokf:template-development-commands] - the everyday command set and the traps in it.
* [Issue Tracker][sokf:template-issue-tracker] - where issues live, the filing conventions, and the triage labels.
* [Definition of Done][sokf:template-definition-of-done] - what a change must satisfy before it merges.
* [Release Procedure][sokf:template-release-procedure] - how a release is cut, the gates, and the irreversible steps.

<!-- sokf:links -->
[sokf:template-adhoc-plan]: /knowledge/templates/adhoc-plan.md
[sokf:template-adr]: /knowledge/templates/adr.md
[sokf:template-architectural-rules]: /knowledge/templates/architectural-rules.md
[sokf:template-architecture]: /knowledge/templates/architecture.md
[sokf:template-backlog]: /knowledge/templates/backlog.md
[sokf:template-bug-report]: /knowledge/templates/bug-report.md
[sokf:template-changelog]: /knowledge/templates/changelog.md
[sokf:template-chore]: /knowledge/templates/chore.md
[sokf:template-code-review]: /knowledge/templates/code-review.md
[sokf:template-coding-standards]: /knowledge/templates/coding-standards.md
[sokf:template-commit-message]: /knowledge/templates/commit-message.md
[sokf:template-configuration]: /knowledge/templates/configuration.md
[sokf:template-constraints-non-goals]: /knowledge/templates/constraints-non-goals.md
[sokf:template-contract-authz]: /knowledge/templates/contract-authz.md
[sokf:template-contract-cli]: /knowledge/templates/contract-cli.md
[sokf:template-contract-config]: /knowledge/templates/contract-config.md
[sokf:template-contract-data]: /knowledge/templates/contract-data.md
[sokf:template-contract-deployment]: /knowledge/templates/contract-deployment.md
[sokf:template-contract-events]: /knowledge/templates/contract-events.md
[sokf:template-contract-file-format]: /knowledge/templates/contract-file-format.md
[sokf:template-contract-graphql]: /knowledge/templates/contract-graphql.md
[sokf:template-contract-interface]: /knowledge/templates/contract-interface.md
[sokf:template-contract-library]: /knowledge/templates/contract-library.md
[sokf:template-contract-mcp]: /knowledge/templates/contract-mcp.md
[sokf:template-contract-rest]: /knowledge/templates/contract-rest.md
[sokf:template-contract-rpc]: /knowledge/templates/contract-rpc.md
[sokf:template-contract-telemetry]: /knowledge/templates/contract-telemetry.md
[sokf:template-contract-ui]: /knowledge/templates/contract-ui.md
[sokf:template-definition-of-done]: /knowledge/templates/definition-of-done.md
[sokf:template-dependency-policy]: /knowledge/templates/dependency-policy.md
[sokf:template-development-commands]: /knowledge/templates/development-commands.md
[sokf:template-development-procedure]: /knowledge/templates/development-procedure.md
[sokf:template-directory-structure]: /knowledge/templates/directory-structure.md
[sokf:template-error-handling]: /knowledge/templates/error-handling.md
[sokf:template-feature-plan]: /knowledge/templates/feature-plan.md
[sokf:template-feature-request]: /knowledge/templates/feature-request.md
[sokf:template-glossary]: /knowledge/templates/glossary.md
[sokf:template-investigation]: /knowledge/templates/investigation.md
[sokf:template-issue-tracker]: /knowledge/templates/issue-tracker.md
[sokf:template-migration-guide]: /knowledge/templates/migration-guide.md
[sokf:template-postmortem]: /knowledge/templates/postmortem.md
[sokf:template-pr-description]: /knowledge/templates/pr-description.md
[sokf:template-project-overview]: /knowledge/templates/project-overview.md
[sokf:template-readme]: /knowledge/templates/readme.md
[sokf:template-release-notes]: /knowledge/templates/release-notes.md
[sokf:template-release-procedure]: /knowledge/templates/release-procedure.md
[sokf:template-security-requirements]: /knowledge/templates/security-requirements.md
[sokf:template-security-review]: /knowledge/templates/security-review.md
[sokf:template-software-components]: /knowledge/templates/software-components.md
[sokf:template-spec]: /knowledge/templates/spec.md
[sokf:template-status-update]: /knowledge/templates/status-update.md
[sokf:template-technology-stack]: /knowledge/templates/technology-stack.md
[sokf:template-test-plan]: /knowledge/templates/test-plan.md
[sokf:template-testing-strategy]: /knowledge/templates/testing-strategy.md
[sokf:template-visual-system]: /knowledge/templates/visual-system.md
