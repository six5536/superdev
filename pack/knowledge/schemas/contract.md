---
type: Schema
id: schema-contract
title: Contract Schema
description: One interface the software offers or depends on — its definition materialised from source, the behaviour the definition cannot say, and the stability promise — for every kind of contract.
---

# Contract Schema

Structural rules for contract documents, filed as
`contract-{nnn}-{kind}-{slug}`, numbered after the highest across every
contract folder — a duplicate number is an error — and placed in their
lifecycle folder by `superdev validate --fix`. One schema governs every
kind (ADR-043);
the kind is in the frontmatter and the id, and the checklist below says
what each kind's Behaviour must cover.

A contract is the outside of the black box: the one place a person or
an agent reads what an interface promises without reading the code.
Its Definition is materialised from the source that declares the
interface (ADR-042), so it is readable in one place and cannot drift. Its Behaviour says
what the definition cannot. Its Stability says what may change.

<!-- sokf:include contract-style -->
**Contract style — a contract defines its interface** (superdev
ADR-033, ADR-042, ADR-043, ADR-044):

- A contract's Definition MUST be one or more source includes of the
  regions that declare the interface, and MUST NOT carry an authored
  block; a caller reads the interface from the contract and reproduces
  it from the source the contract carries.
- A region MUST be bounded by `sokf:begin <name>` and `sokf:end <name>`
  in the source's own comment syntax. What is not marked is not
  promised.
- A doc comment inside an included region is contract text: a MUST
  there binds as a MUST in Behaviour does.
- Prose MUST describe and MUST NOT define. Behaviour MUST carry what no
  single element can say and what no include reaches — stability,
  consumers, behaviour across elements, exit codes, error semantics —
  each normative statement with an RFC 2119 modal verb, one requirement
  per sentence.
- Behaviour MUST cover what the schema's checklist names for the
  contract's kind, one `###` per item that applies.
- A contract MUST bind what it names and MUST NOT state how the
  interface is built inside.
- The Definition is bound by its include. The project MUST bind each
  Behaviour promise by a test of the behaviour it promises.
- A built-from source unreadable as a surface MUST be rendered by a
  generator that writes `sokf:generated-by <what>` in the rendering's
  leading lines, and the rendering MUST be proved current by a test.
- A Behaviour or Stability statement whose behaviour is unbuilt MAY
  carry `PENDING` in uppercase beside its modal verb, naming the issue
  or plan slice in parentheses, and MUST NOT once the feature settles; a
  definition element carries none.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.
<!-- /sokf:include -->

## What Behaviour must cover, by kind

The Definition says what the interface *is*. Behaviour says what the
definition cannot: the promises across elements, the failures, the
limits, what may change. One `###` subsection per bullet below. A
bullet marked **required** is a section every contract of the kind
carries; the rest are carried where they apply and omitted otherwise,
never written as "not applicable". A promise a contract needs that no
bullet names is added here, so the next writer of the kind sees it.

PENDING (I049): these bullets become section rules tagged with their
kinds (ADR-045), the
required ones enforced, each bullet's text as its rule's
`description`, and this prose goes.

### api

- **required** Where it is served, and how a caller reaches it — host, path, port,
  or the stdio transport an MCP server speaks.
- **required** Authentication and authorization: what a caller presents, and what
  each role may call.
- **required** Errors: the shape of a failure, and every code or error type a caller
  may meet, with what each means.
- **required** Limits: rate, request and response size, pagination, timeouts.
- **required** Versioning and deprecation: how a breaking change reaches a caller,
  and how long the old surface stays.
- For MCP: the resources and prompts served beside the tools.

### events

- **required** Transport, and how a topic or channel is named.
- **required** Ordering: what is guaranteed within a key, across keys, and after a
  retry.
- **required** Delivery: at-most-once, at-least-once or exactly-once, and what a
  consumer must do to be idempotent.
- Replay and retention: how far back a consumer may read, and for how
  long a message is kept.
- **required** Schema evolution: what a producer may add or remove without breaking
  a consumer.
- Dead letters: what happens to a message no consumer accepts.

### cli

- **required** Every exit code and its meaning, per command where they differ.
- **required** Streams: what goes to stdout and what to stderr, and what a closed
  pipe does.
- Prompting: when a command prompts, and what it does without a TTY.
- The environment it reads, by reference to the `config` contract.
- Usage errors: what an unknown flag, subcommand or missing value does.
- Side effects: what a command writes, and what it never touches.

### library

- **required** Errors: every error type a caller may match on, and when each is
  returned.
- Invariants and preconditions the signatures cannot say — ordering of
  calls, what must be initialised first.
- Concurrency: what is safe to share across threads or tasks.
- Feature flags: what each enables, and the default set.
- **required** Versioning: the semver policy, and what counts as breaking.

### interface

- **required** Module boundaries: what may call what across the boundary, and what
  must not.
- **required** Key flows: the sequences that cross the boundary, in order.
- **required** Cross-cutting concerns: security, performance, migration,
  observability, each as a promise the module keeps.

### ui

- **required** Routes: what an unknown path does, and which routes require a
  session.
- **required** Screens and states: loading, empty, error and success for each
  screen.
