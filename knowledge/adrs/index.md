# ADRs

* [Pack Entries Are a Top-Level Manifest Array][sokf:adr-001-packs-manifest-section] - content packs are declared as a top-level [[packs]] array in config.toml rather than as capability tables, because an absent pack means the embedded snapshot while an absent capability means disabled.
* [Content Resolves Before Planning][sokf:adr-002-resolve-before-plan] - pack resolution is an engine-owned phase that runs before plan_repo and hands components a resolved content set through Ctx, so Component::plan stays side-effect free.
* [A Pack's Items Are Named by Its Directory Layout][sokf:adr-003-items-by-layout] - the unit that supersedes is an item — a whole skill directory, one project template, one document template — identified as (owning capability, kind, name) by where it sits in the pack tree rather than by a list in the pack manifest.
* [The Base Pack Is Identified by Normalised Source][sokf:adr-004-base-pack-identity] - a pack entry replaces the embedded snapshot when its source normalises to the blueprint's default source; status names which entry it treated as the base, so a wrong match is visible rather than silent.
* [Resolved Packs Are Cached Locally and Fetched Only on Demand][sokf:adr-005-pack-cache-and-fetch] - a resolved pack is kept under .superdev/cache/packs/, and superdev reaches the network only when it needs bytes it does not have — a new pin, or repairing a drifted file on a machine that never fetched it.
* [The Stock Pack Lives at the Repo Root][sokf:adr-006-pack-at-repo-root] - superdev's shipped content moves to a browsable /pack directory at the repository root, reached from superdev-core by a symlink so it still packages into the crates.io tarball without becoming a crate of its own.
* [Git Pack Sources Are Fetched by Spawning Git][sokf:adr-007-git-fetch-by-spawn] - superdev resolves a git pack source with a shallow, blobless, sparse clone through the user's own git binary, so any forge, private repo and ssh URL works with the user's credentials and no token is stored.
* [One Command Per Release, Two Tag Series][sokf:adr-008-one-command-per-release] - the release script cuts the binary and the pack together from one commit, and a second command cuts a pack release alone — so neither release has a second step a human could get wrong.
* [Update Queries the Default Source for a Newer Pack][sokf:adr-009-update-queries-default-source] - superdev update asks the blueprint's default pack source for its newest release and moves that pin there, so a content release reaches repos whose binary has not changed; other sources' pins are never moved.
* [A Knowledge Skeleton Is Any Entry Under concepts/][sokf:adr-010-concepts-entry-is-the-item] - the knowledge/concepts/ kind names its item by the entry directly beneath it — a file of any extension or a directory — because the canonical knowledge ships three scaffolds that are not one .md each.
* [A Path Pack's Identity Is Relative to the Repo Root][sokf:adr-011-path-pack-identity-is-root-relative] - a path source's identity is its canonicalised path expressed relative to the repository root with forward slashes, so the committed lock says the same thing on every checkout and every platform.
* [A Pack Source's Transport Is Allowlisted][sokf:adr-012-pack-source-schemes-are-allowlisted] - a pack source may only name https, ssh or file, refused at parse before anything spawns and refused again by git's own protocol policy, because a manifest arrives with a repository and a transport nobody vetted must not inherit the base pack's standing.
* [Update Proves a Pin Before It Writes It][sokf:adr-013-update-proves-a-pin-before-it-writes-it] - update resolves a moved pack pin before saving the manifest and keeps the old pin when it refuses, because the manifest is what every later run reads and a pin never moves backwards.
* [A Symlink in a Pack Is Refused, and Git Decides What One Is][sokf:adr-014-a-symlink-in-a-pack-is-refused] - a symlink anywhere in a pack fails the run naming the path, and for a fetched pack the index mode decides rather than the filesystem, because Windows checks a link out as a plain file and the same rev would otherwise digest differently there.
* [The Spawn Seam Carries a Deadline and an Environment][sokf:adr-015-the-spawn-seam-carries-a-deadline] - CommandRunner gains an options form of run carrying a timeout and extra environment, defaulted so every existing caller is untouched, and the query update makes unprompted is the first caller to set one.
* [A Path Pack Records No Digest][sokf:adr-016-a-path-pack-records-no-digest] - the lock's digest becomes optional and is omitted for a path source, because a directory is read afresh every run so the value is never checked, and recording it rewrites one committed line on every content commit.
* [AOKF Conformance Is Pass or Fail][sokf:adr-017-aokf-conformance-is-pass-or-fail] - the conformance ladder goes; knowledge passes or fails, because superdev grades at level 2 everywhere and the flag that reaches the other levels can only weaken a gate.

* [The Unattended Loop Is a Skill, Enforced by a Hook That Never Parses a Plan][sokf:adr-018-loop-in-the-skill-enforcement-in-the-hook] - the loop over feature-plan, build and integrate lives in a knowledge-carried skill, and a managed Stop hook keeps the turn going by reading only the run state — so the slice format stays pack content and a repo without the hook still gets the behaviour.
* [Run State Is a Session-Owned File Behind CLI Verbs, and the Hook Owns the Counter][sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs] - an unattended run is armed by .superdev/cache/run.toml, created exclusively by superdev run begin and owned by one session; the Stop hook body is superdev hook run, and the hook alone increments the watchdog counter, capped at ten continues without progress.
* [A Blocked Run Ends Rather Than Pauses][sokf:adr-020-a-blocked-run-ends] - a run that hits a question only the user can answer writes it into the plan's deferred decisions and ends, releasing the run state; resuming is a fresh invocation that re-reads the plan and the answers.
* [Nothing Unattended Reaches the Default Branch][sokf:adr-021-nothing-unattended-reaches-the-default-branch] - a feature runs on the branch /frame creates and an adhoc plan touching code on adhoc/<slug>; an unattended run commits and merges only there, and a human fast-forwards the default branch.
* [A Frontmatter Key Is Required by a Per-Key Flag][sokf:adr-022-a-frontmatter-key-is-required-by-a-per-key-flag] - a schema marks a required frontmatter key with a `required` flag beside that key's constraints, mirroring the section rules' own vocabulary, so requiredness reads where the key is declared.
* [A Content Kind Binds by Presence][sokf:adr-023-a-content-kind-binds-by-presence] - a section satisfies its declared content kind when the kind's form appears in its body — a bullet for bullet-list, a fenced block for code — with other content tolerated, so the kind names the section's substance rather than policing every line.
* [A Schema's Example Is Checked in Place Against Its Own Schema][sokf:adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema] - validate reads each schema's example block as a document and runs the document check with the declaring schema handed to it, so a failure is a finding on the schema file and the example never leaves the file agents read it from.
* [An Example's Links Bind by Form and Never Resolve][sokf:adr-025-an-examples-links-bind-by-form-and-never-resolve] - inside a schema's example a concept link must take the sokf id form and a path link into the knowledge is refused, but no id or target is resolved — an example's content is fictional by design, and a link outside the knowledge keeps its ordinary markdown form.

<!-- sokf:links -->
[sokf:adr-001-packs-manifest-section]: /knowledge/adrs/active/adr-001-packs-manifest-section.md
[sokf:adr-002-resolve-before-plan]: /knowledge/adrs/active/adr-002-resolve-before-plan.md
[sokf:adr-003-items-by-layout]: /knowledge/adrs/active/adr-003-items-by-layout.md
[sokf:adr-004-base-pack-identity]: /knowledge/adrs/active/adr-004-base-pack-identity.md
[sokf:adr-005-pack-cache-and-fetch]: /knowledge/adrs/active/adr-005-pack-cache-and-fetch.md
[sokf:adr-006-pack-at-repo-root]: /knowledge/adrs/active/adr-006-pack-at-repo-root.md
[sokf:adr-007-git-fetch-by-spawn]: /knowledge/adrs/active/adr-007-git-fetch-by-spawn.md
[sokf:adr-008-one-command-per-release]: /knowledge/adrs/active/adr-008-one-command-per-release.md
[sokf:adr-009-update-queries-default-source]: /knowledge/adrs/active/adr-009-update-queries-default-source.md
[sokf:adr-010-concepts-entry-is-the-item]: /knowledge/adrs/active/adr-010-concepts-entry-is-the-item.md
[sokf:adr-011-path-pack-identity-is-root-relative]: /knowledge/adrs/active/adr-011-path-pack-identity-is-root-relative.md
[sokf:adr-012-pack-source-schemes-are-allowlisted]: /knowledge/adrs/active/adr-012-pack-source-schemes-are-allowlisted.md
[sokf:adr-013-update-proves-a-pin-before-it-writes-it]: /knowledge/adrs/active/adr-013-update-proves-a-pin-before-it-writes-it.md
[sokf:adr-014-a-symlink-in-a-pack-is-refused]: /knowledge/adrs/active/adr-014-a-symlink-in-a-pack-is-refused.md
[sokf:adr-015-the-spawn-seam-carries-a-deadline]: /knowledge/adrs/active/adr-015-the-spawn-seam-carries-a-deadline.md
[sokf:adr-016-a-path-pack-records-no-digest]: /knowledge/adrs/active/adr-016-a-path-pack-records-no-digest.md
[sokf:adr-017-aokf-conformance-is-pass-or-fail]: /knowledge/adrs/active/adr-017-aokf-conformance-is-pass-or-fail.md
[sokf:adr-018-loop-in-the-skill-enforcement-in-the-hook]: /knowledge/adrs/active/adr-018-loop-in-the-skill-enforcement-in-the-hook.md
[sokf:adr-019-run-state-is-a-session-owned-file-behind-cli-verbs]: /knowledge/adrs/active/adr-019-run-state-is-a-session-owned-file-behind-cli-verbs.md
[sokf:adr-020-a-blocked-run-ends]: /knowledge/adrs/active/adr-020-a-blocked-run-ends.md
[sokf:adr-021-nothing-unattended-reaches-the-default-branch]: /knowledge/adrs/active/adr-021-nothing-unattended-reaches-the-default-branch.md
[sokf:adr-022-a-frontmatter-key-is-required-by-a-per-key-flag]: /knowledge/adrs/active/adr-022-a-frontmatter-key-is-required-by-a-per-key-flag.md
[sokf:adr-023-a-content-kind-binds-by-presence]: /knowledge/adrs/active/adr-023-a-content-kind-binds-by-presence.md
[sokf:adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema]: /knowledge/adrs/active/adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema.md
[sokf:adr-025-an-examples-links-bind-by-form-and-never-resolve]: /knowledge/adrs/active/adr-025-an-examples-links-bind-by-form-and-never-resolve.md
