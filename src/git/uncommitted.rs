use crate::domain::File;
use crate::domain::Result;
use crate::git::Repo;
use crate::git::status::Record;

impl Repo {
    /// provides the uncommitted files (staged, unstaged, and untracked)
    pub fn uncommitted(&self) -> Result<Vec<File>> {
        Ok(self
            .status(&["--untracked-files=all"])?
            .records()
            .filter(Record::is_uncommitted)
            .map(|record| record.path.into())
            .collect())
    }

    /// provides the files that uncommitted changes delete, including the old names of renamed files
    pub(crate) fn uncommitted_deleted(&self) -> Result<Vec<File>> {
        Ok(self
            .status(&["--no-renames"])?
            .records()
            .filter(Record::is_deleted)
            .map(|record| File::from(record.path))
            .filter(|file| !self.file_exists(file))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::File;
    use crate::domain::Result;
    use crate::git::GitCommandExt;
    use crate::git::Repo;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn expands_untracked_folder_to_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        let sub = repo.file_path("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("one.txt"), "one").unwrap();
        fs::write(sub.join("two.txt"), "two").unwrap();
        let mut have = repo.uncommitted()?;
        have.sort();
        let want = vec![File::from("sub/one.txt"), File::from("sub/two.txt")];
        pretty::assert_eq!(have, want);
        Ok(())
    }

    #[test]
    fn includes_untracked_file_with_spaces() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        fs::write(repo.file_path("my file.txt"), "content").unwrap();
        let have = repo.uncommitted()?;
        let want = vec![File::from("my file.txt")];
        pretty::assert_eq!(have, want);
        Ok(())
    }

    #[test]
    #[cfg(not(windows))] // Windows doesn't seem to support quotes in file names
    fn includes_untracked_file_with_quotes() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        fs::write(repo.file_path("file\"quote.txt"), "hello").unwrap();
        let have = repo.uncommitted()?;
        let want = vec![File::from("file\"quote.txt")];
        pretty::assert_eq!(have, want);
        Ok(())
    }

    #[test]
    fn includes_renamed_file_with_spaces() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        fs::write(repo.file_path("old file.txt"), "hello").unwrap();
        repo.git_command().args(["add", "old file.txt"]).run()?;
        repo.git_command()
            .args(["commit", "--quiet", "--message=Initial"])
            .run()?;
        repo.git_command()
            .args(["mv", "old file.txt", "new file.txt"])
            .run()?;
        let have = repo.uncommitted()?;
        pretty::assert_eq!(have, vec![File::from("new file.txt")]);
        Ok(())
    }

    mod uncommitted_deleted {
        use crate::domain::{File, Result};
        use crate::git::{GitCommandExt, Repo};
        use std::fs;
        use tempfile::TempDir;

        fn repo_with_file(dir: &TempDir, file: &str) -> Result<Repo> {
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file(file)?;
            Ok(repo)
        }

        #[test]
        fn unstaged_deletion() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = repo_with_file(&dir, "gone.rs")?;
            fs::remove_file(repo.file_path("gone.rs")).unwrap();
            pretty::assert_eq!(repo.uncommitted_deleted()?, vec![File::from("gone.rs")]);
            pretty::assert_eq!(repo.uncommitted()?, Vec::<File>::new());
            Ok(())
        }

        #[test]
        fn staged_deletion() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = repo_with_file(&dir, "gone.rs")?;
            repo.git_command()
                .args(["rm", "--quiet", "gone.rs"])
                .run()?;
            pretty::assert_eq!(repo.uncommitted_deleted()?, vec![File::from("gone.rs")]);
            Ok(())
        }

        #[test]
        fn staged_rename_deletes_the_old_name() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = repo_with_file(&dir, "old.rs")?;
            repo.git_command().args(["mv", "old.rs", "new.md"]).run()?;
            pretty::assert_eq!(repo.uncommitted_deleted()?, vec![File::from("old.rs")]);
            Ok(())
        }

        #[test]
        fn recreated_file_is_not_deleted() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = repo_with_file(&dir, "back.rs")?;
            repo.git_command()
                .args(["rm", "--quiet", "back.rs"])
                .run()?;
            repo.create_unstaged_file("back.rs");
            pretty::assert_eq!(repo.uncommitted_deleted()?, Vec::<File>::new());
            Ok(())
        }

        #[test]
        fn staged_change_then_deleted_is_not_an_existing_file() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = repo_with_file(&dir, "gone.rs")?;
            fs::write(repo.file_path("gone.rs"), "changed").unwrap();
            repo.git_command().args(["add", "gone.rs"]).run()?;
            fs::remove_file(repo.file_path("gone.rs")).unwrap();
            pretty::assert_eq!(repo.uncommitted()?, Vec::<File>::new());
            pretty::assert_eq!(repo.uncommitted_deleted()?, vec![File::from("gone.rs")]);
            Ok(())
        }

        #[test]
        fn no_deletions() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = repo_with_file(&dir, "kept.rs")?;
            repo.create_unstaged_file("new.rs");
            pretty::assert_eq!(repo.uncommitted_deleted()?, Vec::<File>::new());
            Ok(())
        }
    }
}
