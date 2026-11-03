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
use crate::config::Config;
use crate::domain::{DetectedStacks, Ignores, Result, StackType};
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

/// provides the stacks whose tests should run
///
/// Other than for lints and fixes, deleted files count as changes here,
/// because deleting a file can break the remaining code of its stack.
/// When no scope is given, this uses [`infer_test_scope`].
pub(crate) fn stacks_to_test(
    config: &Config,
    scope: Option<Scope>,
    repo: Option<&Repo>,
    ignores: &Ignores,
) -> Result<Vec<StackType>> {
    if !config.has_stack_tests() {
        return Ok(Vec::new());
    }
    let scope = scope.unwrap_or_else(|| infer_test_scope(repo));
    let existing = discover_stacks(scope, repo, ignores)?;
    let deleted_files = match (scope, repo) {
        (_, None) => Vec::new(),
        (Scope::Uncommitted, Some(repo)) => repo.uncommitted_deleted()?,
        (Scope::Branch | Scope::All, Some(repo)) => repo.branch_deleted_files()?,
    };
    let deleted = stacks::from_files(&deleted_files, ignores);
    Ok(stacks::all()
        .iter()
        .map(|stack| stack.stack_type())
        .filter(|stack_type| {
            existing.get_stack(*stack_type).is_some() || deleted.get_stack(*stack_type).is_some()
        })
        .collect())
}

/// infers the file scope for tests when `--scope` is omitted
///
/// Works like [`infer_scope`], but also considers deleted files to be changes.
fn infer_test_scope(repo: Option<&Repo>) -> Scope {
    let inferred = infer_scope(repo);
    let Some(repo) = repo else {
        return inferred;
    };
    if inferred == Scope::Uncommitted {
        return inferred;
    }
    if repo
        .uncommitted_deleted()
        .is_ok_and(|files| !files.is_empty())
    {
        return Scope::Uncommitted;
    }
    if repo
        .branch_deleted_files()
        .is_ok_and(|files| !files.is_empty())
    {
        return Scope::Branch;
    }
    inferred
}

#[cfg(test)]
mod tests {
    use super::{discover_stacks, infer_scope, infer_test_scope, resolve_scope, stacks_to_test};
    use crate::cli::input::{Scope, ScopeArg};
    use crate::config::Config;
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

    fn config_with_stack_tests() -> Config {
        Config::parse(
            r#"{ "stacks": { "rust": { "tests": [{ "command": "echo rust" }] } } }"#,
            "trident.json",
        )
        .unwrap()
    }

    #[test]
    fn infer_test_uncommitted_when_only_uncommitted_deletions_exist() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        repo.create_and_commit_file("on-branch.md")?;
        fs::remove_file(repo.file_path("gone.rs")).unwrap();
        pretty::assert_eq!(infer_scope(Some(&repo)), Scope::Branch);
        pretty::assert_eq!(infer_test_scope(Some(&repo)), Scope::Uncommitted);
        Ok(())
    }

    #[test]
    fn infer_test_branch_when_branch_only_deletes_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        commit_deletion(&repo, "gone.rs")?;
        pretty::assert_eq!(infer_scope(Some(&repo)), Scope::All);
        pretty::assert_eq!(infer_test_scope(Some(&repo)), Scope::Branch);
        Ok(())
    }

    #[test]
    fn infer_test_all_without_changes() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        pretty::assert_eq!(infer_test_scope(Some(&repo)), Scope::All);
        Ok(())
    }

    #[test]
    fn infer_test_all_without_git_repo() {
        pretty::assert_eq!(infer_test_scope(None), Scope::All);
    }

    #[test]
    fn stacks_to_test_includes_stack_of_deleted_file() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        repo.create_and_commit_file("on-branch.md")?;
        commit_deletion(&repo, "gone.rs")?;
        let have = stacks_to_test(
            &config_with_stack_tests(),
            Some(Scope::Branch),
            Some(&repo),
            &Ignores::empty(),
        )?;
        pretty::assert_eq!(have, vec![StackType::Markdown, StackType::Rust]);
        Ok(())
    }

    #[test]
    fn stacks_to_test_skips_ignored_deleted_file() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        commit_deletion(&repo, "gone.rs")?;
        let ignores = Ignores::new(&["gone.rs".to_string()], std::path::Path::new("./"))?;
        let have = stacks_to_test(
            &config_with_stack_tests(),
            Some(Scope::Branch),
            Some(&repo),
            &ignores,
        )?;
        pretty::assert_eq!(have, Vec::<StackType>::new());
        Ok(())
    }

    #[test]
    fn stacks_to_test_without_configured_stack_tests() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = feature_branch(&dir)?;
        commit_deletion(&repo, "gone.rs")?;
        let have = stacks_to_test(
            &Config::default(),
            Some(Scope::Branch),
            Some(&repo),
            &Ignores::empty(),
        )?;
        pretty::assert_eq!(have, Vec::<StackType>::new());
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
