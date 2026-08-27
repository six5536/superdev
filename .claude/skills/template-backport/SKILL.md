---
name: template-backport
description: "Use when template changes should be harvested from a real project, or when a new project template is added to superdev."
disable-model-invocation: true
---

<skill name="template-backport" purpose="Backport a Project into Template Assets" input="the path to the exemplar project and the template name" user-input="$ARGUMENTS" output="the template assets, registered, documented, and verified">

<goal persona="release engineer">
You turn a real project into template assets under `pack/projects/<name>/` — refreshing a shipped template or creating a new one, from the exemplar given in the input above.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/core.md" when="always" />
<tool_call name="read_file" path="pack/projects/{name}/" when="if refreshing a shipped template" />
</bootstrap_actions>

<process_actions>
<gate check="The exemplar path and template name are confirmed with the user" on-fail="confirm before writing anything" />
<step name="ESTABLISH TOKENS" task="Establish the exemplar's tokens: read its name, slug, and ident from the package name, workspace name, and crate paths. These concrete values become `{{superdev:project-name}}`, `{{superdev:project-slug}}`, and `{{superdev:project-ident}}` in the assets" />
<step name="CHOOSE THE FILE SET" task="Choose the file set: walk the exemplar and decide per file whether it belongs in the template. Leave out build artefacts, lockfiles, `.git/`, and anything specific to the exemplar (its changelog entries, secrets, submodules); ask when in doubt" />
<step name="DIFF EXISTING TEMPLATE" task="Existing template? Diff against its current file set: additions, content changes, and removals are all part of the proposal" />
<step name="REVERSE-SUBSTITUTE" task="Reverse-substitute exact occurrences: name, then ident, then slug. Review every hit — a slug can sit inside an unrelated word, and when the exemplar's name equals its slug, decide from context (paths and package names are slug; prose and headings are name). Grep the result for any leak of the real name, slug, or ident" />
<step name="ENCODE ON-DISK PATHS" task="On disk, strip a leading dot from the first path segment (`gitattributes`, `devcontainer/`) and write a tokenised path segment as `\_slug*`; the FILES table restores both in the target paths" />
<gate check="A per-file summary — add, change, remove, plus any substitution judgement calls — is presented and confirmed" on-fail="wait for the user's confirmation" />
<step name="WRITE THE ASSETS">Write the assets. Keep the FILES table in `src/templates/<name>.rs`in asset-path sort order, with the array length in its type. New template? Add the module, its`shipped()`entry in`templates.rs`, and its name to `TEMPLATE_HELP`in`template_select.rs`(a test fails if the help line misses a template).</step>
<step name="DOCUMENT" task="Document: update the template's section in`knowledge/specs/spec-007-project-templates.md`(or add one) and`CHANGELOG.md`" />
<step name="VERIFY TESTS" task="Verify: `cargo nextest run --workspace`— template content, disjointness, and help-line tests all run here" />
<step name="VERIFY SCRATCH RENDER">Verify with a scratch render:`cargo run --quiet -- template render <name> --name "Widget Forge" --dir <scratch>`— grep it for exemplar leaks,`bash -n` any shell scripts, parse any JSON.</step>
</process_actions>


<rules>
<rule level="MUST NOT">let a template asset target a path superdev's own capabilities own — `AGENTS.md`, `CLAUDE.md`, `.agents/`, `.claude/`, `.superdev/`, `.mise.toml`, `.mcp.json`, `knowledge/` — because a seeded template must stay disjoint from capability files; `.gitignore` is the one allowed overlap, and a unit test in `templates.rs` fails on a collision</rule>
<rule level="SHALL">keep assets LF: `pack/**` is `-text` in this repo's `.gitattributes`</rule>
</rules>
</skill>
