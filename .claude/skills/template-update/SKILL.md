---
name: template-update
description: "Use when the user asks to update from the project template, pull in template changes, or adopt a template."
disable-model-invocation: true
---

<skill name="template-update" purpose="Update the Repo from the Project Template" input="the template name, when the user gives one" user-input="$ARGUMENTS" output="a report: what was applied, skipped, and left unresolved">

<goal persona="maintainer">
You bring the repo's files up to date with the project template shipped in the installed `superdev` binary.
</goal>

<bootstrap_actions>
<tool_call name="read_file" path=".agents/superdev.md" when="always" />
<tool_call name="read_file" path=".superdev/config.toml" when="if exists" />
</bootstrap_actions>

<process_actions>
<gate check="The working tree is clean" on-fail="stop — the whole update must be one reviewable, revertable diff" />
<step name="IDENTIFY THE TEMPLATE" task="Identify the template from `[template]` in `.superdev/config.toml`. Present: confirm the recorded template and project name with the user. Absent: run `superdev template list`, judge which shipped template matches the repo's shape (workspace layout, packaging, CI), and propose it — or say none fits and stop. Infer the project name from the repo (package name, workspace name, directory) and confirm both with the user" />
<gate check="`[template].version`, when recorded, is older than `superdev --version`, or absent" on-fail="recorded newer: stop and say to update superdev first; equal: say there is nothing to do and stop, unless the user wants a re-check" />
<step name="RENDER THE TEMPLATE">Render the current template into a scratch directory outside the repo: `superdev template render <name> --name "<project-name>" --dir <scratch>`. Keep the printed `project-name`, `project-slug`, and `project-ident` lines for the recording step rather than re-deriving them.</step>
<step name="CLASSIFY EVERY DIFFERENCE">Classify every difference. The merge base for a file is its content as first seeded: recover it from git history (the commit that added the path, or the last template-update commit). Against that base, compare ours (working tree) and theirs (the render):

- Only the template changed → propose taking theirs.
- Only the user changed it → keep ours, no question needed.
- Both changed → conflict; propose a merge, the user decides.
- In the render but not the repo → git history says whether the user deleted it (respect that) or the template grew it (propose adding).
- Seeded but gone from the render → propose deleting only if unmodified since seeding; otherwise just report it.
  An adoption has no base: every collision between repo and render is a conflict for the user.</step>
  <gate check="One summary of everything — adds, updates, conflicts, deletions, grouped by area (CI workflows, release scripts, dev container, repo docs, policy configs, workspace files) — is presented and the user has agreed to proceed" on-fail="ask whether to proceed at all" />
  <step name="ASK PER AREA" task="Ask one question per affected area: accept or skip. Within an accepted area, apply the clean changes and bring each conflict back as its own question with a proposed resolution" />
  <step name="RECORD WHAT WAS APPLIED">Record what was applied: write `[template]` in `.superdev/config.toml` — the table `init` writes: `name`, the `project-name` and `project-slug` lines from the render (`project-ident` is derived, not recorded), and `version = "<superdev --version>"`.</step>
  <step name="VERIFY AND CLEAN UP" task="Verify: run the repo's own gates (build, tests, lint — whatever its CONTRIBUTING or package scripts define) and delete the scratch directory" />
  </process_actions>


<rules>
<rule level="MUST NOT">apply anything the user hasn't seen in the summary</rule>
</rules>
</skill>
