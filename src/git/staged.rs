//! provides the staged files in the current directory

use crate::domain::{File, FileChanges, FileStatus, Result};
use crate::git::Repo;
use crate::git::status::{GitStatusOutput, Record};

impl Repo {
    /// provides the staged files
    pub(crate) fn staged(&self) -> Result<StagedFiles> {
        let status_output = self.status(&[])?;
        Ok(StagedFiles::from(&status_output))
    }
}

/// the files that are staged in the current directory
#[derive(Debug, Default, Eq, Hash, PartialEq)]
pub struct StagedFiles {
    /// partially staged files: some changes made to this file are staged, other changes are not
    pub partial: Vec<File>,

    /// fully staged files: all changes made to this file are staged
    pub full: Vec<File>,

    /// files with staged changes that the fixes cannot process
    /// because the commit deletes them or they don't exist in the workspace anymore
    pub removed: Vec<File>,
}

impl From<&GitStatusOutput> for StagedFiles {
    /// parses the output of "git status --porcelain=v1 -z"
    fn from(output: &GitStatusOutput) -> StagedFiles {
        let mut result = StagedFiles::default();
        for record in output.records() {
            result.add_record(&record);
        }
        result
    }
}

impl StagedFiles {
    /// parses a line from the output of "git status --porcelain=v1 -z" and adds it to this instance
    fn add_record(&mut self, record: &Record<'_>) {
        let is_staged = is_index_change(record.index);
        if record.index == 'D' || (is_staged && record.worktree == 'D') {
            self.removed.push(record.path.into());
            return;
        }
        let is_working = is_index_change(record.worktree);
        if is_staged && is_working {
            self.partial.push(record.path.into());
        } else if is_staged {
            self.full.push(record.path.into());
        }
    }

    /// provides all staged files, i.e. fully and partially staged ones
    #[must_use]
    pub fn all(&self) -> Vec<&File> {
        let mut result = Vec::with_capacity(self.partial.len() + self.full.len());
        result.extend(self.partial.iter());
        result.extend(self.full.iter());
        result.sort();
        result
    }

    /// provides the staged files and the removed files, with their status
    #[must_use]
    pub fn changes(&self) -> FileChanges {
        let changed = self
            .all()
            .into_iter()
            .map(|file| (file.clone(), FileStatus::Changed));
        let removed = self
            .removed
            .iter()
            .map(|file| (file.clone(), FileStatus::Removed));
        changed.chain(removed).collect()
    }

    /// provides whether there are any staged files to fix
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.partial.is_empty() && self.full.is_empty()
    }
}

fn is_index_change(status: char) -> bool {
    matches!(status, 'A' | 'M' | 'R' | 'C' | 'T')
}

#[cfg(test)]
mod tests {

