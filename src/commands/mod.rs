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

/// infers the file scope for `fix`, `fix-unsafe`, `lint`, and `pitstop` when `--scope` is omitted
///
/// Uses uncommitted files if any exist, otherwise files changed on the current
/// branch, otherwise all files. Without a Git repository, this is always `all`.
pub(crate) fn infer_scope(repo: Option<&Repo>) -> Scope {
    let Some(repo) = repo else {
        return Scope::All;
    };
    let Ok(uncommitted) = repo.uncommitted() else {
        // error running Git --> check all files
        return Scope::All;
    };
    if !uncommitted.is_empty() {
        // uncommitted files exist --> check only those
        return Scope::Uncommitted;
    }
    let Ok(branch_changed) = repo.branch_changed_files() else {
        // error running Git --> check all files
        return Scope::All;
    };
    if branch_changed.is_empty() {
        // no branch changes --> check all files
        return Scope::All;
    }
    Scope::Branch
}

/// provides the stacks that match the given file scope
///
/// When the current directory is not a Git repository, every scope scans all files.
pub(crate) fn discover_stacks(
    scope: Scope,
    repo: Option<&Repo>,
    ignores: &Ignores,
) -> Result<DetectedStacks> {
    match (scope, repo) {
        (Scope::All, _) | (_, None) => Ok(stacks::discover_all(ignores)),
        (Scope::Uncommitted, Some(repo)) => Ok(stacks::from_files(&repo.uncommitted()?, ignores)),
        (Scope::Branch, Some(repo)) => {
            Ok(stacks::from_files(&repo.branch_changed_files()?, ignores))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{discover_stacks, infer_scope, resolve_scope};
    use crate::cli::input::{Scope, ScopeArg};
    use crate::domain::{DetectedStacks, File, Ignores, Result};
    use crate::git::Repo;
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
        pretty::assert_eq!(filenames(&stacks), vec![File::from("untracked.md")]);
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
            filenames(&stacks),
            vec![File::from("on-branch.md"), File::from("untracked.md")]
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
