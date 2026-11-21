use crate::domain::File;
use crate::domain::Result;
use crate::git::GitCommandExt;
use crate::git::Repo;

impl Repo {
    /// provides the files that the commits since the given merge base touch
    ///
    /// Lists the old name of a renamed file as well.
    pub(crate) fn branch_committed_files(&self, merge_base: &str) -> Result<Vec<File>> {
        Ok(self
            .git_command()
            .args([
                "diff",
                "-z",
                "--name-only",
                "--no-renames",
                &format!("{merge_base}...HEAD"),
            ])
            .run_stdout_zero()?
            .lines()
            .map(File::from)
            .collect())
    }
}
