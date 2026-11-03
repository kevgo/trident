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

    /// provides the files that uncommitted changes delete or rename away
    ///
    /// Doesn't list files that exist again in the workspace.
    pub(crate) fn uncommitted_removed(&self) -> Result<Vec<File>> {
        Ok(self
            .status(&[])?
            .removed_paths()
            .map(File::from)
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

    #[test]
    fn skips_staged_file_deleted_from_workspace() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("file.txt")?;
        fs::write(repo.file_path("file.txt"), "changed").unwrap();
        repo.git_command().args(["add", "file.txt"]).run()?;
        fs::remove_file(repo.file_path("file.txt")).unwrap();
        pretty::assert_eq!(repo.uncommitted()?, Vec::<File>::new());
        pretty::assert_eq!(repo.uncommitted_removed()?, vec![File::from("file.txt")]);
        Ok(())
    }

    mod uncommitted_removed {
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::GitCommandExt;
        use crate::git::Repo;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn unstaged_deletion() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("deleted.txt")?;
            fs::remove_file(repo.file_path("deleted.txt")).unwrap();
            let have = repo.uncommitted_removed()?;
            pretty::assert_eq!(have, vec![File::from("deleted.txt")]);
            pretty::assert_eq!(repo.uncommitted()?, Vec::<File>::new());
            Ok(())
        }

        #[test]
        fn staged_deletion() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("deleted.txt")?;
            repo.git_command().args(["rm", "deleted.txt"]).run()?;
            let have = repo.uncommitted_removed()?;
            pretty::assert_eq!(have, vec![File::from("deleted.txt")]);
            pretty::assert_eq!(repo.uncommitted()?, Vec::<File>::new());
            Ok(())
        }

        #[test]
        fn staged_rename() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("old.txt")?;
            repo.git_command()
                .args(["mv", "old.txt", "new.txt"])
                .run()?;
            let have = repo.uncommitted_removed()?;
            pretty::assert_eq!(have, vec![File::from("old.txt")]);
            Ok(())
        }

        #[test]
        fn file_removed_from_index_but_still_in_workspace() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("file.txt")?;
            repo.git_command()
                .args(["rm", "--cached", "file.txt"])
                .run()?;
            let have = repo.uncommitted_removed()?;
            pretty::assert_eq!(have, Vec::<File>::new());
            Ok(())
        }

        #[test]
        fn no_removals() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("file.txt")?;
            fs::write(repo.file_path("file.txt"), "changed").unwrap();
            let have = repo.uncommitted_removed()?;
            pretty::assert_eq!(have, Vec::<File>::new());
            Ok(())
        }
    }
}
