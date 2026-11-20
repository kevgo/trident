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

use crate::cli::input::{CLIScope, ScopeArg};
use crate::domain::{DetectedStacks, Ignores, Result};
use crate::git::{FileChanges, Repo};
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

/// the scope that a command should operate on
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum FileScope {
    /// process all files in the workspace
    All,

    /// process only on these changed files
    Changes(FileChanges),
}

/// provides the files in the explicit `--scope` if given, otherwise in the [inferred scope](infer_scope)
pub(crate) fn resolve_scope(explicit: &ScopeArg, repo: Option<&Repo>) -> Result<FileScope> {
    match explicit.scope {
        Some(scope) => files_in_scope(scope, repo),
        None => Ok(infer_scope(repo)),
    }
}

/// infers the file scope for `fix`, `fix-unsafe`, `lint`, and `pitstop` when `--scope` is omitted
/// and provides the files in it
///
/// Uses uncommitted changes if any exist, otherwise changes on the current
/// branch, otherwise all files. Without a Git repository, this is always `all`.
pub(crate) fn infer_scope(repo: Option<&Repo>) -> FileScope {
    let Some(repo) = repo else {
        return FileScope::All;
    };
    let Ok(uncommitted) = repo.uncommitted_changes() else {
        // error running Git --> check all files
        return FileScope::All;
    };
    if !uncommitted.is_empty() {
        // uncommitted changes exist --> check only those
        return FileScope::Changes(uncommitted);
    }
    // here there are no uncommitted changes --> the branch changes are the committed ones
    let Ok(branch) = repo.branch_committed_changes() else {
        // error running Git --> check all files
        return FileScope::All;
    };
    if branch.is_empty() {
        // no branch changes --> check all files
        return FileScope::All;
    }
    FileScope::Changes(branch)
}

/// provides the files in the given scope
///
/// When the current directory is not a Git repository, every scope contains all files.
pub(crate) fn files_in_scope(scope: CLIScope, repo: Option<&Repo>) -> Result<FileScope> {
    Ok(match (scope, repo) {
        (CLIScope::All, _) | (_, None) => FileScope::All,
        (CLIScope::Uncommitted, Some(repo)) => FileScope::Changes(repo.uncommitted_changes()?),
        (CLIScope::Branch, Some(repo)) => FileScope::Changes(repo.branch_changes()?),
    })
}

/// the stacks and files that a command operates on
pub(crate) struct ScopedStacks {
    /// the files in scope that exist in the workspace
    existing: DetectedStacks,

    /// the stacks that differ from `existing` because the scope removes files
    removed: Option<RemovedStacks>,
}

/// see [`ScopedStacks::changed`] and [`ScopedStacks::lint`]
struct RemovedStacks {
    changed: DetectedStacks,
    lint: DetectedStacks,
}

impl ScopedStacks {
    /// all files in the workspace
    pub(crate) fn all(ignores: &Ignores) -> ScopedStacks {
        ScopedStacks {
            existing: stacks::discover_all(ignores),
            removed: None,
        }
    }

    /// all files that the scope changes, including the ones it deletes or renames away
    ///
    /// This is what the user sees as the files in scope.
    pub(crate) fn changed(&self) -> &DetectedStacks {
        match &self.removed {
            Some(removed) => &removed.changed,
            None => &self.existing,
        }
    }

    /// the files to fix: the changed files that still exist in the workspace
    pub(crate) fn fix(&self) -> &DetectedStacks {
        &self.existing
    }

    /// the files to lint: the files to fix,
    /// plus all files in the stacks of the files that the scope deletes or renames away
    /// because those files might still refer to the removed file
    pub(crate) fn lint(&self) -> &DetectedStacks {
        match &self.removed {
            Some(removed) => &removed.lint,
            None => &self.existing,
        }
    }
}

/// provides the stacks for the given files
pub(crate) fn discover_stacks(files: &FileScope, ignores: &Ignores) -> ScopedStacks {
    let FileScope::Changes(changes) = files else {
        return ScopedStacks::all(ignores);
    };
    let existing = stacks::from_files(changes.changed(), ignores);
    if changes.removed().next().is_none() {
        return ScopedStacks {
            existing,
            removed: None,
        };
    }
    let changed = stacks::from_files(changes.all(), ignores);
    let lint = stacks::include_stacks_of_removed_files(&existing, changes.removed(), ignores);
    ScopedStacks {
        existing,
        removed: Some(RemovedStacks { changed, lint }),
    }
}

#[cfg(test)]
mod tests {
    use super::{FileScope, discover_stacks, files_in_scope, infer_scope, resolve_scope};
    use crate::cli::input::{CLIScope, ScopeArg};
    use crate::domain::{DetectedStacks, File, Ignores, Result};
    use crate::git::FileStatus::{self, Changed, Removed};
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

