---
type: Schema
id: schema-error-handling
title: Error Handling Schema
description: The error taxonomy or exit codes and the failure-reporting rules callers rely on, in knowledge/error-handling.md.
---

# Error Handling Schema

Structural rules for `knowledge/error-handling.md`, the canonical knowledge's Convention
concept for how failures are classified and reported. `Exit codes` comes
first and is literal; the mechanism headings after it are the author's to
name.

````yaml
target-files: "knowledge/error-handling.md"
description: >
  The exit codes or error classes and what each means, and one section per
  failure mechanism callers rely on.
line-limit: 800

frontmatter:
  type:
    const: Convention
  id:
    const: error-handling
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Exit codes"
    level: 1
    required: true
    content: prose
    description: >
      The codes or error classes and what each means — including the ones that
      signal a finding rather than a failure.
  - heading-pattern: '^.+$'
    level: 1
    required: true
    repeatable: true
    content: prose
    description: >
      One heading per mechanism callers rely on: how failures are reported,
      retried, rolled back, or logged — and the deliberate exceptions.

example: |
  ---
  type: Convention
  id: error-handling
  title: Error Handling & Logging
  description: Three exit codes, and a finding is not a failure.
  status: stable
  ---

  # Exit codes

  0 is success. 1 is a usage or environment error — the command could not
  run. 2 means the command ran and found something to report: a validation
  finding, a stale lockfile. Scripts that treat any non-zero as a failure
  will misread 2, which is why it is separate.

  # Reporting

  Every error names the file and, where there is one, the line. An error that
  cannot name where it came from is a bug in the error, not in the input.

  # Retries

  Only network fetches retry, three times with backoff. Parse and validation
  failures never retry: they are deterministic, and retrying hides them.
````
