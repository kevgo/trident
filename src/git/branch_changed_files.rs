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

    /// provides the names of all files deleted on the current branch, including uncommitted deletions
    /// and the old names of renamed files
    pub(crate) fn branch_deleted_files(&self) -> Result<Vec<File>> {
        let mut files = self.uncommitted_deleted()?;
        if let Some(gittown_files) = gittown::files_changed_on_current_branch(self) {
            files.extend(gittown_files);
        } else if let Some(merge_base) = self
            .default_branch()
            .and_then(|default_branch| self.merge_base(&default_branch))
        {
            files.extend(self.branch_committed_deleted_files(&merge_base)?);
        }
        files.retain(|file| !self.file_exists(file));
        files.sort();
        files.dedup();
        Ok(files)
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

    mod branch_deleted_files {
        use crate::domain::{File, Result};
        use crate::git::{GitCommandExt, Repo};
        use std::fs;
        use std::process::Stdio;
        use tempfile::TempDir;

        fn commit_all(repo: &Repo) -> Result<()> {
            repo.git_command().args(["add", "--all"]).run()?;
            repo.git_command()
                .args(["commit", "--quiet", "--message=change"])
                .run()
        }

        fn feature_branch(dir: &TempDir) -> Result<Repo> {
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("kept.rs")?;
            repo.create_and_commit_file("gone.rs")?;
            repo.create_and_switch_to_branch("feature")?;
            Ok(repo)
        }

        #[test]
        fn committed_deletion() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = feature_branch(&dir)?;
            fs::remove_file(repo.file_path("gone.rs")).unwrap();
            commit_all(&repo)?;
            pretty::assert_eq!(repo.branch_deleted_files()?, vec![File::from("gone.rs")]);
            pretty::assert_eq!(repo.branch_changed_files()?, Vec::<File>::new());
            Ok(())
        }

        #[test]
        fn uncommitted_deletion() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = feature_branch(&dir)?;
            fs::remove_file(repo.file_path("gone.rs")).unwrap();
            pretty::assert_eq!(repo.branch_deleted_files()?, vec![File::from("gone.rs")]);
            Ok(())
        }

        #[test]
        fn committed_rename_deletes_the_old_name() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = feature_branch(&dir)?;
            repo.git_command().args(["mv", "gone.rs", "new.md"]).run()?;
            commit_all(&repo)?;
            pretty::assert_eq!(repo.branch_deleted_files()?, vec![File::from("gone.rs")]);
            Ok(())
        }

        #[test]
        fn file_added_and_deleted_on_the_branch() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = feature_branch(&dir)?;
            repo.create_and_commit_file("temp.rs")?;
            fs::remove_file(repo.file_path("temp.rs")).unwrap();
            commit_all(&repo)?;
            pretty::assert_eq!(repo.branch_deleted_files()?, Vec::<File>::new());
            Ok(())
        }

        #[test]
        fn on_main_branch() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.create_and_commit_file("kept.rs")?;
            pretty::assert_eq!(repo.branch_deleted_files()?, Vec::<File>::new());
            Ok(())
        }

        #[test]
        fn git_town_branch() -> Result<()> {
            if which::which("git-town").is_err() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.git_command()
                .args(["config", "git-town.main-branch", "main"])
                .run()?;
            repo.create_and_commit_file("gone.rs")?;
            repo.create_and_commit_file("renamed.rs")?;
            repo.git_command()
                .args(["town", "append", "feature"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .run()?;
            fs::remove_file(repo.file_path("gone.rs")).unwrap();
            repo.git_command()
                .args(["mv", "renamed.rs", "new.md"])
                .run()?;
            commit_all(&repo)?;
            let want = vec![File::from("gone.rs"), File::from("renamed.rs")];
            pretty::assert_eq!(repo.branch_deleted_files()?, want);
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
