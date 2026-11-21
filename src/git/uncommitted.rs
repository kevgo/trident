use crate::domain::File;
use crate::domain::Result;
use crate::git::Repo;
use crate::git::file_changes::FileChanges;
use crate::git::status::Record;

impl Repo {
    /// provides the files that uncommitted changes touch (staged, unstaged, and untracked)
    pub(crate) fn uncommitted_changes(&self) -> Result<FileChanges> {
        Ok(FileChanges::from_paths(self, self.uncommitted_files()?))
    }

    /// provides the names of the files that uncommitted changes touch
    pub(crate) fn uncommitted_files(&self) -> Result<Vec<File>> {
        Ok(self
            .status(&["--untracked-files=all"])?
            .records()
            .filter(Record::is_change)
            .map(|record| record.path.into())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Result;
    use crate::git::FileStatus::{Changed, Removed};
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
        let have = repo.uncommitted_changes()?;
        let want = vec![("sub/one.txt", Changed), ("sub/two.txt", Changed)];
        pretty::assert_eq!(have.summary(), want);
        Ok(())
    }

    #[test]
    fn includes_untracked_file_with_spaces() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        fs::write(repo.file_path("my file.txt"), "content").unwrap();
        let have = repo.uncommitted_changes()?;
        pretty::assert_eq!(have.summary(), vec![("my file.txt", Changed)]);
        Ok(())
    }

    #[test]
    #[cfg(not(windows))] // Windows doesn't seem to support quotes in file names
    fn includes_untracked_file_with_quotes() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        fs::write(repo.file_path("file\"quote.txt"), "hello").unwrap();
        let have = repo.uncommitted_changes()?;
        pretty::assert_eq!(have.summary(), vec![("file\"quote.txt", Changed)]);
        Ok(())
    }

    #[test]
    fn staged_rename_with_spaces() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("old file.txt")?;
        repo.git_command()
            .args(["mv", "old file.txt", "new file.txt"])
            .run()?;
        let have = repo.uncommitted_changes()?;
        let want = vec![("new file.txt", Changed), ("old file.txt", Removed)];
        pretty::assert_eq!(have.summary(), want);
        Ok(())
    }

    #[test]
    fn unstaged_deletion() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("deleted.txt")?;
        fs::remove_file(repo.file_path("deleted.txt")).unwrap();
        let have = repo.uncommitted_changes()?;
        pretty::assert_eq!(have.summary(), vec![("deleted.txt", Removed)]);
        Ok(())
    }

    #[test]
    fn staged_deletion() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("deleted.txt")?;
        repo.git_command()
            .args(["rm", "--quiet", "deleted.txt"])
            .run()?;
        let have = repo.uncommitted_changes()?;
        pretty::assert_eq!(have.summary(), vec![("deleted.txt", Removed)]);
        Ok(())
    }

    #[test]
    fn staged_change_then_deleted() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("file.txt")?;
        fs::write(repo.file_path("file.txt"), "changed").unwrap();
        repo.git_command().args(["add", "file.txt"]).run()?;
        fs::remove_file(repo.file_path("file.txt")).unwrap();
        let have = repo.uncommitted_changes()?;
        pretty::assert_eq!(have.summary(), vec![("file.txt", Removed)]);
        Ok(())
    }

    #[test]
    fn removed_from_index_but_still_in_workspace() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("file.txt")?;
        repo.git_command()
            .args(["rm", "--quiet", "--cached", "file.txt"])
            .run()?;
        let have = repo.uncommitted_changes()?;
        pretty::assert_eq!(have.summary(), vec![("file.txt", Changed)]);
        Ok(())
    }

    #[test]
    fn no_changes() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("file.txt")?;
        assert!(repo.uncommitted_changes()?.is_empty());
        Ok(())
    }
}
