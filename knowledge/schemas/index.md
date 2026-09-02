# Schemas

The structural contract for every document the development process produces:
what sections it must carry, what its frontmatter must say, and a worked example
that satisfies the contract. A schema is checkable — the tool enforces it — where
a template could only be copied.

## Planning & design

* [Feature Plan Schema][sokf:schema-feature-plan] - the feature's slice list — per slice its dependencies, a done-check, its test-plan cases and a done marker — filed among the plans.
* [Ad-hoc Plan Schema][sokf:schema-adhoc-plan] - implementation plans for one-off work outside the feature workflow, filed among the plans.
* [Idea Schema][sokf:schema-idea] - a thought captured for considering later — what it is, plus whatever reasoning exists at the time — filed in knowledge/ideas/.
* [ADR Schema][sokf:schema-adr] - architecture decision records — context, the decision, options considered and consequences — filed among the ADRs.

## Contracts

One document per contract, all durable. An interface contract is internal — keyed to a module boundary, updated as features change it; the rest are promised outward as public contracts.

* [Interface Contract Schema][sokf:schema-contract-interface] - the interfaces build codes against — data model, module boundaries, key flows — an internal contract, durable and keyed to the interface.
* [Contract Schema][sokf:schema-contract] - one interface the software offers or depends on — its definition materialised from source, the behaviour the definition cannot say, and the stability promise — for every kind of contract.
* [CLI Contract Schema][sokf:schema-contract-cli] - one command-line surface — its commands, their behaviour, the exit codes and the stability promise, a public contract.
* [REST Contract Schema][sokf:schema-contract-rest] - one HTTP API — its endpoints in TypeSpec, the authentication, the error responses and the stability promise, a public contract.
* [GraphQL Contract Schema][sokf:schema-contract-graphql] - one GraphQL API — its SDL, endpoint, error and limit behaviour, and the stability promise, a public contract.
* [RPC Contract Schema][sokf:schema-contract-rpc] - one RPC service — its IDL, transport, authentication, error codes and stability promise, a public contract.
* [MCP Contract Schema][sokf:schema-contract-mcp] - one MCP server — its transport, the tools it exposes, how failures are reported and the stability promise, a public contract.
* [Library Contract Schema][sokf:schema-contract-library] - one published library — what ships, its exported API in the host language, its errors and the stability promise, a public contract.
* [UI Contract Schema][sokf:schema-contract-ui] - the user-facing surface — its routes, its screens and their states, the platforms supported, and the stability promise, a public contract.
* [Event Contract Schema][sokf:schema-contract-events] - one published message or event stream — its transport, payloads, delivery guarantees and stability promise, a public contract.
* [Data Contract Schema][sokf:schema-contract-data] - the persisted store — its schema, the constraints it holds, how it migrates, and the stability promise, a public contract.
* [Configuration Contract Schema][sokf:schema-contract-config] - what a deployer must supply to run the software — the settings, where they come from, which source wins, and the stability promise, a public contract.
* [Binary Format Contract Schema][sokf:schema-contract-binary-format] - one binary file others read or write — its magic number and version, the byte layout of every field, how a reader treats the unexpected, and the stability promise, a public contract.
* [Text Format Contract Schema][sokf:schema-contract-text-format] - one text file others read or write — where it lives, its shape as a schema or a worked example, how a reader treats the unexpected, and the stability promise, a public contract.
* [Deployment Contract Schema][sokf:schema-contract-deployment] - what a deployer must provide to run the software — the artifact, the runtime it needs, its health and lifecycle, and the stability promise, a public contract.
* [Authorisation Contract Schema][sokf:schema-contract-authz] - what a caller may do — the principals, the role and scope vocabulary, the permissions and the boundaries every surface enforces, a public contract.
* [Telemetry Contract Schema][sokf:schema-contract-telemetry] - the signal operators build on — the metrics, the log shape, the traces, and the stability promise, a public contract.

## Change delivery

* [Code Review Schema][sokf:schema-code-review] - code review findings — verdict first, findings ranked by severity with concrete failure scenarios.
* [Security Review Schema][sokf:schema-security-review] - security reviews — risk verdict, scope and threat model, findings with attack scenarios, and areas checked sound.

## Release & migration

* [Changelog Schema][sokf:schema-changelog] - CHANGELOG.md — Keep-a-Changelog, with Unreleased plus per-release change groups and compare links.
* [Release Notes Schema][sokf:schema-release-notes] - release notes — headline, highlights, breaking changes with migration steps, fixes and the upgrade command.
* [Migration Guide Schema][sokf:schema-migration-guide] - migration guides — old-to-new steps with per-step verification, behavioural differences, rollback and troubleshooting.

