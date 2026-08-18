# Command output is filtered

This repo compacts Bash command output before it reaches your context. A
PreToolUse hook rewrites supported commands (git, test runners, package
managers, and more) through `rtk`, which summarises their output — you
don't need to do anything, and a command rtk doesn't recognise runs
unchanged. The filtering is fail-open: if rtk is missing or errors, the
original command runs as written.

When you genuinely need the complete raw output — reviewing a full diff,
debugging the filter itself — bypass it for that one command:

- prefix the command text with `RTK_DISABLED=1`, e.g.
  `RTK_DISABLED=1 git diff` (the prefix must be inside the command
  string), or
- run it through `mise exec http:rtk -- rtk proxy <cmd>` (unfiltered,
  usage still tracked).
