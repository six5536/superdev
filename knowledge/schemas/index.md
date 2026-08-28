# Schemas

The structural contract for every document the development process produces:
what sections it must carry, what its frontmatter must say, and a worked example
that satisfies the contract. A schema is checkable — the tool enforces it — where
a template could only be copied.

## Planning & design

* [Spec Schema](spec.md) - feature specs filed in knowledge/specs/ — the spec body plus the appended test plan.
* [Feature Plan Schema](feature-plan.md) - the feature's slice list — per slice a done-check, its test-plan cases and a done marker — filed in knowledge/plans/.
* [Ad-hoc Plan Schema](adhoc-plan.md) - implementation plans for one-off work outside the feature workflow, filed in knowledge/plans/.
* [ADR Schema](adr.md) - architecture decision records — context, the decision, options considered and consequences — filed in knowledge/decisions/.

## Contracts

One document per contract. The interface contract is private to a feature and discarded once the code is canonical; the rest are promised outward and live in knowledge/contracts/public/.

* [Interface Contract Schema](contract-interface.md) - the interfaces build codes against — data model, module boundaries, key flows — filed in knowledge/contracts/private/.
* [CLI Contract Schema](contract-cli.md) - one command-line surface — its commands, their behaviour, the exit codes and the stability promise, in knowledge/contracts/public/.
* [REST Contract Schema](contract-rest.md) - one HTTP API — its endpoints in TypeSpec, the authentication, the error responses and the stability promise, in knowledge/contracts/public/.
* [GraphQL Contract Schema](contract-graphql.md) - one GraphQL API — its SDL, endpoint, error and limit behaviour, and the stability promise, in knowledge/contracts/public/.
* [RPC Contract Schema](contract-rpc.md) - one RPC service — its IDL, transport, authentication, error codes and stability promise, in knowledge/contracts/public/.
* [MCP Contract Schema](contract-mcp.md) - one MCP server — its transport, the tools it exposes, how failures are reported and the stability promise, in knowledge/contracts/public/.
* [Library Contract Schema](contract-library.md) - one published library — what ships, its exported API in the host language, its errors and the stability promise, in knowledge/contracts/public/.
* [UI Contract Schema](contract-ui.md) - the user-facing surface — its routes, its screens and their states, the platforms supported, and the stability promise, in knowledge/contracts/public/.
* [Event Contract Schema](contract-events.md) - one published message or event stream — its transport, payloads, delivery guarantees and stability promise, in knowledge/contracts/public/.
* [Data Contract Schema](contract-data.md) - the persisted store — its schema, the constraints it holds, how it migrates, and the stability promise, in knowledge/contracts/public/.
* [Configuration Contract Schema](contract-config.md) - what a deployer must supply to run the software — the settings, where they come from, which source wins, and the stability promise, in knowledge/contracts/public/.
* [File Format Contract Schema](contract-file-format.md) - one file others read or write — where it lives, its shape, how a reader treats the unexpected, and the stability promise, in knowledge/contracts/public/.
* [Deployment Contract Schema](contract-deployment.md) - what a deployer must provide to run the software — the artifact, the runtime it needs, its health and lifecycle, and the stability promise, in knowledge/contracts/public/.
* [Authorisation Contract Schema](contract-authz.md) - what a caller may do — the principals, the role and scope vocabulary, the permissions and the boundaries every surface enforces, in knowledge/contracts/public/.
* [Telemetry Contract Schema](contract-telemetry.md) - the signal operators build on — the metrics, the log shape, the traces, and the stability promise, in knowledge/contracts/public/.

## Change delivery

* [Code Review Schema](code-review.md) - code review findings — verdict first, findings ranked by severity with concrete failure scenarios.
* [Security Review Schema](security-review.md) - security reviews — risk verdict, scope and threat model, findings with attack scenarios, and areas checked sound.

## Release & migration

* [Changelog Schema](changelog.md) - CHANGELOG.md — Keep-a-Changelog, with Unreleased plus per-release change groups and compare links.
* [Release Notes Schema](release-notes.md) - release notes — headline, highlights, breaking changes with migration steps, fixes and the upgrade command.
* [Migration Guide Schema](migration-guide.md) - migration guides — old-to-new steps with per-step verification, behavioural differences, rollback and troubleshooting.

