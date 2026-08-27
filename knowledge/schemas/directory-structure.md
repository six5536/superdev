---
type: Schema
id: schema-directory-structure
title: Directory Structure Schema
description: What lives where in the repository, in knowledge/directory-structure.md.
---

# Directory Structure Schema

Structural rules for `knowledge/directory-structure.md`, the canonical knowledge's
Reference concept for the repository layout. The document is a tree block
and a note, with no headings, so it declares a preamble and no sections.

````yaml
target-files: "knowledge/directory-structure.md"
description: >
  What lives where in the repository — the annotated tree, and only what the
  tree itself cannot say.
line-limit: 800

frontmatter:
  type:
    const: Reference
  id:
    const: directory-structure
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: code
  description: >
    A fenced block listing the directories with a one-line note against each,
    then only what the tree cannot say: generated directories, files that must
    not be hand-edited, and where new code goes.

example: |
  ---
  type: Reference
  id: directory-structure
  title: Directory Structure
  description: What lives where in the repository.
  status: stable
  ---

  ```text
  crates/lib/superdev-core/   # resolution, manifest, lock, knowledge reads
  crates/bin/superdev-cli/    # argument parsing and output
  knowledge/                  # the canonical project knowledge this tool validates
  .superdev/cache/            # generated, machine-local, safe to delete
  ```

  `.superdev/cache/` is written by the tool and gitignored; nothing in it is
  ever hand-edited. New core behaviour goes under `crates/lib/superdev-core/`
  and reaches the CLI only through the library's public API.
````
