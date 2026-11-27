use crate::domain::Result;
use crate::domain::{FileChanges, FileStatus};
use crate::git::GitCommandExt;
use crate::git::Repo;
use crate::git::ZeroString;

impl Repo {
    /// provides the files that the commits since the given merge base touch
    /// and whether they exist in HEAD
    ///
    /// Lists the old name of a renamed file as a removed file.
    pub(crate) fn branch_committed_files(&self, merge_base: &str) -> Result<FileChanges> {
        let output = self
            .git_command()
            .args([
                "diff",
                "-z",
                "--name-status",
                "--no-renames",
                &format!("{merge_base}...HEAD"),
            ])
            .run_stdout_zero()?;
        Ok(parse_name_status(&output))
    }
}

/// parses the output of `git diff -z --name-status`,
/// which consists of a status field followed by the path, all separated by NUL
fn parse_name_status(output: &ZeroString) -> FileChanges {
    let mut result = vec![];
    let mut fields = output.lines();
    while let Some(status) = fields.next() {
        let Some(path) = fields.next() else {
            log_unexpected_output(status);
            break;
        };
        let file_status = if status == "D" {
            FileStatus::Removed
        } else {
            FileStatus::Changed
        };
        result.push((path.into(), file_status));
    }
    result.into_iter().collect()
}

fn log_unexpected_output(text: &str) {
    println!("unexpected output of \"git diff -z --name-status\": {text}");
}

#[cfg(test)]
mod tests {
    use super::parse_name_status;
    use crate::domain::FileStatus::{Changed, Removed};
    use crate::git::ZeroString;

    #[test]
    fn parses_statuses() {
        let give =
            ZeroString::from("M\0changed.rs\0A\0my file.rs\0D\0deleted.rs\0T\0typechange.rs\0");
        let have = parse_name_status(&give);
        let want = vec![
            ("changed.rs", Changed),
            ("deleted.rs", Removed),
            ("my file.rs", Changed),
            ("typechange.rs", Changed),
        ];
        pretty::assert_eq!(have.summary(), want);
    }

    #[test]
    fn empty() {
        assert!(parse_name_status(&ZeroString::from("")).is_empty());
    }

    #[test]
    fn ignores_trailing_status_without_path() {
        let have = parse_name_status(&ZeroString::from("M\0file.rs\0D"));
        pretty::assert_eq!(have.summary(), vec![("file.rs", Changed)]);
    }
}
