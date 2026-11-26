//! files changed on the current branch compared to its parent branch
//!
//! The parent branch is the one that Git Town has stored, otherwise the default branch.

use crate::domain::Result;
use crate::git::Repo;
use crate::git::file_changes::FileChanges;
use crate::gittown;

impl Repo {
    /// provides the files that the current branch touches, including uncommitted changes
    ///
    /// For files with uncommitted changes, Git status knows whether they exist in the workspace.
    /// The workspace contains the other committed files the way HEAD has them.
    pub(crate) fn branch_changes(&self) -> Result<FileChanges> {
        Ok(self
            .uncommitted_changes()?
            .merge(self.branch_committed_changes()?))
    }

    /// provides the files that the commits on the current branch touch,
    /// including both names of renamed files
    ///
    /// The status of each file describes whether it exists in HEAD.
    /// This matches the workspace only if there are no uncommitted changes.
    pub(crate) fn branch_committed_changes(&self) -> Result<FileChanges> {
        let merge_base = gittown::parent_branch(self)
            // the parent branch that Git Town knows might not exist anymore --> use the default branch
            .and_then(|parent| self.merge_base(&parent))
            .or_else(|| {
                self.default_branch()
                    .and_then(|default_branch| self.merge_base(&default_branch))
            });
        let Some(merge_base) = merge_base else {
            // cannot determine the merge base --> no committed files
            return Ok(FileChanges::default());
        };
        self.branch_committed_files(&merge_base)
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
    fn includes_committed_files_when_only_origin_main_exists() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.git_command()
            .args(["update-ref", "refs/remotes/origin/main", "HEAD"])
            .run()?;
        repo.git_command()
            .args([
                "symbolic-ref",
                "refs/remotes/origin/HEAD",
                "refs/remotes/origin/main",
            ])
            .run()?;
        repo.create_and_switch_to_branch("feature")?;
        repo.git_command().args(["branch", "-D", "main"]).run()?;
        repo.create_and_commit_file("on-branch.rs")?;
        repo.create_unstaged_file("uncommitted.rs");
        let have = repo.branch_changes()?;
        let want = vec![("on-branch.rs", Changed), ("uncommitted.rs", Changed)];
        pretty::assert_eq!(have.summary(), want);
        Ok(())
    }