## Reports & analysis

* [Bug Report Schema][sokf:schema-bug-report] - bug reports filed in the issue tracker — symptom, repro, root cause and regression risk.
* [Feature Request Schema][sokf:schema-feature-request] - feature requests filed in the issue tracker — motivation, proposed behaviour, EARS acceptance criteria, alternatives and scope, with no room for invented repro steps.
* [Chore Schema][sokf:schema-chore] - scoped mechanical work filed in the issue tracker — the surfaces it touches and what done means, with no room for a root cause it does not have.
* [Investigation Schema][sokf:schema-investigation] - investigation write-ups — conclusion first, evidence with sources, ruled-out hypotheses and a recommendation.
* [Postmortem Schema][sokf:schema-postmortem] - blameless incident write-ups — impact, timeline, root cause and typed action items.
* [Status Update Schema][sokf:schema-status-update] - status updates — TL;DR, done since last update, in progress, blockers with recommended defaults, and next.
* [Research Schema][sokf:schema-research] - research findings filed as footnote-cited concepts in knowledge/research/, derived from the research skill and the SOKF spec.

## Project files

* [README Schema][sokf:schema-readme] - README.md — install, quick start, usage, configuration and the development loop.

## Knowledge concepts

* [Project Overview Schema][sokf:schema-project-overview] - what the project is, for whom, and its current status, in knowledge/project-overview.md.
* [Architecture Schema][sokf:schema-architecture] - the system's layers, its subsystems, and the files it reads and writes, in knowledge/architecture.md.
* [Architectural Rules Schema][sokf:schema-architectural-rules] - the invariants behind the architecture, each with its reason, in knowledge/architectural-rules.md.
* [Software Components Schema][sokf:schema-software-components] - the deliverables and the CI/CD that builds them, in knowledge/software-components.md.
* [Configuration Schema][sokf:schema-configuration] - the config files and stores, their shape, and what lives outside the repo, in knowledge/configuration.md.
* [Directory Structure Schema][sokf:schema-directory-structure] - what lives where in the repository, in knowledge/directory-structure.md.
* [Technology Stack Schema][sokf:schema-technology-stack] - languages, dependencies with their reasons, and the pinned toolchain, in knowledge/technology-stack.md.
* [Dependency Policy Schema][sokf:schema-dependency-policy] - when a dependency may be added and how its version is chosen, in knowledge/dependency-policy.md.
* [Coding Standards Schema][sokf:schema-coding-standards] - the behavioural rules, prose rules and per-language conventions CI enforces, in knowledge/coding-standards.md.
* [Testing Strategy Schema][sokf:schema-testing-strategy] - the test layers, the key choices behind them, and where they run, in knowledge/testing-strategy.md.
* [Error Handling Schema][sokf:schema-error-handling] - the error taxonomy or exit codes and the failure-reporting rules callers rely on, in knowledge/error-handling.md.
* [Security Requirements Schema][sokf:schema-security-requirements] - the vulnerability policy in brief and the security guarantees the design makes, in knowledge/security-requirements.md.
* [Development Commands Schema][sokf:schema-development-commands] - the everyday command set and the traps in it, in knowledge/development-commands.md.
* [Development Procedure Schema][sokf:schema-development-procedure] - setup, the change workflow, and what must pass before a PR, in knowledge/development-procedure.md.
* [Release Procedure Schema][sokf:schema-release-procedure] - how a release is cut, the gates on it, and the steps that cannot be undone, in knowledge/release-procedure.md.
* [Definition of Done Schema][sokf:schema-definition-of-done] - the gates a change must satisfy before it merges, in knowledge/definition-of-done.md.
* [Issue Tracker Schema][sokf:schema-issue-tracker] - where issues live, the filing conventions and the triage label vocabulary, in knowledge/issue-tracker.md.
* [Glossary Schema][sokf:schema-glossary] - the domain terms the project's code and docs assume, one definition each, in knowledge/glossary.md.
* [Constraints & Non-Goals Schema][sokf:schema-constraints-non-goals] - what the project deliberately does not do and the limitations it accepts, in knowledge/constraints-non-goals.md.
* [Backlog Schema][sokf:schema-backlog] - ideas under consideration and ideas decided against, with the reasoning, in knowledge/backlog.md.
* [Visual System Schema][sokf:schema-visual-system] - the design tokens later slices build against, in knowledge/visual-system.md.

## The index itself

* [Schemas Index Schema][sokf:schema-schemas-index] - the grouped listing of every schema with its one-line summary, in knowledge/schemas/index.md.