    mod staged {
        use super::super::StagedFiles;
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::GitCommandExt;
        use crate::git::Repo;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn includes_fully_staged_file_with_spaces() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("my file.txt"), "content").unwrap();
            repo.git_command().args(["add", "my file.txt"]).run()?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![File::from("my file.txt")],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        #[cfg(not(windows))] // Windows doesn't seem to support quotes in file names
        fn includes_fully_staged_file_with_quotes() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("file\"quote.txt"), "content").unwrap();
            repo.git_command().args(["add", "file\"quote.txt"]).run()?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![File::from("file\"quote.txt")],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn removed_files() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("staged-delete.txt")?;
            repo.create_and_commit_file("unstaged-delete.txt")?;
            repo.create_and_commit_file("old.txt")?;
            repo.create_and_commit_file("untracked-again.txt")?;
            repo.git_command()
                .args(["rm", "--quiet", "staged-delete.txt"])
                .run()?;
            repo.git_command()
                .args(["mv", "old.txt", "new.txt"])
                .run()?;
            repo.git_command()
                .args(["rm", "--quiet", "--cached", "untracked-again.txt"])
                .run()?;
            fs::remove_file(repo.file_path("unstaged-delete.txt")).unwrap();
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![File::from("new.txt")],
                removed: vec![
                    File::from("old.txt"),
                    File::from("staged-delete.txt"),
                    File::from("untracked-again.txt"),
                ],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn skips_staged_file_deleted_from_workspace() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("file.txt"), "content").unwrap();
            repo.git_command().args(["add", "file.txt"]).run()?;
            fs::remove_file(repo.file_path("file.txt")).unwrap();
            let want = StagedFiles {
                partial: vec![],
                full: vec![],
                removed: vec![File::from("file.txt")],
            };
            pretty::assert_eq!(repo.staged()?, want);
            Ok(())
        }

        #[test]
        fn includes_renamed_file_with_spaces() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("old file.txt"), "content").unwrap();
            repo.git_command().args(["add", "old file.txt"]).run()?;
            repo.git_command()
                .args(["mv", "old file.txt", "new file.txt"])
                .run()?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![File::from("new file.txt")],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn includes_partially_staged_file_with_spaces() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("my file.txt"), "v1").unwrap();
            repo.git_command().args(["add", "my file.txt"]).run()?;
            repo.git_command()
                .args(["commit", "--quiet", "--message=init"])
                .run()?;
            fs::write(repo.file_path("my file.txt"), "v2").unwrap();
            repo.git_command().args(["add", "my file.txt"]).run()?;
            fs::write(repo.file_path("my file.txt"), "v3").unwrap();
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![File::from("my file.txt")],
                full: vec![],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn ignores_untracked_file_with_spaces() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("my file.txt"), "content").unwrap();
            let have = repo.staged()?;
            let want = StagedFiles::default();
            pretty::assert_eq!(have, want);
            Ok(())
        }
    }

    mod staged_files {
        use super::super::StagedFiles;
        use crate::domain::File;
        use crate::git::status::{GitStatusOutput, Record};
        use maplit::hashmap;

        #[test]
        fn all() {
            let partial_1 = File::from("partial_1.txt");
            let partial_2 = File::from("partial_2.txt");
            let full_1 = File::from("full_1.txt");
            let full_2 = File::from("full_2.txt");
            let give = StagedFiles {
                partial: vec![partial_1.clone(), partial_2.clone()],
                full: vec![full_1.clone(), full_2.clone()],
                removed: vec![],
            };
            let want = vec![&full_1, &full_2, &partial_1, &partial_2];
            let have = give.all();
            assert_eq!(have, want);
        }

        #[test]
        fn new() {
            let give = GitStatusOutput::from(
                [
                    "MM partial.txt",
                    "M  full.txt",
                    " A unstaged.txt",
                    "?? untracked",
                    "R  dir/new.rs",
                    "dir/old.rs",
                    "C  copy.rs",
                    "original.rs",
                    "M  my file.txt",
                    "D  deleted.txt",
                    " D unstaged-delete.txt",
                    "MD changed-then-deleted.txt",
                ]
                .join("\0"),
            );
            let want = StagedFiles {
                partial: vec!["partial.txt".into()],
                full: vec![
                    "full.txt".into(),
                    "dir/new.rs".into(),
                    "copy.rs".into(),
                    "my file.txt".into(),
                ],
                removed: vec!["deleted.txt".into(), "changed-then-deleted.txt".into()],
            };
            let have = StagedFiles::from(&give);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn parse() {
            let tests = hashmap! {
                "MM file.rs" => StagedFiles {
                    partial: vec!["file.rs".into()],
                    full: vec![],
                    removed: vec![],
                },
                "M  file.rs" => StagedFiles {
                    partial: vec![],
                    full: vec!["file.rs".into()],
                    removed: vec![],
                },
                "M  my file.txt" => StagedFiles {
                    partial: vec![],
                    full: vec!["my file.txt".into()],
                    removed: vec![],
                },
                "MM my file.txt" => StagedFiles {
                    partial: vec!["my file.txt".into()],
                    full: vec![],
                    removed: vec![],
                },
                "A  file\"quote.txt" => StagedFiles {
                    partial: vec![],
                    full: vec!["file\"quote.txt".into()],
                    removed: vec![],
                },
                "?? file.rs" => StagedFiles {
                    partial: vec![],
                    full: vec![],
                    removed: vec![],
                },
                "?? my file.txt" => StagedFiles {
                    partial: vec![],
                    full: vec![],
                    removed: vec![],
                },
                "!! file.rs" => StagedFiles {
                    partial: vec![],
                    full: vec![],
                    removed: vec![],
                },
                "UU file.rs" => StagedFiles {
                    partial: vec![],
                    full: vec![],
                    removed: vec![],
                },
                "D  file.rs" => StagedFiles {
                    partial: vec![],
                    full: vec![],
                    removed: vec!["file.rs".into()],
                },
                " D file.rs" => StagedFiles {
                    partial: vec![],
                    full: vec![],
                    removed: vec![],
                },
                "MD file.rs" => StagedFiles {
                    partial: vec![],
                    full: vec![],
                    removed: vec!["file.rs".into()],
                },
                "A  file.rs" => StagedFiles {
                    partial: vec![],
                    full: vec!["file.rs".into()],
                    removed: vec![],
                },
                "R  dir/new.rs" => StagedFiles {
                    partial: vec![],
                    full: vec!["dir/new.rs".into()],
                    removed: vec![],
                },
                "C  dir/new.rs" => StagedFiles {
                    partial: vec![],
                    full: vec!["dir/new.rs".into()],
                    removed: vec![],
                },
                "R  new file.txt" => StagedFiles {
                    partial: vec![],
                    full: vec!["new file.txt".into()],
                    removed: vec![],
                },
            };
            for (give, want) in tests {
                let mut have = StagedFiles::default();
                let record = Record::parse(give).unwrap();
                have.add_record(&record);
                assert_eq!(have, want, "{give}");
            }
        }
    }
}
