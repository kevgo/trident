use crate::git::GitCommandExt;
use crate::git::Repo;

/// provides the parent branch that Git Town has stored for the current branch
///
/// Returns None if Git Town isn't installed or doesn't know a parent for the current branch,
/// for example because the current branch is the main branch.
pub(crate) fn parent_branch(repo: &Repo) -> Option<String> {
    repo.git_command()
        .args(["town", "config", "get-parent"])
        .run_stdout_trimmed()
        .ok()
        .filter(|parent| !parent.is_empty())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::parent_branch;
    use crate::domain::Result;
    use crate::git::GitCommandExt;
    use crate::git::Repo;
    use std::path::Path;
    use std::process::Stdio;
    use tempfile::TempDir;

    /// creates a Git repo that is configured for Git Town
    pub(crate) fn gittown_repo(path: &Path) -> Result<Repo> {
        let repo = Repo::init(path)?;
        repo.git_command()
            .args(["config", "git-town.main-branch", "main"])
            .run()?;
        Ok(repo)
    }

    /// creates a Git Town branch with the given name as a child of the current branch
    pub(crate) fn append_branch(repo: &Repo, name: &str) -> Result<()> {
        repo.git_command()
            .args(["town", "append", name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .run()
    }

    pub(crate) fn git_town_installed() -> bool {
        which::which("git-town").is_ok()
    }

    #[test]
    fn none_when_unconfigured() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        pretty::assert_eq!(parent_branch(&repo), None);
        Ok(())
    }

    #[test]
    fn none_on_main_branch() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = gittown_repo(dir.path())?;
        pretty::assert_eq!(parent_branch(&repo), None);
        Ok(())
    }

    #[test]
    fn none_when_feature_has_no_parent() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = gittown_repo(dir.path())?;
        repo.create_and_switch_to_branch("feature")?;
        pretty::assert_eq!(parent_branch(&repo), None);
        Ok(())
    }

    #[test]
    fn parent_of_stacked_branch() -> Result<()> {
        if !git_town_installed() {
            return Ok(());
        }
        let dir = TempDir::new().unwrap();
        let repo = gittown_repo(dir.path())?;
        append_branch(&repo, "parent")?;
        pretty::assert_eq!(parent_branch(&repo), Some("main".to_string()));
        append_branch(&repo, "child")?;
        pretty::assert_eq!(parent_branch(&repo), Some("parent".to_string()));
        Ok(())
    }
}
