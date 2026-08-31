# Issues

## Externally sourced content packs

* [update can move a pin to a pack format this binary cannot read, and cannot move it back][sokf:issue-001-bug-update-can-pin-an-unreadable-pack-format] - update persisted a moved pin before sync validated it, and a pin never moves backwards, so a content release in a newer format left every later sync and update failing until the manifest was hand-edited; fixed in P005 slice 6, which proves a pin before writing it.
* [The default-source query has no time bound, so a black-holed network stalls update][sokf:issue-002-bug-no-time-bound-on-the-update-query] - update runs git ls-remote on every untargeted invocation and CommandRunner had no timeout, so a network that neither answered nor refused stalled the command until the OS gave up; fixed in P005 slices 4 and 5, which gave the spawn seam a five-second deadline.
* [Deleting an item from a local pack leaves its live copy in place, and the drift check stays green][sokf:issue-003-bug-a-local-pack-cannot-remove-what-it-dropped] - a path pack layers rather than replacing, so an item deleted or renamed under pack/ is still written from the embedded snapshot and status --drift exits 0 until the binary is rebuilt; wontfix — the layering rule stands, and the rebuild a pack developer needs anyway is the answer.
* [A path pack's lock digest is rewritten by every content commit and verified by nothing][sokf:issue-004-bug-a-path-packs-digest-churns-and-is-never-checked] - the lock recorded a digest over a path pack's whole tree that resolution never checked, so every commit touching pack/ rewrote the same line and conflicted between concurrent content PRs; fixed in P005 slice 7, which makes the digest optional and records none for a path source.
* [Sync re-records a hash only for a file it writes, so backporting an edit leaves the lock stale][sokf:issue-005-bug-a-backport-leaves-the-lock-stale] - after a live edit was mirrored into the pack, sync had nothing to write and never refreshed that file's recorded hash, so the next legitimate write reported it as user-edited and backed it up; fixed in slice 17, which reconciles every claim against disk before saving the lock.
* [Content packs are absent from the user documentation, and the update command now describes itself wrongly][sokf:issue-006-feature-request-content-packs-are-undocumented-for-users] - neither the README nor the CLI help nor the man page mentioned packs, so a user could not discover the feature, and update's description still claimed it moved pins to this binary's defaults; fixed in slice 18, which added a packs section and corrected update on every surface.
* [A pack source's scheme is unchecked, so the base pack can be fetched over a transport anyone on-path can answer][sokf:issue-007-bug-a-pack-source-reaches-git-with-no-scheme-check] - superdev allowlisted no scheme, so git:// and http:// normalised onto the default identity and a cloned manifest could have the base pack fetched over an unauthenticated transport; fixed in P005 slice 1 and ADR-012, which allowlist https, ssh and file at parse.
* [A symlinked file in a pack is followed, copying the target's contents into the repo][sokf:issue-008-bug-a-symlinked-file-in-a-pack-is-followed] - read_dir skipped a symlinked directory but not a symlinked file, so a pack could name a link to any readable file on the machine and superdev wrote its contents into the working tree as pack content; fixed in slice 16 and hardened by P005 slices 2 and 3 into a refusal.
* [A symlink inside a pack is skipped in silence, so an item a pack meant to ship quietly disappears][sokf:issue-009-bug-a-skipped-symlink-says-nothing] - the walk dropped every symlink without reporting it, so a pack that deduped an item with a link resolved clean while that item was simply absent; fixed in P005 slices 2 and 3, which refuse a symlink anywhere in a pack and name the path.
* [An index entry may say anything about a concept, and nothing notices][sokf:issue-010-feature-request-index-entries-are-never-checked-against-their-concept] - SPEC §9 says an index entry should carry the linked concept's description, but check_indexes only tests that the target exists, so an index can drift from every concept it lists — or hold the only copy of something — and validate still passes.
* [The shape SPEC §9 gives an index is described but never enforced][sokf:issue-011-feature-request-index-shape-is-described-but-not-enforced] - SPEC §9 fixes what an index.md looks like — no frontmatter, heading-grouped link lists, one entry per concept — but no validator checks any of it, so an index can carry frontmatter, drop its heading, or mix bullet styles and still pass.
* [Five findings the repository alone can decide are only warnings, and go unread][sokf:issue-012-feature-request-five-decidable-findings-only-warn] - broken links, missing resources, missing sources, missing index targets and unjoined footnotes are all decidable from the tree, but SPEC §11 makes them warnings; the canonical knowledge carried 39 of them unactioned until someone happened to look.

## Issue tracker

* [The Issue type has one shape, bug-report, so everything filed has to pretend to be a defect][sokf:issue-015-feature-request-every-issue-must-be-a-bug-report] - one schema and one template constrain type Issue, so a feature request, a rename or a decision has to invent repro steps and an environment to be filed at all — six of the fourteen issues on file already do, and a feature request has no home but an untracked bullet in the backlog.

## The pack


## Naming

* [The canonical knowledge is called "the bundle" on every surface, and the word describes nothing][sokf:issue-013-chore-the-knowledge-is-called-the-bundle] - the specification, the documents and the CLI are clear of the word, but "bundle" remains in 365 places inside crates/ — load_bundle, bundle_dir, BundleManifest, bundle_root — where it tells a reader nothing about a directory of markdown the repository owns.
* [The schema validator is called "format", which already means three other things here][sokf:issue-014-chore-the-schema-validator-is-called-format] - the grammar-driven validator lived at src/format/, shipped its grammar at .agents/format/ and called its files "superdev-format", while format! is 457 lines away in the same crate, "pack format" is a glossary term and the knowledge format is itself a format.

## The authoring format

* [The format the agent must write in has no document, and the renderer that would produce one has no consumer][sokf:issue-017-feature-request-the-format-has-no-agent-facing-document] - every skill and schema is written in superdev-format, and the only statement of it is a 700-line grammar file the agent is never pointed at; the doc renderer ported for exactly this now exists in the binary with nothing calling it but a flag nobody runs.

## The schema layer

* [A schema declares content kinds and a frontmatter contract, and the validator reads neither][sokf:issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else] - P008 made schemas govern documents, but only their sections — the content kind under each heading and the frontmatter constraints beside it are declared on every schema and read by nothing, which is the fault P008 set out to cure, one level down.
* [validate reads a file named on the command line as a skill, whatever it is][sokf:issue-019-bug-validate-reads-a-named-file-as-a-skill] - superdev validate knowledge/architecture.md reports nine errors about missing skill blocks, because a named path takes the grammar's fallback kind; the document is never checked against the schema its type names, so the check a path argument most obviously invites is the one it cannot run.
* [A schema's worked example is the thing agents copy, and it is the one part of the schema nothing checks][sokf:issue-022-feature-request-a-schemas-worked-example-is-checked-by-nothing] - every schema carries an `example:` block showing a conforming document, and no check reads it — five of the twenty-three example ids on file broke their own schema's id pattern, left behind by a migration that changed the pattern and not the example.

## The engine

* [A claimed file superdev never wrote has no lock hash, so its first rewrite misreports as a user edit][sokf:issue-025-bug-a-claim-never-written-gets-no-lock-hash] - the lock reconcile refreshes existing entries and never adds one for a claim already satisfied on disk, so all 53 shipped schemas were unrecorded and each first rewrite reports "overwrote a user-edited file" and spawns a backup.

## The workflow

* [The workflow cannot deliver a feature unattended][sokf:issue-024-feature-request-the-workflow-cannot-run-unattended] - every phase boundary stops and waits for the user, no feature gets a branch of its own, a plan models no slice dependencies, and integrate leaves its record edits uncommitted.

## Link checking

* [A skill naming a concept by path breaks silently when that concept moves, because link checking stops at the knowledge directory][sokf:issue-023-feature-request-a-concept-path-written-outside-the-knowledge-is-checked-by-nothing] - P010 made a link inside the SOKF knowledge survive a rename, but the eleven concept paths written in skills and agent files are checked by nothing, so the failure P010 removed from the knowledge still stands one directory away.

<!-- sokf:links -->
[sokf:issue-001-bug-update-can-pin-an-unreadable-pack-format]: /knowledge/issues/done/issue-001-bug-update-can-pin-an-unreadable-pack-format.md
[sokf:issue-002-bug-no-time-bound-on-the-update-query]: /knowledge/issues/done/issue-002-bug-no-time-bound-on-the-update-query.md
[sokf:issue-003-bug-a-local-pack-cannot-remove-what-it-dropped]: /knowledge/issues/wontfix/issue-003-bug-a-local-pack-cannot-remove-what-it-dropped.md
[sokf:issue-004-bug-a-path-packs-digest-churns-and-is-never-checked]: /knowledge/issues/done/issue-004-bug-a-path-packs-digest-churns-and-is-never-checked.md
[sokf:issue-005-bug-a-backport-leaves-the-lock-stale]: /knowledge/issues/done/issue-005-bug-a-backport-leaves-the-lock-stale.md
[sokf:issue-006-feature-request-content-packs-are-undocumented-for-users]: /knowledge/issues/done/issue-006-feature-request-content-packs-are-undocumented-for-users.md
[sokf:issue-007-bug-a-pack-source-reaches-git-with-no-scheme-check]: /knowledge/issues/done/issue-007-bug-a-pack-source-reaches-git-with-no-scheme-check.md
[sokf:issue-008-bug-a-symlinked-file-in-a-pack-is-followed]: /knowledge/issues/done/issue-008-bug-a-symlinked-file-in-a-pack-is-followed.md
[sokf:issue-009-bug-a-skipped-symlink-says-nothing]: /knowledge/issues/done/issue-009-bug-a-skipped-symlink-says-nothing.md
[sokf:issue-010-feature-request-index-entries-are-never-checked-against-their-concept]: /knowledge/issues/open/issue-010-feature-request-index-entries-are-never-checked-against-their-concept.md
[sokf:issue-011-feature-request-index-shape-is-described-but-not-enforced]: /knowledge/issues/open/issue-011-feature-request-index-shape-is-described-but-not-enforced.md
[sokf:issue-012-feature-request-five-decidable-findings-only-warn]: /knowledge/issues/open/issue-012-feature-request-five-decidable-findings-only-warn.md
[sokf:issue-013-chore-the-knowledge-is-called-the-bundle]: /knowledge/issues/open/issue-013-chore-the-knowledge-is-called-the-bundle.md
[sokf:issue-014-chore-the-schema-validator-is-called-format]: /knowledge/issues/done/issue-014-chore-the-schema-validator-is-called-format.md
[sokf:issue-015-feature-request-every-issue-must-be-a-bug-report]: /knowledge/issues/done/issue-015-feature-request-every-issue-must-be-a-bug-report.md
[sokf:issue-017-feature-request-the-format-has-no-agent-facing-document]: /knowledge/issues/open/issue-017-feature-request-the-format-has-no-agent-facing-document.md
[sokf:issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else]: /knowledge/issues/open/issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else.md
[sokf:issue-019-bug-validate-reads-a-named-file-as-a-skill]: /knowledge/issues/open/issue-019-bug-validate-reads-a-named-file-as-a-skill.md
[sokf:issue-022-feature-request-a-schemas-worked-example-is-checked-by-nothing]: /knowledge/issues/open/issue-022-feature-request-a-schemas-worked-example-is-checked-by-nothing.md
[sokf:issue-023-feature-request-a-concept-path-written-outside-the-knowledge-is-checked-by-nothing]: /knowledge/issues/open/issue-023-feature-request-a-concept-path-written-outside-the-knowledge-is-checked-by-nothing.md
[sokf:issue-024-feature-request-the-workflow-cannot-run-unattended]: /knowledge/issues/done/issue-024-feature-request-the-workflow-cannot-run-unattended.md
[sokf:issue-025-bug-a-claim-never-written-gets-no-lock-hash]: /knowledge/issues/open/issue-025-bug-a-claim-never-written-gets-no-lock-hash.md