## Reports & analysis

* [Bug Report Schema](bug-report.md) - bug reports filed as Issue concepts in knowledge/issues/ — symptom, repro, root cause and regression risk.
* [Feature Request Schema](feature-request.md) - feature requests filed in knowledge/issues/ — motivation, proposed behaviour, alternatives and scope, with no room for invented repro steps.
* [Chore Schema](chore.md) - scoped mechanical work filed in knowledge/issues/ — the surfaces it touches and what done means, with no room for a root cause it does not have.
* [Investigation Schema](investigation.md) - investigation write-ups — conclusion first, evidence with sources, ruled-out hypotheses and a recommendation.
* [Postmortem Schema](postmortem.md) - blameless incident write-ups — impact, timeline, root cause and typed action items.
* [Status Update Schema](status-update.md) - status updates — TL;DR, done since last update, in progress, blockers with recommended defaults, and next.
* [Research Schema](research.md) - research findings filed as footnote-cited concepts in knowledge/research/, derived from the research skill and the SOKF spec.

## Project files

* [README Schema](readme.md) - README.md — install, quick start, usage, configuration and the development loop.

## Knowledge concepts

* [Project Overview Schema](project-overview.md) - what the project is, for whom, and its current status, in knowledge/project-overview.md.
* [Architecture Schema](architecture.md) - the system's layers, its subsystems, and the files it reads and writes, in knowledge/architecture.md.
* [Architectural Rules Schema](architectural-rules.md) - the invariants behind the architecture, each with its reason, in knowledge/architectural-rules.md.
* [Software Components Schema](software-components.md) - the deliverables and the CI/CD that builds them, in knowledge/software-components.md.
* [Configuration Schema](configuration.md) - the config files and stores, their shape, and what lives outside the repo, in knowledge/configuration.md.
* [Directory Structure Schema](directory-structure.md) - what lives where in the repository, in knowledge/directory-structure.md.
* [Technology Stack Schema](technology-stack.md) - languages, dependencies with their reasons, and the pinned toolchain, in knowledge/technology-stack.md.
* [Dependency Policy Schema](dependency-policy.md) - when a dependency may be added and how its version is chosen, in knowledge/dependency-policy.md.
* [Coding Standards Schema](coding-standards.md) - the behavioural rules, prose rules and per-language conventions CI enforces, in knowledge/coding-standards.md.
* [Testing Strategy Schema](testing-strategy.md) - the test layers, the key choices behind them, and where they run, in knowledge/testing-strategy.md.
* [Error Handling Schema](error-handling.md) - the error taxonomy or exit codes and the failure-reporting rules callers rely on, in knowledge/error-handling.md.
* [Security Requirements Schema](security-requirements.md) - the vulnerability policy in brief and the security guarantees the design makes, in knowledge/security-requirements.md.
* [Development Commands Schema](development-commands.md) - the everyday command set and the traps in it, in knowledge/development-commands.md.
* [Development Procedure Schema](development-procedure.md) - setup, the change workflow, and what must pass before a PR, in knowledge/development-procedure.md.
* [Release Procedure Schema](release-procedure.md) - how a release is cut, the gates on it, and the steps that cannot be undone, in knowledge/release-procedure.md.
* [Definition of Done Schema](definition-of-done.md) - the gates a change must satisfy before it merges, in knowledge/definition-of-done.md.
* [Issue Tracker Schema](issue-tracker.md) - where issues live, the filing conventions and the triage label vocabulary, in knowledge/issue-tracker.md.
* [Glossary Schema](glossary.md) - the domain terms the project's code and docs assume, one definition each, in knowledge/glossary.md.
* [Constraints & Non-Goals Schema](constraints-non-goals.md) - what the project deliberately does not do and the limitations it accepts, in knowledge/constraints-non-goals.md.
* [Backlog Schema](backlog.md) - ideas under consideration and ideas decided against, with the reasoning, in knowledge/backlog.md.
* [Visual System Schema](visual-system.md) - the design tokens later slices build against, in knowledge/visual-system.md.

## The index itself

* [Templates Index Schema](templates-index.md) - the grouped listing of every template with its one-line summary, in knowledge/templates/index.md.
