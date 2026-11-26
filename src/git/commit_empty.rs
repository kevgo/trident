#[cfg(test)]
use crate::domain::Result;
#[cfg(test)]
use crate::git::GitCommandExt;
use crate::git::Repo;

impl Repo {
    /// creates an empty commit with the given message
    #[cfg(test)]
    pub(crate) fn commit_empty(&self, message: &str) -> Result<()> {
        self.git_command()
            .args(["commit", "--quiet", "--message", message, "--allow-empty"])
            .run()
    }
}

#[cfg(test)]
mod tests {
    mod commit_empty {
        use crate::domain::FileStatus::Changed;
        use crate::domain::Result;
        use crate::git::Repo;
        use big_s::S;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn creates_commit_with_message() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.commit_empty("empty commit")?;
            assert_eq!(repo.last_commit_message()?, "empty commit");
            pretty::assert_eq!(repo.committed_files()?, Vec::<String>::new());
            assert!(repo.status(&[])?.is_empty());
            Ok(())
        }

        #[test]
        fn successive_commits() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.commit_empty("first")?;
            repo.commit_empty("second")?;
            assert_eq!(repo.last_commit_message()?, "second");
            pretty::assert_eq!(repo.committed_files()?, Vec::<String>::new());
            Ok(())
        }

        #[test]
        fn message_with_spaces() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.commit_empty("hello world")?;
            assert_eq!(repo.last_commit_message()?, "hello world");
            Ok(())
        }

        #[test]
        fn leaves_untracked_files_uncommitted() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            fs::write(repo.file_path("untracked.txt"), "content").unwrap();
            repo.commit_empty("empty")?;
            pretty::assert_eq!(repo.committed_files()?, Vec::<String>::new());
            pretty::assert_eq!(
                repo.uncommitted_changes()?.summary(),
                vec![("untracked.txt", Changed)]
            );
            assert_eq!(repo.last_commit_message()?, "empty");
            Ok(())
        }

        #[test]
        fn does_not_change_existing_committed_files() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("a.txt")?;
            repo.commit_empty("empty")?;
            pretty::assert_eq!(repo.committed_files()?, vec![S("a.txt")]);
            assert_eq!(repo.last_commit_message()?, "empty");
            assert!(repo.status(&[])?.is_empty());
            Ok(())
        }
    }
}
