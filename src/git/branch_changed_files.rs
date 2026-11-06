//! files changed on the current branch compared to the default branch

use crate::domain::File;
use crate::domain::Result;
use crate::git::Repo;
use crate::gittown;

impl Repo {
    /// provides the names of all files changed on the current branch, including uncommitted changes
    pub(crate) fn branch_changed_files(&self) -> Result<Vec<File>> {
        let uncommitted = self.uncommitted()?;

        // try to use Git Town
        if let Some(gittown_files) = gittown::files_changed_on_current_branch(self) {
            return Ok(unique_existing(self, gittown_files, uncommitted));
        }

        // here Git Town didn't work --> use vanilla Git
        let Some(default_branch) = self.default_branch() else {
            // cannot determine the default branch --> process only the uncommitted files
            return Ok(uncommitted);
        };
        let Some(merge_base) = self.merge_base(&default_branch) else {
            // cannot determine the merge base --> process only the uncommitted files
            return Ok(uncommitted);
        };
        let committed = self.branch_committed_files(&merge_base)?;
        Ok(unique_existing(self, committed, uncommitted))
    }

    /// provides the names of all files that the current branch deletes or renames away,
    /// including uncommitted changes
    pub(crate) fn branch_removed_files(&self) -> Result<Vec<File>> {
        let uncommitted = self.uncommitted_removed()?;

        // try to use Git Town
        if let Some(gittown_files) = gittown::files_removed_on_current_branch(self) {
            return Ok(unique_missing(self, gittown_files, uncommitted));
        }

        // here Git Town didn't work --> use vanilla Git
        let Some(default_branch) = self.default_branch() else {
            // cannot determine the default branch --> process only the uncommitted files
            return Ok(uncommitted);
        };
        let Some(merge_base) = self.merge_base(&default_branch) else {
            // cannot determine the merge base --> process only the uncommitted files
            return Ok(uncommitted);
        };
        let committed = self.branch_committed_removed_files(&merge_base)?;
        Ok(unique_missing(self, committed, uncommitted))
    }
}

/// provides the actually existing files from both lists
fn unique_existing(repo: &Repo, mut files: Vec<File>, extra: Vec<File>) -> Vec<File> {
    files.extend(extra);
    files.sort();
    files.dedup();
    files.retain(|file| repo.file_exists(file));
    files
}

/// provides the files from both lists that don't exist in the workspace
fn unique_missing(repo: &Repo, mut files: Vec<File>, extra: Vec<File>) -> Vec<File> {
    files.extend(extra);
    files.sort();
    files.dedup();
    files.retain(|file| !repo.file_exists(file));
    files
}

#[cfg(test)]
mod tests {
    mod branch_changed_files {
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::GitCommandExt;
        use crate::git::Repo;
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
            let have = repo.branch_changed_files()?;
            let want = vec![File::from("on-branch.rs"), File::from("uncommitted.rs")];
            pretty::assert_eq!(have, want);
            Ok(())
        }
    }

    mod branch_removed_files {
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::GitCommandExt;
        use crate::git::Repo;
        use std::fs;
        use tempfile::TempDir;

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
                .args(["rm", "deleted-in-commit.md"])
                .run()?;
            repo.git_command()
                .args(["mv", "renamed-in-commit.md", "new-name.md"])
                .run()?;
            repo.git_command()
                .args(["commit", "--quiet", "--message=Remove"])
                .run()?;
            fs::remove_file(repo.file_path("deleted-uncommitted.md")).unwrap();
            let have = repo.branch_removed_files()?;
            let want = vec![
                File::from("deleted-in-commit.md"),
                File::from("deleted-uncommitted.md"),
                File::from("renamed-in-commit.md"),
            ];
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn file_recreated_after_committed_deletion() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("file.md")?;
            repo.create_and_switch_to_branch("feature")?;
            repo.git_command().args(["rm", "file.md"]).run()?;
            repo.git_command()
                .args(["commit", "--quiet", "--message=Remove"])
                .run()?;
            repo.create_unstaged_file("file.md");
            let have = repo.branch_removed_files()?;
            pretty::assert_eq!(have, Vec::<File>::new());
            Ok(())
        }
    }

    mod unique_existing {
        use super::super::unique_existing;
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::Repo;
        use tempfile::TempDir;

        #[test]
        fn empty() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            let have = unique_existing(&repo, vec![], vec![]);
            let want = Vec::<File>::new();
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn already_unique_and_sorted() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_unstaged_files(&["a.rs", "b.rs", "c.rs"]);
            let have = unique_existing(
                &repo,
                vec![File::from("a.rs"), File::from("b.rs")],
                vec![File::from("c.rs")],
            );
            let want = vec![File::from("a.rs"), File::from("b.rs"), File::from("c.rs")];
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn sorts() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_unstaged_files(&["a.rs", "b.rs", "c.rs"]);
            let have = unique_existing(
                &repo,
                vec![File::from("c.rs"), File::from("a.rs")],
                vec![File::from("b.rs")],
            );
            let want = vec![File::from("a.rs"), File::from("b.rs"), File::from("c.rs")];
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn dedups_across_both_lists() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_unstaged_files(&["a.rs", "b.rs"]);
            let have = unique_existing(
                &repo,
                vec![File::from("b.rs"), File::from("a.rs")],
                vec![File::from("b.rs"), File::from("a.rs")],
            );
            let want = vec![File::from("a.rs"), File::from("b.rs")];
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn dedups_within_one_list() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_unstaged_files(&["a.rs", "b.rs"]);
            let have = unique_existing(
                &repo,
                vec![File::from("b.rs"), File::from("a.rs"), File::from("b.rs")],
                vec![],
            );
            let want = vec![File::from("a.rs"), File::from("b.rs")];
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn drops_missing_files() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_unstaged_files(&["a.rs"]);
            let have = unique_existing(
                &repo,
                vec![File::from("gone.rs"), File::from("a.rs")],
                vec![File::from("also-gone.rs")],
            );
            let want = vec![File::from("a.rs")];
            pretty::assert_eq!(have, want);
            Ok(())
        }
    }
}
