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
kind (ADR-043); the kind is in the frontmatter and the id, and the
section rules tagged with a kind say what its Behaviour must cover
(ADR-045).

A contract is the outside of the black box: the one place a person or
an agent reads what an interface promises without reading the code.
Its Definition is materialised from the source that declares the
interface (ADR-042), so it is readable in one place and cannot drift. Its Behaviour says
what the definition cannot. Its Stability says what may change.

The Definition says what the interface *is*. Behaviour says what the
definition cannot: the promises across elements, the failures, the
limits, what may change. The level-3 rules below are the checklist,
one per kind's item: a required section is one every contract of the
kind carries; an optional one is carried where it applies and omitted
otherwise, never written as "not applicable". A promise a contract
needs that no rule names is added here, tagged with its kinds, so the
next writer of the kind sees it.

**Contract style — a contract defines its interface** (superdev
ADR-033, ADR-042, ADR-043, ADR-044, ADR-046, ADR-047, ADR-050,
ADR-051):

- A contract's Definition MUST be one or more source includes of the
  regions that declare the interface, and MUST NOT carry an authored
  block; a caller reads the interface from the contract and reproduces
  it from the source the contract carries.
- A region MUST be bounded by `sokf:begin <name>` and `sokf:end <name>`
  in the source's own comment syntax. What is not marked is not
  promised.
- A doc comment inside an included region is contract text: a modal
  verb there binds as a promise under Behaviour does.
- Prose MUST describe and MUST NOT define. Behaviour MUST carry what no
  single element can say and what no include reaches — stability,
  consumers, behaviour across elements, exit codes, error semantics —
  as promises.
- A promise MUST be one top-level bullet under Behaviour or Stability,
  at any heading depth, opening with its key in a code span and an
  EARS tag — `[ubiquitous]`, `[event]`, `[state]`, `[conditional]`,
  `[optional]` or `[complex]` — then the sentence in that pattern's words: the
  trigger or condition, the interface element as the subject (`init`,
  the validator, a caller, a served tool), one modal verb, one
  requirement. The verb MUST be `SHALL` or `SHALL NOT` for a
  requirement, `SHOULD` or `SHOULD NOT` for a recommendation, or `MAY`
  for an option; `MUST`, `REQUIRED`, `RECOMMENDED` and `OPTIONAL` are
  retired from contracts.
- A promise MAY carry a nested bullet list of the criteria that check
  it, one criterion per nested bullet, each in the promise's form and
  held to the same rules — its key in a code span, an EARS tag, then
  the sentence in that pattern's words with one modal verb from the
  same five, a retired keyword and a second modal verb failing a
  criterion as they fail a promise. A promise with no criterion is
  its own check. A bullet below a criterion binds nothing.
- A key MUST be `P_` followed by a slug of lowercase letters and digits
  joined by hyphens, unique within the contract across both sections.
  A criterion's key MUST be `AC_` followed by the same slug form, unique
  within the contract with every `P_` key.
  The key is the promise's identity: a rewording keeps it, a promise
  that no longer holds is removed with its key and the key is not
  reused, and a new promise takes a new key. A criterion's key is held
  the same way.
- Prose under Behaviour and Stability MUST describe and MUST NOT carry
  a modal verb. A numbered list is a sequence — the steps of a flow —
  and never a promise. A table stays where the kind's checklist wants
  one; its rows are not sentences.
- A promise MUST be cited by its bare key where the contract is the
  subject — a test's doc comment, a plan case — and by the contract's
  id followed by the key elsewhere: `contract-002-cli-superdev
  P_init-outside-git`. A criterion MUST be cited the same way:
  `AC_init-outside-git` where the contract is the subject,
  `contract-002-cli-superdev AC_init-outside-git` elsewhere.
- Behaviour MUST cover what the schema's checklist names for the
  contract's kind, one `###` per item that applies.
- A contract MUST bind what it names and MUST NOT state how the
  interface is built inside.
- The Definition is bound by its include. The project MUST bind each
  Behaviour promise by a test of the behaviour it promises.
