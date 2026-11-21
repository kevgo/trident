use crate::domain::{File, Result};
use crate::git::GitCommandExt;
use crate::git::Repo;

impl Repo {
    #[cfg(test)]
    pub(crate) fn stage_file(&self, file: impl AsRef<str>) -> Result<()> {
        self.git_command().args(["add", "--", file.as_ref()]).run()
    }

    /// stages the given files
    pub fn stage_files(&self, files: &[&File]) -> Result<()> {
        if files.is_empty() {
            return Ok(());
        }
        self.git_command()
            .args(["add", "--"])
            .args(files.iter().map(|file| file.as_str()))
            .run()
    }
}

#[cfg(test)]
mod tests {
    mod stage_file {
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::Repo;
        use crate::git::StagedFiles;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn stages_untracked_file() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("a.txt"), "content").unwrap();
            repo.stage_file("a.txt")?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![File::from("a.txt")],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn file_with_spaces() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("my file.txt"), "content").unwrap();
            repo.stage_file("my file.txt")?;
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
        fn file_with_quotes() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("file\"quote.txt"), "content").unwrap();
            repo.stage_file("file\"quote.txt")?;
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
        fn nested_file() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::create_dir_all(repo.file_path("sub")).unwrap();
            fs::write(repo.file_path("sub/nested.txt"), "content").unwrap();
            repo.stage_file("sub/nested.txt")?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![File::from("sub/nested.txt")],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn stages_modified_file() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("a.txt")?;
            fs::write(repo.file_path("a.txt"), "changed").unwrap();
            repo.stage_file("a.txt")?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![File::from("a.txt")],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn leaves_other_files_unstaged() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("a.txt"), "a").unwrap();
            fs::write(repo.file_path("b.txt"), "b").unwrap();
            repo.stage_file("a.txt")?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![File::from("a.txt")],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn missing_file_fails() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            assert!(repo.stage_file("missing.txt").is_err());
            Ok(())
        }
    }

    mod stage_files {
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::Repo;
        use crate::git::StagedFiles;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn empty_list_is_noop() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("a.txt"), "content").unwrap();
            repo.stage_files(&[])?;
            pretty::assert_eq!(repo.staged()?, StagedFiles::default());
            Ok(())
        }

        #[test]
        fn stages_multiple_files() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("a.txt"), "a").unwrap();
            fs::write(repo.file_path("b.txt"), "b").unwrap();
            let a = File::from("a.txt");
            let b = File::from("b.txt");
            repo.stage_files(&[&a, &b])?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![a, b],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn file_with_spaces() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("my file.txt"), "content").unwrap();
            let file = File::from("my file.txt");
            repo.stage_files(&[&file])?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![file],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        #[cfg(not(windows))] // Windows doesn't seem to support quotes in file names
        fn file_with_quotes() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("file\"quote.txt"), "content").unwrap();
            let file = File::from("file\"quote.txt");
            repo.stage_files(&[&file])?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![file],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn file_starting_with_dash() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("-foo.txt"), "content").unwrap();
            let file = File::from("-foo.txt");
            repo.stage_files(&[&file])?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![file],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn only_given_files() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("a.txt"), "a").unwrap();
            fs::write(repo.file_path("b.txt"), "b").unwrap();
            let a = File::from("a.txt");
            repo.stage_files(&[&a])?;
            let have = repo.staged()?;
            let want = StagedFiles {
                partial: vec![],
                full: vec![a],
                removed: vec![],
            };
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn missing_file_fails() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            let file = File::from("missing.txt");
            assert!(repo.stage_files(&[&file]).is_err());
            Ok(())
        }
    }
}
