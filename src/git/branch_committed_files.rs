use crate::domain::File;
use crate::domain::Result;
use crate::git::GitCommandExt;
use crate::git::Repo;

impl Repo {
    pub(crate) fn branch_committed_files(&self, merge_base: &str) -> Result<Vec<File>> {
        Ok(self
            .git_command()
            .args([
                "diff",
                "-z",
                "--name-only",
                "--diff-filter=ACMRT",
                &format!("{merge_base}...HEAD"),
            ])
            .run_stdout_zero()?
            .lines()
            .map(File::from)
            .collect())
    }

    /// provides the files that the commits since the given merge base delete or rename away
    pub(crate) fn branch_committed_removed_files(&self, merge_base: &str) -> Result<Vec<File>> {
        Ok(self
            .git_command()
            .args([
                "diff",
                "-z",
                "--name-only",
                "--no-renames",
                "--diff-filter=D",
                &format!("{merge_base}...HEAD"),
            ])
            .run_stdout_zero()?
            .lines()
            .map(File::from)
            .collect())
    }
}
