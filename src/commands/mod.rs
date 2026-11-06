mod ci;
mod fix;
mod fix_unsafe;
mod full;
pub mod init_config;
mod lint;
mod pitstop;
mod post_edit;
mod precommit;
mod test;
mod update_tools;

use crate::cli::input::{Scope, ScopeArg};
use crate::domain::{DetectedStacks, Ignores, Result};
use crate::git::Repo;
use crate::stacks;
pub use ci::ci;
pub use fix::fix;
pub use fix_unsafe::fix_unsafe;
pub use full::full;
pub use init_config::init_config;
pub use lint::lint;
pub use pitstop::pitstop;
pub use post_edit::post_edit;
pub use precommit::precommit;
pub use test::test;
pub use update_tools::update_tools;

/// uses the explicit `--scope` if given, otherwise [`infer_scope`]
pub(crate) fn resolve_scope(explicit: &ScopeArg, repo: Option<&Repo>) -> Scope {
    match explicit.scope {
        Some(scope) => scope,
        None => infer_scope(repo),
    }
}

/// infers the file scope for `fix`, `fix-unsafe`, `lint`, `pitstop`, and `test` when `--scope` is omitted
///
/// Uses uncommitted changes if any exist, otherwise changes on the current
/// branch, otherwise all files. Without a Git repository, this is always `all`.
/// Deleted files count as changes.
pub(crate) fn infer_scope(repo: Option<&Repo>) -> Scope {
    let Some(repo) = repo else {
        return Scope::All;
    };
    let Ok(uncommitted) = repo.uncommitted() else {
        // error running Git --> check all files
        return Scope::All;
    };
    let Ok(uncommitted_removed) = repo.uncommitted_removed() else {
        // error running Git --> check all files
        return Scope::All;
    };
    if !uncommitted.is_empty() || !uncommitted_removed.is_empty() {
        // uncommitted changes exist --> check only those
        return Scope::Uncommitted;
    }
    let Ok(branch_changed) = repo.branch_changed_files() else {
        // error running Git --> check all files
        return Scope::All;
    };
    if !branch_changed.is_empty() {
        return Scope::Branch;
    }
    let Ok(branch_deleted) = repo.branch_deleted_files() else {
        // error running Git --> check all files
        return Scope::All;
    };
    if branch_deleted.is_empty() {
        // no branch changes --> check all files
        return Scope::All;
    }
    Scope::Branch
}

/// provides the stacks that have changes in the given file scope
///
/// Deleted files count as changes. Their stacks are part of the result, but not the files themselves.
/// When the current directory is not a Git repository, every scope scans all files.
pub(crate) fn discover_stacks(
    scope: Scope,
    repo: Option<&Repo>,
    ignores: &Ignores,
) -> Result<DetectedStacks> {
    let Some(repo) = repo else {
        return Ok(stacks::discover_all(ignores));
    };
    match scope {
        Scope::All => {
            // processing all files doesn't otherwise need Git, so don't fail if Git doesn't work
            let deleted = repo.branch_deleted_files().unwrap_or_default();
            Ok(stacks::with_deleted(
                stacks::discover_all(ignores),
                &deleted,
                ignores,
            ))
        }
        Scope::Uncommitted => Ok(stacks::with_deleted(
            stacks::from_files(&repo.uncommitted()?, ignores),
            &repo.uncommitted_deleted()?,
            ignores,
        )),
        Scope::Branch => Ok(stacks::with_deleted(
            stacks::from_files(&repo.branch_changed_files()?, ignores),
            &repo.branch_deleted_files()?,
            ignores,
        )),
    }
    let changed = stacks::include_removed_files(&existing, &removed, ignores);
    let lint = stacks::include_stacks_of_removed_files(&existing, &removed, ignores);
    Ok(ScopedStacks {
        existing,
        removed: Some(RemovedStacks { changed, lint }),
    })
}

#[cfg(test)]
mod tests {
    use super::{discover_stacks, infer_scope, resolve_scope};
    use crate::cli::input::{Scope, ScopeArg};
    use crate::domain::{DetectedStacks, File, Ignores, Result, StackType};
    use crate::git::{GitCommandExt, Repo};
    use std::fs;
    use tempfile::TempDir;

    fn filenames(stacks: &DetectedStacks) -> Vec<File> {
        let mut files: Vec<File> = stacks
            .into_iter()
            .flat_map(|stack| stack.files.into_iter().cloned())
            .collect();
        files.sort();
        files
    }