    /// provides the files in the given scope with their status, None for all files
    fn summary(files: &FileScope) -> Option<Vec<(&str, FileStatus)>> {
        match files {
            FileScope::All => None,
            FileScope::Changes(changes) => Some(changes.summary()),
        }
    }

    #[test]
    fn uncommitted_includes_only_uncommitted_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("committed.md")?;
        repo.create_unstaged_file("untracked.md");
        let files = files_in_scope(CLIScope::Uncommitted, Some(&repo))?;
        pretty::assert_eq!(summary(&files), Some(vec![("untracked.md", Changed)]));
        Ok(())
    }

    #[test]
    fn branch_includes_committed_and_uncommitted_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("on-main.md")?;
        repo.create_and_switch_to_branch("feature")?;
        repo.create_and_commit_file("on-branch.md")?;
        repo.create_unstaged_file("untracked.md");
        let files = files_in_scope(CLIScope::Branch, Some(&repo))?;
        let want = vec![("on-branch.md", Changed), ("untracked.md", Changed)];
        pretty::assert_eq!(summary(&files), Some(want));
        Ok(())
    }

    #[test]
    fn every_scope_is_all_without_git_repo() -> Result<()> {
        for scope in [CLIScope::All, CLIScope::Uncommitted, CLIScope::Branch] {
            pretty::assert_eq!(files_in_scope(scope, None)?, FileScope::All);
        }
        Ok(())
    }

    #[test]
    fn stacks_of_removed_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("deleted.md")?;
        repo.create_and_commit_file("untouched.md")?;
        repo.create_unstaged_file("untracked.md");
        std::fs::remove_file(repo.file_path("deleted.md")).unwrap();
        let files = files_in_scope(CLIScope::Uncommitted, Some(&repo))?;
        let stacks = discover_stacks(&files, &Ignores::empty());
        pretty::assert_eq!(filenames(stacks.fix()), vec![File::from("untracked.md")]);
        pretty::assert_eq!(
            filenames(stacks.changed()),
            vec![File::from("deleted.md"), File::from("untracked.md")]
        );
        Ok(())
    }

    #[test]
    fn stacks_without_removed_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("untouched.md")?;
        repo.create_unstaged_file("untracked.md");
        let files = files_in_scope(CLIScope::Uncommitted, Some(&repo))?;
        let stacks = discover_stacks(&files, &Ignores::empty());
        let want = vec![File::from("untracked.md")];
        pretty::assert_eq!(filenames(stacks.fix()), want);
        pretty::assert_eq!(filenames(stacks.changed()), want);
        pretty::assert_eq!(filenames(stacks.lint()), want);
        Ok(())
    }

    #[test]
    fn infer_all_without_git_repo() {
        pretty::assert_eq!(infer_scope(None), FileScope::All);
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
        pretty::assert_eq!(summary(&have), Some(vec![("untracked.md", Changed)]));
    }

    #[test]
    fn infer_branch_when_clean_tree_has_branch_changes() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        repo.create_and_commit_file("on-branch.md").unwrap();
        let have = infer_scope(Some(&repo));
        pretty::assert_eq!(summary(&have), Some(vec![("on-branch.md", Changed)]));
    }

    #[test]
    fn infer_uncommitted_when_only_uncommitted_deletions_exist() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        std::fs::remove_file(repo.file_path("on-main.md")).unwrap();
        let have = infer_scope(Some(&repo));
        pretty::assert_eq!(summary(&have), Some(vec![("on-main.md", Removed)]));
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
        pretty::assert_eq!(summary(&have), Some(vec![("on-main.md", Removed)]));
    }

    #[test]
    fn infer_all_when_clean_tree_on_main() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        pretty::assert_eq!(infer_scope(Some(&repo)), FileScope::All);
    }

    #[test]
    fn infer_all_when_clean_tree_on_empty_feature_branch() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        pretty::assert_eq!(infer_scope(Some(&repo)), FileScope::All);
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
    fn resolve_uses_explicit_scope() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let explicit = ScopeArg {
            scope: Some(CLIScope::All),
        };
        let have = resolve_scope(&explicit, Some(&repo))?;
        pretty::assert_eq!(have, FileScope::All);
        Ok(())
    }

    #[test]
    fn resolve_infers_when_scope_omitted() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let omitted = ScopeArg { scope: None };
        let have = resolve_scope(&omitted, Some(&repo))?;
        pretty::assert_eq!(summary(&have), Some(vec![("untracked.md", Changed)]));
        Ok(())
    }
}