    #[test]
    fn committed_and_uncommitted_removals() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("deleted-in-commit.md")?;
        repo.create_and_commit_file("renamed-in-commit.md")?;
        repo.create_and_commit_file("deleted-uncommitted.md")?;
        repo.create_and_commit_file("kept.md")?;
        repo.create_and_switch_to_branch("feature")?;
        repo.git_command()
            .args(["rm", "--quiet", "deleted-in-commit.md"])
            .run()?;
        repo.git_command()
            .args(["mv", "renamed-in-commit.md", "new-name.md"])
            .run()?;
        repo.git_command()
            .args(["commit", "--quiet", "--message=Remove"])
            .run()?;
        fs::remove_file(repo.file_path("deleted-uncommitted.md")).unwrap();
        let have = repo.branch_changes()?;
        let want = vec![
            ("deleted-in-commit.md", Removed),
            ("deleted-uncommitted.md", Removed),
            ("new-name.md", Changed),
            ("renamed-in-commit.md", Removed),
        ];
        pretty::assert_eq!(have.summary(), want);
        Ok(())
    }

    #[test]
    fn file_recreated_after_committed_deletion() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("file.md")?;
        repo.create_and_switch_to_branch("feature")?;
        repo.git_command()
            .args(["rm", "--quiet", "file.md"])
            .run()?;
        repo.git_command()
            .args(["commit", "--quiet", "--message=Remove"])
            .run()?;
        repo.create_unstaged_file("file.md");
        let have = repo.branch_changes()?;
        pretty::assert_eq!(have.summary(), vec![("file.md", Changed)]);
        Ok(())
    }

    #[test]
    fn file_deleted_after_committed_change() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_switch_to_branch("feature")?;
        repo.create_and_commit_file("file.md")?;
        fs::remove_file(repo.file_path("file.md")).unwrap();
        let have = repo.branch_changes()?;
        pretty::assert_eq!(have.summary(), vec![("file.md", Removed)]);
        // the committed changes describe HEAD, which still contains the file
        let have = repo.branch_committed_changes()?;
        pretty::assert_eq!(have.summary(), vec![("file.md", Changed)]);
        Ok(())
    }

    mod with_git_town {
        use crate::domain::Result;
        use crate::git::FileStatus::{Changed, Removed};
        use crate::git::GitCommandExt;
        use crate::gittown::parent_branch;
        use crate::gittown::tests::{append_branch, git_town_installed, gittown_repo};
        use tempfile::TempDir;

        #[test]
        fn falls_back_to_default_branch_when_parent_branch_is_gone() -> Result<()> {
            if !git_town_installed() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            append_branch(&repo, "parent")?;
            repo.create_and_commit_file("parent.txt")?;
            append_branch(&repo, "child")?;
            repo.create_and_commit_file("child.txt")?;
            repo.git_command()
                .args(["branch", "--quiet", "-D", "parent"])
                .run()?;
            // Git Town still knows the deleted branch as the parent
            pretty::assert_eq!(parent_branch(&repo), Some("parent".to_string()));
            let have = repo.branch_committed_changes()?;
            let want = vec![("child.txt", Changed), ("parent.txt", Changed)];
            pretty::assert_eq!(have.summary(), want);
            Ok(())
        }

        #[test]
        fn only_own_files_on_stacked_branch() -> Result<()> {
            if !git_town_installed() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            append_branch(&repo, "parent")?;
            repo.create_and_commit_file("parent.txt")?;
            append_branch(&repo, "child")?;
            repo.create_and_commit_file("child.txt")?;
            let have = repo.branch_committed_changes()?;
            pretty::assert_eq!(have.summary(), vec![("child.txt", Changed)]);
            Ok(())
        }

        #[test]
        fn excludes_uncommitted_files() -> Result<()> {
            if !git_town_installed() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            append_branch(&repo, "feature")?;
            repo.create_and_commit_file("committed.txt")?;
            repo.create_unstaged_file("uncommitted.txt");
            let have = repo.branch_committed_changes()?;
            pretty::assert_eq!(have.summary(), vec![("committed.txt", Changed)]);
            Ok(())
        }

        #[test]
        fn empty_when_branch_has_no_changes() -> Result<()> {
            if !git_town_installed() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            append_branch(&repo, "feature")?;
            assert!(repo.branch_committed_changes()?.is_empty());
            Ok(())
        }

        #[test]
        fn deleted_and_renamed_files() -> Result<()> {
            if !git_town_installed() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            repo.create_and_commit_file("deleted.txt")?;
            repo.create_and_commit_file("old.txt")?;
            repo.create_and_commit_file("changed.txt")?;
            append_branch(&repo, "feature")?;
            repo.git_command()
                .args(["rm", "--quiet", "deleted.txt"])
                .run()?;
            repo.git_command()
                .args(["mv", "old.txt", "new.txt"])
                .run()?;
            std::fs::write(repo.file_path("changed.txt"), "changed").unwrap();
            repo.git_command().args(["add", "changed.txt"]).run()?;
            repo.git_command()
                .args(["commit", "--quiet", "--message=Remove"])
                .run()?;
            let have = repo.branch_committed_changes()?;
            let want = vec![
                ("changed.txt", Changed),
                ("deleted.txt", Removed),
                ("new.txt", Changed),
                ("old.txt", Removed),
            ];
            pretty::assert_eq!(have.summary(), want);
            Ok(())
        }

        #[test]
        fn file_with_spaces() -> Result<()> {
            if !git_town_installed() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            append_branch(&repo, "feature")?;
            repo.create_and_commit_file("my file.txt")?;
            let have = repo.branch_committed_changes()?;
            pretty::assert_eq!(have.summary(), vec![("my file.txt", Changed)]);
            Ok(())
        }
    }
}
