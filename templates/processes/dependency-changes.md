# Process: Dependency changes

## 1. Adding a dependency — justify it first

- Check whether the stdlib, an existing dependency, or ~30 lines of local code already covers the need. A dependency is a long-term liability: supply-chain surface, upgrade burden, install weight.
- Vet the candidate: maintenance activity, download/adoption signals, open critical issues, transitive dependency count, license compatibility with the project, install size.
- Prefer the package the ecosystem/project already leans toward over the newest alternative.
- Pin per project convention (exact vs caret range) and add it to the correct section (runtime vs dev).

## 2. Upgrading — read before you bump

- Read the changelog/release notes between the current and target versions, looking for breaking changes, behavior changes, and dropped runtime support.
- Treat a major-version bump as a migration, not a version edit: budget for API changes, run the project's affected paths, follow the package's migration guide if one exists.
- Upgrade deliberately: one package (or one related group) per change, so a regression bisects to a single bump.
- Security-driven upgrades: confirm the advisory actually applies to how the project uses the package, then take the minimal version that fixes it.

## 3. Removing — verify nothing still needs it

- Search for all imports/usages, including config files, scripts, and lazy/dynamic requires that grep for the name won't catch via imports alone.
- Remove it from the manifest, reinstall, and build+test from clean to prove nothing implicit depended on it.

## 4. Lockfile discipline

- Commit lockfile changes with the manifest change that caused them — never let a lockfile drift in unrelated commits.
- Review the lockfile diff at least for surprises: unexpected new transitive packages, packages resolving from unexpected registries.
- Never hand-edit a lockfile; regenerate it with the package manager.

## 5. Verify and report

- Clean install, build, full test suite — on the exact committed manifest+lockfile pair.
- Report: what changed and why, the vetting done (for additions), breaking changes handled (for upgrades), and anything deferred (e.g. a major bump postponed, with the reason).
