//! Default-branch discovery and locally reserved numeric record identities.
use super::*;

/// Discover the default branch from an explicit override, local configuration,
/// origin's symbolic HEAD, or one unambiguous conventional local branch.
pub fn default_branch(root: &Path, explicit: Option<&str>) -> Result<String> {
    if let Some(branch) = explicit.filter(|branch| !branch.is_empty()) {
        validate_ref(branch)?;
        revision(root, &format!("refs/heads/{branch}"))?;
        return Ok(branch.into());
    }
    let configured = Command::new("git")
        .args(["config", "--get", "superdev.defaultBranch"])
        .current_dir(root)
        .output()
        .map_err(|source| Error::Io {
            path: root.into(),
            source,
        })?;
    if configured.status.success() {
        return default_branch(
            root,
            Some(String::from_utf8_lossy(&configured.stdout).trim()),
        );
    }
    if configured.status.code() != Some(1) {
        return Err(Error::Manifest {
            message: "cannot read superdev.defaultBranch".into(),
        });
    }
    let remote = Command::new("git")
        .args(["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"])
        .current_dir(root)
        .output()
        .map_err(|source| Error::Io {
            path: root.into(),
            source,
        })?;
    if remote.status.success() {
        let text = String::from_utf8_lossy(&remote.stdout);
        if let Some(branch) = text.trim().strip_prefix("refs/remotes/origin/") {
            return default_branch(root, Some(branch));
        }
    }
    let branches = local_branches(root)?;
    let conventional: Vec<_> = branches
        .iter()
        .filter(|branch| matches!(branch.as_str(), "main" | "master" | "trunk"))
        .collect();
    if conventional.len() == 1 {
        return Ok(conventional[0].clone());
    }
    if branches.len() == 1 && !branches[0].starts_with("work/") {
        return Ok(branches[0].clone());
    }
    Err(Error::Manifest {
        message: "default branch is ambiguous; set git config superdev.defaultBranch <branch>"
            .into(),
    })
}

/// List local branches without interpreting a detached checkout as a branch.
pub fn local_branches(root: &Path) -> Result<Vec<String>> {
    let output = git(
        root,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads/"],
    )?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// List paths in an immutable branch snapshot.
pub fn paths_at_revision(root: &Path, branch: &str) -> Result<Vec<String>> {
    validate_ref(branch)?;
    let output = git(root, &["ls-tree", "-r", "--name-only", branch, "--"])?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// Refuse a new spelling of an already reserved number across lifecycle folders.
/// The caller holds the repository transaction through validation and publication.
pub fn require_unique_record_number(root: &Path, prefix: &str, id: &str) -> Result<()> {
    let number = id
        .strip_prefix(&format!("{prefix}-"))
        .and_then(|tail| tail.split('-').next())
        .filter(|number| number.len() == 3 && number.bytes().all(|byte| byte.is_ascii_digit()))
        .ok_or_else(|| Error::Manifest {
            message: format!("invalid {prefix} identity `{id}`"),
        })?;
    let needle = format!("{prefix}-{number}-");
    let mut paths = paths_at_revision(root, "HEAD")?;
    paths.extend(working_paths(root)?);
    for path in paths {
        if !path.starts_with("knowledge/") {
            continue;
        }
        if let Some(stem) = Path::new(&path).file_stem().and_then(|stem| stem.to_str())
            && stem.starts_with(&needle)
            && stem != id
        {
            return Err(Error::Manifest {
                message: format!(
                    "{prefix} number {number} is already reserved by `{stem}`; choose another ID"
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::{command, repository};
    use super::*;

    #[test]
    fn default_discovery_survives_work_branch_switches() {
        let dir = repository();
        command(dir.path(), &["branch", "-m", "trunk"]);
        command(dir.path(), &["switch", "-c", "work/001-one"]);
        assert_eq!(default_branch(dir.path(), None).unwrap(), "trunk");
        command(dir.path(), &["branch", "main"]);
        assert!(default_branch(dir.path(), None).is_err());
        command(dir.path(), &["config", "superdev.defaultBranch", "trunk"]);
        assert_eq!(default_branch(dir.path(), None).unwrap(), "trunk");
    }

    #[test]
    fn independent_slugs_cannot_reuse_reserved_numbers() {
        let dir = repository();
        std::fs::create_dir_all(dir.path().join("knowledge/plans/open")).unwrap();
        std::fs::write(
            dir.path().join("knowledge/plans/open/plan-042-first.md"),
            "reserved",
        )
        .unwrap();
        command(dir.path(), &["add", "knowledge"]);
        command(dir.path(), &["commit", "-m", "reserve"]);
        assert!(require_unique_record_number(dir.path(), "plan", "plan-042-second").is_err());
        assert!(require_unique_record_number(dir.path(), "plan", "plan-043-second").is_ok());
        assert!(require_unique_record_number(dir.path(), "plan", "plan-042-first").is_ok());
    }
}