- A built-from source unreadable as a surface MUST be rendered by a
  generator that writes `sokf:generated-by <what>` in the rendering's
  leading lines, and the rendering MUST be proved current by a test.
- A Behaviour or Stability promise whose behaviour is unbuilt MAY
  carry `PENDING` in uppercase beside its modal verb, naming the issue
  or work block in parentheses, and MUST NOT once the feature settles; a
  definition element carries none. A contract MUST NOT carry a `TBD`
  item: a contract promises or says `PENDING`, and never defers.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.

## The contract

````yaml
description: >
  One interface, of one kind: what it is, materialised from the source
  that declares it; what it promises that the definition cannot say;
  and what may change.
line-limit: 4000
variant-key: kind

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
  - heading-pattern: '^API contract: .+$'
    level: 1
    required: true
    variants: [api]
    description: >
      The kind's display name, "contract:", and the interface's name —
      "API contract: sokf over MCP". Each kind's title rule is tagged
      with the kind, so the display name agrees with `kind` by
      construction.
  - heading-pattern: '^Events contract: .+$'
    level: 1
    required: true
    variants: [events]
    description: >
      The title of an `events` contract — "Events contract: order stream".
  - heading-pattern: '^CLI contract: .+$'
    level: 1
    required: true
    variants: [cli]
    description: >
      The title of a `cli` contract — "CLI contract: superdev".
  - heading-pattern: '^Library contract: .+$'
    level: 1
    required: true
    variants: [library]
    description: >
      The title of a `library` contract — "Library contract: widget-core".
  - heading-pattern: '^Interface contract: .+$'
    level: 1
    required: true
    variants: [interface]
    description: >
      The title of an `interface` contract — "Interface contract: the planner".
  - heading-pattern: '^UI contract: .+$'
    level: 1
    required: true
    variants: [ui]
    description: >
      The title of a `ui` contract — "UI contract: the console".
  - heading-pattern: '^Data contract: .+$'
    level: 1
    required: true
    variants: [data]
    description: >
      The title of a `data` contract — "Data contract: the orders store".
  - heading-pattern: '^Format contract: .+$'
    level: 1
    required: true
    variants: [format]
    description: >
      The title of a `format` contract — "Format contract: lock.toml".
  - heading-pattern: '^Config contract: .+$'
    level: 1
    required: true
    variants: [config]
    description: >
      The title of a `config` contract — "Config contract: config.toml".
  - heading-pattern: '^Telemetry contract: .+$'
    level: 1
    required: true
    variants: [telemetry]
    description: >
      The title of a `telemetry` contract — "Telemetry contract: the service".
  - heading-pattern: '^Authz contract: .+$'
    level: 1
    required: true
    variants: [authz]
    description: >
      The title of an `authz` contract — "Authz contract: the console".
  - heading-pattern: '^Deployment contract: .+$'
    level: 1
    required: true
    variants: [deployment]
    description: >
      The title of a `deployment` contract — "Deployment contract: the service".
  - heading: "Definition"
    level: 2
    required: true
    content: include
    description: >
      The interface, materialised from the source that declares it: one
      or more include blocks naming a repository path and region
      (`<!-- sokf:include /path#region -->`), nothing authored. The
      source's doc comments arrive with it and are contract text. A
      source unreadable as a surface is included through a generated
      rendering that names its generator.
  - heading: "Behaviour"
    level: 2
    required: true
    content: bullet-list
    item-key: '^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
    item-pattern: '(?s)^`P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` \[(ubiquitous|event|state|conditional|optional|complex)\] .*\b(SHALL|SHOULD|MAY)\b'
    item-only-pattern: '\b(SHALL|SHOULD|MAY|MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    item-prohibited-pattern: '\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b|(?s)\b(SHALL|SHOULD|MAY)\b.*\b(SHALL|SHOULD|MAY)\b'
    nested:
      required: false
      item-key: '^`(AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
      item-pattern: '(?s)^`AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` \[(ubiquitous|event|state|conditional|optional|complex)\] .*\b(SHALL|SHOULD|MAY)\b'
      item-prohibited-pattern: '\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b|(?s)\b(SHALL|SHOULD|MAY)\b.*\b(SHALL|SHOULD|MAY)\b'
    description: >
      What the definition cannot say, as keyed EARS promises under one
      `###` per level-3 rule tagged with the kind that applies — the
      required ones always. Every top-level bullet, at any heading depth,
      is one promise: a `P_` key in a code span, an EARS tag, the
      interface element as the subject, and one verb from SHALL, SHALL
      NOT, SHOULD, SHOULD NOT and MAY — MUST, REQUIRED, RECOMMENDED and
      OPTIONAL are retired. A promise MAY carry a nested bullet list of
      the criteria that check it, one per bullet, in the promise's form
      and held to the same rules: an `AC_` key in a code span, an EARS
      tag, and one verb from the same five — a retired keyword, or a
      second modal verb, fails a criterion as it fails a promise. A
      promise with no criterion is its own check. Prose describes and
      carries no modal verb; a numbered list is a sequence, never a
      promise; a table stays where the kind wants one. A key is the
      identity of its promise or criterion, unique across Behaviour and
      Stability and across both levels — a rewording keeps it, a
      removed key is not reused — and is cited bare where the contract
      is the subject, in a plan case or a test's doc comment, and after
      the contract's id elsewhere: `contract-002-cli-superdev
      P_init-outside-git`, `contract-002-cli-superdev
      AC_init-outside-git`. A promise whose behaviour is not built yet
      carries PENDING beside its verb, naming the issue or work block; no
      item reads TBD.
  - heading: "Transport"
    level: 3
    required: true
    variants: [api, events]
    description: >
      Where the interface is served and how a caller reaches it — host,
      path, port, or the stdio transport an MCP server speaks; for
      events, the broker and how a topic or channel is named.
  - heading: "Authentication"
    level: 3
    required: true
    variants: [api]
    description: >
      Authentication and authorization: what a caller presents, and
      what each role may call.
  - heading: "Errors"
    level: 3
    required: true
    variants: [api, library]
    description: >
      The shape of a failure, and every code or error type a caller may
      meet or match on, with what each means and when it is returned.
  - heading: "Limits"
    level: 3
    required: true
    variants: [api]
    description: Rate, request and response size, pagination, timeouts.
  - heading: "Versioning"
    level: 3
    required: true
    variants: [api, library]
    description: >
      How a breaking change reaches a caller and how long the old
      surface stays; for a library, the semver policy and what counts
      as breaking.
  - heading: "Resources and prompts"
    level: 3
    variants: [api]
    description: For MCP, the resources and prompts served beside the tools.
  - heading: "Ordering"
    level: 3
    required: true
    variants: [events]
    description: >
      What is guaranteed within a key, across keys, and after a retry.
  - heading: "Delivery"
    level: 3
    required: true
    variants: [events]
    description: >
      At-most-once, at-least-once or exactly-once, and what a consumer
      must do to be idempotent.
  - heading: "Replay and retention"
    level: 3
    variants: [events]
    description: >
      How far back a consumer may read, and for how long a message is
      kept.
  - heading: "Schema evolution"
    level: 3
    required: true
    variants: [events]
    description: >
      What a producer may add or remove without breaking a consumer.
  - heading: "Dead letters"
    level: 3
    variants: [events]
    description: What happens to a message no consumer accepts.
  - heading: "Exit codes"
    level: 3
    required: true
    variants: [cli]
    description: Every exit code and its meaning, per command where they differ.
  - heading: "Streams"
    level: 3
    required: true
    variants: [cli]
    description: >
      What goes to stdout and what to stderr, and what a closed pipe
      does.
  - heading: "Prompting"
    level: 3
    variants: [cli]
    description: When a command prompts, and what it does without a TTY.
  - heading: "Environment"
    level: 3
    variants: [cli]
    description: The environment it reads, by reference to the `config` contract.
  - heading: "Usage errors"
    level: 3
    variants: [cli]
    description: What an unknown flag, subcommand or missing value does.
  - heading: "Side effects"
    level: 3
    variants: [cli]
    description: What a command writes, and what it never touches.
  - heading: "Invariants"
    level: 3
    variants: [library]
    description: >
      Invariants and preconditions the signatures cannot say — ordering
      of calls, what must be initialised first.
  - heading: "Concurrency"
    level: 3
    variants: [library]
    description: What is safe to share across threads or tasks.
  - heading: "Feature flags"
    level: 3
    variants: [library]
    description: What each flag enables, and the default set.
  - heading: "Module boundaries"
    level: 3
    required: true
    variants: [interface]
    description: >
      What may call what across the boundary, and what must not.
  - heading: "Key flows"
    level: 3
    required: true
    variants: [interface]
    description: The sequences that cross the boundary, in order.
  - heading: "Cross-cutting concerns"
    level: 3
    required: true
    variants: [interface]
    description: >
      Security, performance, migration, observability, each as a
      promise the module keeps.
  - heading: "Routes"
    level: 3
    required: true
    variants: [ui]
    description: What an unknown path does, and which routes require a session.
  - heading: "Screens and states"
    level: 3
    required: true
    variants: [ui]
    description: Loading, empty, error and success for each screen.
  - heading: "Platforms and accessibility"
    level: 3
    required: true
    variants: [ui]
    description: What is supported, and the standard met.
  - heading: "Visual system"
    level: 3
    variants: [ui]
    description: The visual system followed, by reference.
  - heading: "Store"
    level: 3
    required: true
    variants: [data]
    description: The store, and how the software reaches it.
  - heading: "Constraints"
    level: 3
    required: true
    variants: [data]
    description: >
      Constraints the schema cannot state: cross-table rules, allowed
      transitions.
  - heading: "Migration"
    level: 3
    required: true
    variants: [data]
    description: >
      Forward and backward compatibility, and whether a migration runs
      with downtime.
  - heading: "Retention and personal data"
    level: 3
    variants: [data]
    description: What is kept, for how long, and what is never stored.
  - heading: "Files"
    level: 3
    required: true
    variants: [format]
    description: >
      Where the format is written, where it is read, and how a file is
      identified — magic number, version field, extension.
  - heading: "Unknown content"
    level: 3
    required: true
    variants: [format]
    description: >
      What a reader does with a key, field or section it does not know.
  - heading: "Compatibility"
    level: 3
    required: true
    variants: [format]
    description: >
      What a newer writer promises an older reader, and the reverse.
  - heading: "Encoding"
    level: 3
    variants: [format]
    description: >
      For binary: endianness, alignment, and how a version is read
      before anything else.
  - heading: "Sources and precedence"
    level: 3
    required: true
    variants: [config]
    description: Environment, file, flags, defaults, and which wins.
  - heading: "Defaults"
    level: 3
    required: true
    variants: [config]
    description: The defaults, and which settings are required with none.
  - heading: "Secrets"
    level: 3
    variants: [config]
    description: >
      Which settings carry credentials, how they are supplied, and what
      is never logged or echoed.
  - heading: "Validation"
    level: 3
    variants: [config]
    description: What an invalid or unknown setting does.
  - heading: "Metrics"
    level: 3
    required: true
    variants: [telemetry]
    description: Which labels are bounded, and the cardinality budget.
  - heading: "Logs"
    level: 3
    required: true
    variants: [telemetry]
    description: Levels, structure, and what is never logged.
  - heading: "Traces"
    level: 3
    variants: [telemetry]
    description: Which spans exist and how context propagates.
  - heading: "Retention"
    level: 3
    variants: [telemetry]
    description: How long each signal is kept.
  - heading: "Principals"
    level: 3
    required: true
    variants: [authz]
    description: Who acts, and how identity is established.
  - heading: "Roles and scopes"
    level: 3
    required: true
    variants: [authz]
    description: The vocabulary, one meaning each.
  - heading: "Permissions"
    level: 3
    required: true
    variants: [authz]
    description: Which action on which resource needs what.
  - heading: "Boundaries"
    level: 3
    required: true
    variants: [authz]
    description: What no role may reach, regardless of grant.
  - heading: "Audit"
    level: 3
    variants: [authz]
    description: How a decision is recorded.
  - heading: "Artifact"
    level: 3
    required: true
    variants: [deployment]
    description: What is shipped, and how a build is identified.
  - heading: "Runtime"
    level: 3
    required: true
    variants: [deployment]
    description: Ports, user, writable paths, resource limits.
  - heading: "Health and lifecycle"
    level: 3
    required: true
    variants: [deployment]
    description: Readiness, liveness, and how a shutdown proceeds.
  - heading: "Dependencies"
    level: 3
    variants: [deployment]
    description: What must be up first, and what happens when it is not.
  - heading: "Stability"
    level: 2
    required: true
    content: bullet-list
    item-key: '^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
    item-pattern: '(?s)^`P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` \[(ubiquitous|event|state|conditional|optional|complex)\] .*\b(SHALL|SHOULD|MAY)\b'
    item-only-pattern: '\b(SHALL|SHOULD|MAY|MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    item-prohibited-pattern: '\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b|(?s)\b(SHALL|SHOULD|MAY)\b.*\b(SHALL|SHOULD|MAY)\b'
    nested:
      required: false
      item-key: '^`(AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
      item-pattern: '(?s)^`AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` \[(ubiquitous|event|state|conditional|optional|complex)\] .*\b(SHALL|SHOULD|MAY)\b'
      item-prohibited-pattern: '\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b|(?s)\b(SHALL|SHOULD|MAY)\b.*\b(SHALL|SHOULD|MAY)\b'
    description: >
      What may change and how a caller learns of it — the versioning
      policy, the deprecation path, what is promised across a release —
      as keyed EARS promises in the form the Behaviour rule states: a
      `P_` key unique across both sections, an EARS tag, the interface
      element as the subject, one verb from SHALL, SHOULD and MAY, and
      the key cited bare where the contract is the subject and after
      the contract's id elsewhere. A promise here MAY carry its nested
      `AC_` criteria as a Behaviour promise may, keyed uniquely with
      every other key of the contract. An internal interface promises so
      here — "`P_internal` [ubiquitous] Every item above MAY change with
      the crate." — rather than omitting the section.

example:
  api: |
    ---
    type: Contract
    id: contract-001-api-widget
    kind: api
    title: API contract for widget
    description: The widget API — every route, its inputs and its failures.
    lifecycle: active
    ---

    # API contract: widget

    ## Definition

    <!-- sokf:include /crates/widget/src/routes.rs#api -->
    ```rust
    /// List the widgets.
    pub async fn list() -> Json<Vec<Widget>> {}
    ```
    <!-- /sokf:include -->

    ## Behaviour

    Every route answers JSON.

    ### Transport

    - `P_https-under-v1` [ubiquitous] The API SHALL be served over
      HTTPS under `/v1`.

    ### Authentication

    - `P_bearer-token` [ubiquitous] A caller SHALL present a bearer
      token.
    - `P_delete-needs-admin` [event] WHEN a caller deletes a widget,
      the API SHALL require the `admin` role.

    ### Errors

    - `P_failure-body` [event] WHEN a request fails, the API SHALL
      answer a JSON body carrying `code` and `message`.
      - `AC_not-found-body` [event] WHEN a widget is not found, the
        API SHALL answer 404 with `code` set to `not_found`.
      - `AC_invalid-body` [event] WHEN a request body fails to parse,
        the API SHALL answer 400 with `code` set to `invalid`.

    ### Limits

    - `P_rate-limit` [ubiquitous] A caller SHALL NOT exceed 100
      requests per minute.

    ### Versioning

    - `P_breaking-under-v2` [event] WHEN a change breaks a caller, the
      change SHALL ship under `/v2`.
    - `P_v1-stays-twelve-months` [state] WHILE `/v2` is served, `/v1`
      SHALL stay for 12 months.

    ## Stability

    Stable.

    - `P_route-may-be-added` [ubiquitous] A route MAY be added.
    - `P_route-stays-until-v1-retires` [state] WHILE `/v1` is served,
      a route SHALL NOT be removed.
  events: |
    ---
    type: Contract
    id: contract-001-events-orders
    kind: events
    title: Events contract for the order stream
    description: The order events — every message a consumer may read.
    lifecycle: active
    ---

    # Events contract: orders

    ## Definition

    <!-- sokf:include /schemas/orders.proto#events -->
    ```proto
    // An order was placed.
    message OrderPlaced { string order_id = 1; }
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Transport

    - `P_topic` [ubiquitous] The producer SHALL publish every event to
      the Kafka topic `orders.v1`.

    ### Ordering

    - `P_order-within-key` [ubiquitous] Events sharing an `order_id`
      SHALL arrive in the order they were published.

    ### Delivery

    Delivery is at-least-once.

    - `P_repeat-is-one` [event] WHEN a consumer reads a repeated
      `order_id`, the consumer SHALL treat the repeat as one event.

    ### Schema evolution

    - `P_field-may-be-added` [ubiquitous] A producer MAY add a field.
    - `P_field-not-renumbered` [ubiquitous] A producer SHALL NOT
      renumber a field.

    ## Stability

    Stable.

    - `P_message-may-be-added` [ubiquitous] A message type MAY be
      added.
    - `P_message-stays-in-major` [state] WHILE a major version is
      served, a message type SHALL NOT be removed.
  cli: |
    ---
    type: Contract
    id: contract-001-cli-widget
    kind: cli
    title: CLI contract for widget
    description: The widget command line — every command, flag and exit code it offers.
    lifecycle: active
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

    Every command acts on the current directory.

    ### Exit codes

    | Code | Meaning |
    |------|---------|
    | 0 | success |
    | 1 | a check failed |
    | 2 | a usage error |

    - `P_exit-codes` [ubiquitous] `build` SHALL exit with the code the
      table names.

    ### Streams

    - `P_report-to-stdout` [ubiquitous] `build` SHALL write its report
      to stdout and its diagnostics to stderr.

    ## Stability

    Unreleased.

    - `P_unreleased` [ubiquitous] Every command and flag above MAY
      change without notice.
  library: |
    ---
    type: Contract
    id: contract-001-library-widget-core
    kind: library
    title: Library contract for widget-core
    description: The published surface of the widget-core crate.
    lifecycle: active
    ---

    # Library contract: widget-core

    ## Definition

    <!-- sokf:include /crates/widget-core/src/lib.rs#api -->
    ```rust
    /// Build a widget from its spec.
    pub fn build(spec: &Spec) -> Result<Widget, Error> {}
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Errors

    - `P_invalid-spec` [event] WHEN `spec` names no parts, `build`
      SHALL return `Error::InvalidSpec`.

    ### Versioning

    A signature change is breaking.

    - `P_semver` [ubiquitous] The crate SHALL follow semver.

    ## Stability

    Stable.

    - `P_item-in-minor` [ubiquitous] A public item MAY be added in a
      minor release.
  interface: |
    ---
    type: Contract
    id: contract-001-interface-planner
    kind: interface
    title: Interface contract for the planner
    description: The boundary between the planner and the executor.
    lifecycle: active
    ---

    # Interface contract: the planner

    ## Definition

    <!-- sokf:include /crates/widget/src/planner.rs#boundary -->
    ```rust
    /// The actions a plan owes the executor.
    pub fn plan(manifest: &Manifest) -> Vec<Action> {}
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Module boundaries

    - `P_executor-calls-plan` [ubiquitous] The executor SHALL call
      `plan` for its actions.
    - `P_executor-never-reads-manifest` [ubiquitous] The executor SHALL
      NOT read the manifest itself.

    ### Key flows

    A sync runs in this order:

    1. `plan` reads the manifest and returns the actions.
    2. The executor applies the actions.
    3. The executor records what it applied.

    ### Cross-cutting concerns

    - `P_plan-is-pure` [ubiquitous] `plan` SHALL NOT touch the
      filesystem.

    ## Stability

    Internal.

    - `P_internal` [ubiquitous] Every item above MAY change with the
      crate.
  ui: |
    ---
    type: Contract
    id: contract-001-ui-console
    kind: ui
    title: UI contract for the console
    description: The console's routes, screens and the platforms it supports.
    lifecycle: active
    ---

    # UI contract: the console

    ## Definition

    <!-- sokf:include /apps/console/src/routes.ts#routes -->
    ```typescript
    /** Every route the console serves. */
    export const routes = ["/", "/widgets/:id"];
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Routes

    - `P_unknown-path` [event] WHEN a path matches no route, the
      console SHALL render the not-found screen.
    - `P_widget-needs-session` [event] WHEN `/widgets/:id` is opened,
      the console SHALL require a session.

    ### Screens and states

    - `P_three-states` [ubiquitous] Every screen SHALL render a
      loading, an empty and an error state.

    ### Platforms and accessibility

    - `P_wcag` [ubiquitous] The console SHALL meet WCAG 2.2 AA on the
      last two releases of each major browser.

    ## Stability

    Stable.

    - `P_route-may-be-added` [ubiquitous] A route MAY be added.
    - `P_removed-route-redirects` [event] WHEN a route is removed, the
      console SHALL redirect the old path.
  data: |
    ---
    type: Contract
    id: contract-001-data-orders
    kind: data
    title: Data contract for the orders store
    description: The orders tables, their constraints and their migrations.
    lifecycle: active
    ---

    # Data contract: the orders store

    ## Definition

    <!-- sokf:include /db/schema.sql#orders -->
    ```sql
    -- One order per row.
    CREATE TABLE orders (id TEXT PRIMARY KEY, state TEXT NOT NULL);
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Store

    - `P_database-url` [ubiquitous] The software SHALL reach the store
      through the `DATABASE_URL` connection.

    ### Constraints

    - `P_state-forward-only` [event] WHEN an order changes state, the
      store SHALL move it from `placed` to `shipped` and never back.

    ### Migration

    - `P_migration-no-downtime` [ubiquitous] A migration SHALL run
      without downtime.
    - `P_migration-reversible` [ubiquitous] A migration SHALL be
      reversible.

    ## Stability

    Stable.

    - `P_column-may-be-added` [ubiquitous] A column MAY be added.
    - `P_column-stays-in-major` [state] WHILE a major version is
      served, a column SHALL NOT be dropped.
  format: |
    ---
    type: Contract
    id: contract-001-format-lock
    kind: format
    title: Format contract for lock.toml
    description: The lock file — what it records, and what a reader does with the rest.
    lifecycle: active
    ---

    # Format contract: lock.toml

    ## Definition

    <!-- sokf:include /crates/widget/src/lock.rs#format -->
    ```rust
    /// The lock file as written and read.
    pub struct Lock { pub version: u32 }
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Files

    The file is identified by its `version` field.

    - `P_lock-path` [ubiquitous] The writer SHALL write the lock to
      `.widget/lock.toml`.

    ### Unknown content

    - `P_unknown-key-kept` [event] WHEN a reader meets a key it does
      not know, the reader SHALL keep the key unchanged.

    ### Compatibility

    - `P_key-meaning-stable` [ubiquitous] A newer writer SHALL NOT
      change the meaning of an existing key.

    ## Stability

    Stable.

    - `P_key-may-be-added` [ubiquitous] A key MAY be added.
    - `P_key-stays-in-major` [state] WHILE a major version is served,
      a key SHALL NOT be removed.
  config: |
    ---
    type: Contract
    id: contract-001-config-widget
    kind: config
    title: Config contract for widget
    description: Every setting widget reads, where it comes from, and what wins.
    lifecycle: active
    ---

    # Config contract: config.toml

    ## Definition

    <!-- sokf:include /crates/widget/src/config.rs#settings -->
    ```rust
    /// The settings widget reads.
    pub struct Config { pub verbose: bool }
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Sources and precedence

    A setting is read from three sources, the first that carries it
    winning:

    1. A flag.
    2. The environment.
    3. The file.

    - `P_flag-wins` [event] WHEN a setting is given as a flag, the
      loader SHALL take the flag over the environment and the file.

    ### Defaults

    No setting is required.

    - `P_verbose-default` [ubiquitous] `verbose` SHALL default to
      false.

    ## Stability

    Stable.

    - `P_setting-may-be-added` [ubiquitous] A setting MAY be added.
    - `P_setting-stays-in-major` [state] WHILE a major version is
      served, a setting SHALL NOT be removed.
  telemetry: |
    ---
    type: Contract
    id: contract-001-telemetry-service
    kind: telemetry
    title: Telemetry contract for the service
    description: The metrics and logs the service emits.
    lifecycle: active
    ---

    # Telemetry contract: the service

    ## Definition

    <!-- sokf:include /crates/widget/src/telemetry.rs#metrics -->
    ```rust
    /// Requests served, by route and status.
    pub static REQUESTS: Counter = counter!("requests", "route", "status");
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Metrics

    The cardinality budget is 1,000 series.

    - `P_labels-bounded` [ubiquitous] The `route` and `status` labels
      SHALL be bounded.

    ### Logs

    - `P_json-lines` [ubiquitous] The service SHALL write logs as JSON
      lines at `info` and above.
    - `P_no-token-logged` [ubiquitous] The service SHALL NOT write a
      token to a log.

    ## Stability

    Stable.

    - `P_metric-may-be-added` [ubiquitous] A metric MAY be added.
    - `P_label-stays-in-major` [state] WHILE a major version is
      served, a label SHALL NOT be removed.
  authz: |
    ---
    type: Contract
    id: contract-001-authz-console
    kind: authz
    title: Authz contract for the console
    description: Who may do what in the console, and what no role reaches.
    lifecycle: active
    ---

    # Authz contract: the console

    ## Definition

    <!-- sokf:include /policy/console.rego#roles -->
    ```rego
    # The viewer role reads widgets.
    allow { input.role == "viewer"; input.action == "read" }
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Principals

    - `P_signed-session` [ubiquitous] The console SHALL identify a
      principal by a signed session token.

    ### Roles and scopes

    | Role | Scope |
    |------|-------|
    | `viewer` | reads |
    | `admin` | reads and writes |

    - `P_two-roles` [ubiquitous] The console SHALL admit no role
      beyond the two in the table.

    ### Permissions

    - `P_write-needs-admin` [event] WHEN a principal writes, the
      console SHALL require the `admin` role.

    ### Boundaries

    - `P_tenant-boundary` [ubiquitous] A role SHALL NOT read another
      tenant's widgets.

    ## Stability

    Stable.

    - `P_role-may-be-added` [ubiquitous] A role MAY be added.
    - `P_permission-widens-in-major` [conditional] IF a permission
      widens, the change SHALL ship in a major version.
  deployment: |
    ---
    type: Contract
    id: contract-001-deployment-service
    kind: deployment
    title: Deployment contract for the service
    description: What is shipped, where it runs, and how it reports health.
    lifecycle: active
    ---

    # Deployment contract: the service

    ## Definition

    <!-- sokf:include /deploy/service.yaml#runtime -->
    ```yaml
    # The container as deployed.
    image: widget/service:1.4.0
    ports: [8080]
    ```
    <!-- /sokf:include -->

    ## Behaviour

    ### Artifact

    - `P_one-image` [ubiquitous] A release SHALL be one image tagged
      with its semver.

    ### Runtime

    - `P_port-and-user` [ubiquitous] The service SHALL listen on 8080
      as a non-root user.
    - `P_writes-tmp-only` [ubiquitous] The service SHALL write only to
      `/tmp`.

    ### Health and lifecycle

    - `P_healthz` [state] WHILE the service is ready, `/healthz` SHALL
      answer 200.
    - `P_drain` [event] WHEN a shutdown starts, the service SHALL
      drain within 30 s.

    ## Stability

    Stable.

    - `P_port-may-be-added` [ubiquitous] A port MAY be added.
    - `P_port-8080-stays-in-major` [state] WHILE a major version is
      served, the service SHALL keep listening on 8080.
````