    #[test]
    fn uncommitted_includes_only_uncommitted_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("committed.md")?;
        repo.create_unstaged_file("untracked.md");
        let stacks = discover_stacks(Scope::Uncommitted, Some(&repo), &Ignores::empty())?;
        pretty::assert_eq!(filenames(stacks.fix()), vec![File::from("untracked.md")]);
        Ok(())
    }

    #[test]
    fn branch_includes_committed_branch_files_and_uncommitted_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("on-main.md")?;
        repo.create_and_switch_to_branch("feature")?;
        repo.create_and_commit_file("on-branch.md")?;
        repo.create_unstaged_file("untracked.md");
        let stacks = discover_stacks(Scope::Branch, Some(&repo), &Ignores::empty())?;
        pretty::assert_eq!(
            filenames(stacks.fix()),
            vec![File::from("on-branch.md"), File::from("untracked.md")]
        );
        Ok(())
    }

    #[test]
    fn changed_stacks_include_removed_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("deleted.md")?;
        repo.create_and_commit_file("untouched.md")?;
        repo.create_unstaged_file("untracked.md");
        std::fs::remove_file(repo.file_path("deleted.md")).unwrap();
        let stacks = discover_stacks(Scope::Uncommitted, Some(&repo), &Ignores::empty())?;
        pretty::assert_eq!(filenames(stacks.fix()), vec![File::from("untracked.md")]);
        pretty::assert_eq!(
            filenames(stacks.changed()),
            vec![File::from("deleted.md"), File::from("untracked.md")]
        );
        Ok(())
    }

    #[test]
    fn infer_all_without_git_repo() {
        pretty::assert_eq!(infer_scope(None), Scope::All);
    }

    #[test]
    fn infer_uncommitted_when_uncommitted_files_exist() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        repo.create_and_commit_file("on-branch.md").unwrap();
        repo.create_unstaged_file("untracked.md");
        let have = infer_scope(Some(&repo));
        let want = Scope::Uncommitted;
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn infer_branch_when_clean_tree_has_branch_changes() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        repo.create_and_commit_file("on-branch.md").unwrap();
        let have = infer_scope(Some(&repo));
        let want = Scope::Branch;
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn infer_uncommitted_when_only_uncommitted_deletions_exist() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        std::fs::remove_file(repo.file_path("on-main.md")).unwrap();
        let have = infer_scope(Some(&repo));
        let want = Scope::Uncommitted;
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn infer_branch_when_branch_only_deletes_files() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        repo.git_command()
            .args(["rm", "--quiet", "on-main.md"])
            .run()
            .unwrap();
        repo.git_command()
            .args(["commit", "--quiet", "--message=Delete"])
            .run()
            .unwrap();
        let have = infer_scope(Some(&repo));
        let want = Scope::Branch;
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn infer_all_when_clean_tree_on_main() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        let have = infer_scope(Some(&repo));
        let want = Scope::All;
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn infer_all_when_clean_tree_on_empty_feature_branch() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        let have = infer_scope(Some(&repo));
        let want = Scope::All;
        pretty::assert_eq!(have, want);
    }

    /// a repo with committed Rust and Markdown files on a feature branch without changes
    fn feature_branch(dir: &TempDir) -> Result<Repo> {
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("gone.rs")?;
        repo.create_and_commit_file("kept.rs")?;
        repo.create_and_commit_file("kept.md")?;
        repo.create_and_switch_to_branch("feature")?;
        Ok(repo)
    }

    fn commit_deletion(repo: &Repo, file: &str) -> Result<()> {
        repo.git_command().args(["rm", "--quiet", file]).run()?;
        repo.git_command()
            .args(["commit", "--quiet", "--message=delete"])
            .run()
    }

    #[test]
    fn infer_uncommitted_when_only_uncommitted_deletions_exist() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        repo.create_and_commit_file("on-branch.md")?;
        fs::remove_file(repo.file_path("gone.rs")).unwrap();
        pretty::assert_eq!(infer_scope(Some(&repo)), Scope::Uncommitted);
        Ok(())
    }

    #[test]
    fn infer_branch_when_branch_only_deletes_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        commit_deletion(&repo, "gone.rs")?;
        pretty::assert_eq!(infer_scope(Some(&repo)), Scope::Branch);
        Ok(())
    }

    #[test]
    fn uncommitted_scope_includes_stacks_of_uncommitted_deletions() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        fs::remove_file(repo.file_path("gone.rs")).unwrap();
        let stacks = discover_stacks(Scope::Uncommitted, Some(&repo), &Ignores::empty())?;
        pretty::assert_eq!(filenames(&stacks), Vec::<File>::new());
        assert!(stacks.get_stack(StackType::Rust).is_some());
        Ok(())
    }

    #[test]
    fn uncommitted_scope_ignores_committed_deletions() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        commit_deletion(&repo, "gone.rs")?;
        let stacks = discover_stacks(Scope::Uncommitted, Some(&repo), &Ignores::empty())?;
        assert!(stacks.is_empty());
        Ok(())
    }

    #[test]
    fn branch_scope_includes_stacks_of_committed_deletions() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        repo.create_and_commit_file("on-branch.md")?;
        commit_deletion(&repo, "gone.rs")?;
        let stacks = discover_stacks(Scope::Branch, Some(&repo), &Ignores::empty())?;
        pretty::assert_eq!(filenames(&stacks), vec![File::from("on-branch.md")]);
        assert!(stacks.get_stack(StackType::Rust).is_some());
        Ok(())
    }

    #[test]
    fn scopes_skip_ignored_deleted_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        commit_deletion(&repo, "gone.rs")?;
        let ignores = Ignores::new(&["gone.rs".to_string()], std::path::Path::new("./"))?;
        let stacks = discover_stacks(Scope::Branch, Some(&repo), &ignores)?;
        assert!(stacks.is_empty());
        Ok(())
    }

    #[test]
    fn resolve_uses_explicit_scope() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_unstaged_file("untracked.md");
        let explicit = ScopeArg {
            scope: Some(Scope::All),
        };
        let have = resolve_scope(&explicit, Some(&repo));
        let want = Scope::All;
        pretty::assert_eq!(have, want);
    }

    #[test]
    fn resolve_infers_when_scope_omitted() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_unstaged_file("untracked.md");
        let omitted = ScopeArg { scope: None };
        let have = resolve_scope(&omitted, Some(&repo));
        let want = Scope::Uncommitted;
        pretty::assert_eq!(have, want);
    }
}