<!-- sokf:links -->
[sokf:schema-adhoc-plan]: /knowledge/schemas/adhoc-plan.md
[sokf:schema-adr]: /knowledge/schemas/adr.md
[sokf:schema-architectural-rules]: /knowledge/schemas/architectural-rules.md
[sokf:schema-architecture]: /knowledge/schemas/architecture.md
[sokf:schema-backlog]: /knowledge/schemas/backlog.md
[sokf:schema-bug-report]: /knowledge/schemas/bug-report.md
[sokf:schema-changelog]: /knowledge/schemas/changelog.md
[sokf:schema-chore]: /knowledge/schemas/chore.md
[sokf:schema-code-review]: /knowledge/schemas/code-review.md
[sokf:schema-coding-standards]: /knowledge/schemas/coding-standards.md
[sokf:schema-configuration]: /knowledge/schemas/configuration.md
[sokf:schema-constraints-non-goals]: /knowledge/schemas/constraints-non-goals.md
[sokf:schema-contract]: /knowledge/schemas/contract.md
[sokf:schema-contract-authz]: /knowledge/schemas/contract-authz.md
[sokf:schema-contract-binary-format]: /knowledge/schemas/contract-binary-format.md
[sokf:schema-contract-cli]: /knowledge/schemas/contract-cli.md
[sokf:schema-contract-config]: /knowledge/schemas/contract-config.md
[sokf:schema-contract-data]: /knowledge/schemas/contract-data.md
[sokf:schema-contract-deployment]: /knowledge/schemas/contract-deployment.md
[sokf:schema-contract-events]: /knowledge/schemas/contract-events.md
[sokf:schema-contract-graphql]: /knowledge/schemas/contract-graphql.md
[sokf:schema-contract-interface]: /knowledge/schemas/contract-interface.md
[sokf:schema-contract-library]: /knowledge/schemas/contract-library.md
[sokf:schema-contract-mcp]: /knowledge/schemas/contract-mcp.md
[sokf:schema-contract-rest]: /knowledge/schemas/contract-rest.md
[sokf:schema-contract-rpc]: /knowledge/schemas/contract-rpc.md
[sokf:schema-contract-telemetry]: /knowledge/schemas/contract-telemetry.md
[sokf:schema-contract-text-format]: /knowledge/schemas/contract-text-format.md
[sokf:schema-contract-ui]: /knowledge/schemas/contract-ui.md
[sokf:schema-definition-of-done]: /knowledge/schemas/definition-of-done.md
[sokf:schema-dependency-policy]: /knowledge/schemas/dependency-policy.md
[sokf:schema-development-commands]: /knowledge/schemas/development-commands.md
[sokf:schema-development-procedure]: /knowledge/schemas/development-procedure.md
[sokf:schema-directory-structure]: /knowledge/schemas/directory-structure.md
[sokf:schema-error-handling]: /knowledge/schemas/error-handling.md
[sokf:schema-feature-plan]: /knowledge/schemas/feature-plan.md
[sokf:schema-feature-request]: /knowledge/schemas/feature-request.md
[sokf:schema-glossary]: /knowledge/schemas/glossary.md
[sokf:schema-idea]: /knowledge/schemas/idea.md
[sokf:schema-investigation]: /knowledge/schemas/investigation.md
[sokf:schema-issue-tracker]: /knowledge/schemas/issue-tracker.md
[sokf:schema-migration-guide]: /knowledge/schemas/migration-guide.md
[sokf:schema-postmortem]: /knowledge/schemas/postmortem.md
[sokf:schema-project-overview]: /knowledge/schemas/project-overview.md
[sokf:schema-readme]: /knowledge/schemas/readme.md
[sokf:schema-release-notes]: /knowledge/schemas/release-notes.md
[sokf:schema-release-procedure]: /knowledge/schemas/release-procedure.md
[sokf:schema-research]: /knowledge/schemas/research.md
[sokf:schema-schemas-index]: /knowledge/schemas/schemas-index.md
[sokf:schema-security-requirements]: /knowledge/schemas/security-requirements.md
[sokf:schema-security-review]: /knowledge/schemas/security-review.md
[sokf:schema-software-components]: /knowledge/schemas/software-components.md
[sokf:schema-status-update]: /knowledge/schemas/status-update.md
[sokf:schema-technology-stack]: /knowledge/schemas/technology-stack.md
[sokf:schema-testing-strategy]: /knowledge/schemas/testing-strategy.md
[sokf:schema-visual-system]: /knowledge/schemas/visual-system.md
