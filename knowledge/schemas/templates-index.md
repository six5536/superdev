---
type: Schema
id: schema-templates-index
title: Templates Index Schema
description: The grouped listing of every template with its one-line summary, in knowledge/templates/index.md.
---

# Templates Index Schema

Structural rules for `knowledge/templates/index.md`, the grouped listing of
the templates. Derived from the index itself, which is both the template and
the instance — the only file in the set that is its own example. The
category headings are the author's to name, so what this schema holds is the
shape of an entry, not the taxonomy.

````yaml
target-files: "knowledge/templates/index.md"
description: >
  The listing of every template, grouped by the part of the process that
  produces it, each entry linking the file and summarising it in one line.
line-limit: 800

sections-ordered: true
sections:
  - heading: "Templates"
    level: 1
    required: true
    content: prose
    description: >
      What the templates are — copy-verbatim skeletons for the documents the
      development process produces — and how to use one: read it by id, strip
      the frontmatter, and fill in the angle-bracket placeholders.
  - heading-pattern: '^.+$'
    level: 2
    required: true
    repeatable: true
    content: bullet-list
    description: >
      One heading per group, named for the part of the process that produces
      the templates under it. One bullet per template: a link carrying its
      title, then a dash and a one-line summary of what the template produces
      and where it is filed. The summary matches the template's own frontmatter
      description, so a change to one is visible as a difference from the
      other.

example: |
  # Templates

  Copy-verbatim skeletons for the documents the development process produces.
  Read one with `sokf_read` (id `template-<name>`), strip the frontmatter, and
  fill in the angle-bracket placeholders.

  ## Planning & design

  * [Spec][sokf:template-spec] - what done looks like from outside — behaviour, acceptance criteria, UI states, edge cases, out of scope. Filed as a draft concept in `knowledge/specs/`, tagged done at accept.
  * [ADR][sokf:template-adr] - architecture decision record — context, the decision, options considered, and consequences. Filed as a Decision concept in `knowledge/decisions/`.

  ## Change delivery

  * [Code Review][sokf:template-code-review] - verdict first, findings ranked by severity with concrete failure scenarios, and what was checked and found fine.
````