- **required** Platforms and accessibility: what is supported, and the standard met.
- The visual system followed, by reference.

### data

- **required** The store, and how the software reaches it.
- **required** Constraints the schema cannot express: cross-table rules, allowed
  transitions.
- **required** Migration: forward and backward compatibility, and whether a
  migration runs with downtime.
- Retention and personal data: what is kept, for how long, and what is
  never stored.

### format

- **required** Files: where the format is written, where it is read, and how a file
  is identified — magic number, version field, extension.
- **required** Unknown content: what a reader does with a key, field or section it
  does not know.
- **required** Compatibility: what a newer writer promises an older reader, and the
  reverse.
- For binary: endianness, alignment, and how a version is read before
  anything else.

### config

- **required** Sources and precedence: environment, file, flags, defaults, and which
  wins.
- **required** Defaults, and which settings are required with none.
- Secrets: which settings carry credentials, how they are supplied, and
  what is never logged or echoed.
- Validation: what an invalid or unknown setting does.

### telemetry

- **required** Metrics: which labels are bounded, and the cardinality budget.
- **required** Logs: levels, structure, and what is never logged.
- Traces: which spans exist and how context propagates.
- Retention: how long each signal is kept.

### authz

- **required** Principals: who acts, and how identity is established.
- **required** Roles and scopes: the vocabulary, one meaning each.
- **required** Permissions: which action on which resource needs what.
- **required** Boundaries: what no role may reach, regardless of grant.
- Audit: how a decision is recorded.

### deployment

- **required** Artifact: what is shipped, and how a build is identified.
- **required** Runtime: ports, user, writable paths, resource limits.
- **required** Health and lifecycle: readiness, liveness, and how a shutdown
  proceeds.
- Dependencies: what must be up first, and what happens when it is not.

## The contract

````yaml
description: >
  One interface, of one kind: what it is, materialised from the source
  that declares it; what it promises that the definition cannot say;
  and what may change.
line-limit: 800

frontmatter:
  type:
    required: true
    const: Contract
  id:
    required: true
    pattern: '^contract-\d{3}-(api|events|cli|library|interface|ui|data|format|config|telemetry|authz|deployment)-[a-z0-9-]+$'
  kind:
    required: true
    enum: [api, events, cli, library, interface, ui, data, format, config, telemetry, authz, deployment]
    description: >
      What a reader asks for — the API, the CLI, the database — never
      the form the definition takes. The id's third segment carries the
      same token, and the filing check reports a disagreement.
  title:
    required: true
    description: The one-line name of the interface.
  description:
    required: true
  lifecycle:
    enum: [active, deprecated]
    description: >
      The folder is the value: active while callers may rely on it,
      deprecated once superseded.
  resource:
    description: >
      The implementation the contract describes, for a reader who wants
      the code. Not the definition — the Definition section carries
      that, from the declaring source.

sections-ordered: true
sections:
  - heading-pattern: '^(API|Events|CLI|Library|Interface|UI|Data|Format|Config|Telemetry|Authz|Deployment) contract: .+$'
    level: 1
    required: true
    description: >
      The kind's display name, "contract:", and the interface's name —
      "CLI contract: superdev", "API contract: sokf over MCP". The
      display name is the kind capitalised, with API, CLI and UI as
      initialisms.
  - heading: "Definition"
    level: 2
    required: true
    content: code
    description: >
      The interface, materialised from the source that declares it: one
      or more include blocks naming a repository path and region
      (`<!-- sokf:include /path#region -->`), nothing authored. The
      source's doc comments arrive with it and are contract text. A
      source unreadable as a surface is included through a generated
      rendering that names its generator. Until I049's build lands
      `content: include`, this section declares `code`; a hand-written
      block here is already against the standard above.
  - heading: "Behaviour"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What the definition cannot say, one requirement per sentence with
      an RFC 2119 modal verb, under one `###` per item of the kind's
      checklist above that applies. A promise whose behaviour is not
      built yet carries PENDING beside its verb, naming the issue or
      slice.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What may change and how a caller learns of it: the versioning
      policy, the deprecation path, what is promised across a release.
      An internal interface says so here — "Internal. Changes with the
      crate." — rather than omitting the section.

example: |
  ---
  type: Contract
  id: contract-002-cli-widget
  kind: cli
  title: CLI contract for widget
  description: The widget command line — every command, flag and exit code it offers.
  lifecycle: active
  resource: /crates/widget/src/main.rs
  ---

  # CLI contract: widget

  ## Definition

  <!-- sokf:include /crates/widget/src/main.rs#cli -->
  ```rust
  /// Build the widget.
  #[derive(Parser)]
  pub struct Build {
      /// Skip the tests.
      #[arg(long)]
      pub no_test: bool,
  }
  ```
  <!-- /sokf:include -->

  ## Behaviour

  ### Exit codes

  `build` MUST exit 0 on success, 1 when a check fails, and 2 on a usage
  error.

  ### Streams

  `build` MUST write its report to stdout and diagnostics to stderr.

  ### Side effects

  `build` MUST NOT write outside `target/`.

  ## Stability

  Unreleased. Every command and flag above MAY change without notice.
````

