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

use crate::cli::input::{CLIScope, ScopeFlag};
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

/// arguments for [`scoped_stacks`]
pub(crate) struct ScopedStacksArgs<'a> {
    /// value of the CLI `--scope` flag
    pub(crate) scope_flag: ScopeFlag,

    /// the scope to use when `--scope` is omitted, if none is given, use the smallest change: uncommitted files, or branch, or all files
    pub(crate) default_scope: Option<CLIScope>,

    /// the Git repository in the current directory
    pub(crate) repo: Option<&'a Repo>,

    /// the files to ignore
    pub(crate) ignores: Ignores,
}

/// provides the stacks and files that a command should process
pub(crate) fn scoped_stacks(
    ScopedStacksArgs {
        scope_flag,
        default_scope,
        repo,
        ignores,
    }: ScopedStacksArgs,
) -> ScopedStacks {
    let files = determine_files(&scope_flag, default_scope, repo, &ignores);
    group_into_stacks(&files, &ignores)
}

/// provides the files in the scope given via `--scope`,
/// otherwise in the default scope, otherwise in the inferred scope
///
/// Uses all files in the workspace if there is no Git repo or Git fails.
fn determine_files(
    scope_flag: &ScopeFlag,
    default_scope: Option<CLIScope>,
    repo: Option<&Repo>,
    ignores: &Ignores,
) -> FileChanges {
    let Some(repo) = repo else {
        return all_files(ignores);
    };
    let files = match scope_flag.scope.or(default_scope) {
        Some(scope) => files_in_scope(scope, repo, ignores),
        None => infer_files(repo, ignores),
    };
    files.unwrap_or_else(|err| {
        err.print();
        println!("processing all files instead");
        all_files(ignores)
    })
}

/// provides the files in the given scope
fn files_in_scope(scope: CLIScope, repo: &Repo, ignores: &Ignores) -> Result<FileChanges> {
    match scope {
        CLIScope::All => Ok(all_files(ignores)),
        CLIScope::Uncommitted => repo.uncommitted_changes(),
        CLIScope::Branch => repo.branch_changes(),
    }
}

/// provides the uncommitted files if there are any,
/// otherwise the files changed on the current branch if there are any,
/// otherwise all files
fn infer_files(repo: &Repo, ignores: &Ignores) -> Result<FileChanges> {
    let uncommitted = repo.uncommitted_changes()?;
    if !uncommitted.is_empty() {
        return Ok(uncommitted);
    }
    // here there are no uncommitted changes --> the branch changes are the committed ones
    let branch = repo.branch_committed_changes()?;
    if !branch.is_empty() {
        return Ok(branch);
    }
    Ok(all_files(ignores))
}

/// provides all files in the workspace
fn all_files(ignores: &Ignores) -> FileChanges {
    FileChanges::from_existing(stacks::workspace_files(ignores))
}

/// the stacks and files that a command operates on
pub(crate) struct ScopedStacks {
    /// the files in scope that exist in the workspace
    existing: DetectedStacks,

    /// the stacks that differ from `existing` because the scope removes files
    removed: Option<RemovedStacks>,
}

/// files that have been removed from the workspace
struct RemovedStacks {
    /// see [`ScopedStacks::changed`]
    changed: DetectedStacks,

    /// see [`ScopedStacks::lint`]
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

/// groups the given files into stacks
fn group_into_stacks(files: &FileChanges, ignores: &Ignores) -> ScopedStacks {
    let existing = stacks::from_files(files.changed(), ignores);
    if files.removed().next().is_none() {
        return ScopedStacks {
            existing,
            removed: None,
        };
    }
    let changed = stacks::from_files(files.all(), ignores);
    let lint = stacks::include_stacks_of_removed_files(&existing, files.removed(), ignores);
    ScopedStacks {
        existing,
        removed: Some(RemovedStacks { changed, lint }),
    }
}

#[cfg(test)]
mod tests {
    use super::{all_files, determine_files, files_in_scope, group_into_stacks, infer_files};
    use crate::cli::input::{CLIScope, ScopeFlag};
    use crate::domain::{DetectedStacks, File, Ignores, Result};
    use crate::git::FileStatus::{Changed, Removed};
    use crate::git::{FileChanges, GitCommandExt, Repo};
    use tempfile::TempDir;

    fn filenames(stacks: &DetectedStacks) -> Vec<File> {
        let mut files: Vec<File> = stacks
            .into_iter()
            .flat_map(|stack| stack.files.into_iter().cloned())
            .collect();
        files.sort();
        files
    }

    /// all files in the workspace
    fn all() -> FileChanges {
        all_files(&Ignores::empty())
    }

    #[test]
    fn uncommitted_includes_only_uncommitted_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("committed.md")?;
        repo.create_unstaged_file("untracked.md");
        let files = files_in_scope(CLIScope::Uncommitted, &repo, &Ignores::empty())?;
        pretty::assert_eq!(files.summary(), vec![("untracked.md", Changed)]);
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
        let files = files_in_scope(CLIScope::Branch, &repo, &Ignores::empty())?;
        let want = vec![("on-branch.md", Changed), ("untracked.md", Changed)];
        pretty::assert_eq!(files.summary(), want);
        Ok(())
    }

    #[test]
    fn every_scope_is_all_without_git_repo() {
        let want = all();
        for scope in [CLIScope::All, CLIScope::Uncommitted, CLIScope::Branch] {
            let flag = ScopeFlag { scope: Some(scope) };
            pretty::assert_eq!(determine_files(&flag, None, None, &Ignores::empty()), want);
            let no_flag = ScopeFlag { scope: None };
            pretty::assert_eq!(
                determine_files(&no_flag, Some(scope), None, &Ignores::empty()),
                want
            );
        }
    }

    #[test]
    fn stacks_of_removed_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_and_commit_file("deleted.md")?;
        repo.create_and_commit_file("untouched.md")?;
        repo.create_unstaged_file("untracked.md");
        std::fs::remove_file(repo.file_path("deleted.md")).unwrap();
        let files = files_in_scope(CLIScope::Uncommitted, &repo, &Ignores::empty())?;
        let stacks = group_into_stacks(&files, &Ignores::empty());
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
        let files = files_in_scope(CLIScope::Uncommitted, &repo, &Ignores::empty())?;
        let stacks = group_into_stacks(&files, &Ignores::empty());
        let want = vec![File::from("untracked.md")];
        pretty::assert_eq!(filenames(stacks.fix()), want);
        pretty::assert_eq!(filenames(stacks.changed()), want);
        pretty::assert_eq!(filenames(stacks.lint()), want);
        Ok(())
    }

    #[test]
    fn infer_all_without_git_repo() {
        let no_flag = ScopeFlag { scope: None };
        pretty::assert_eq!(
            determine_files(&no_flag, None, None, &Ignores::empty()),
            all()
        );
    }

    #[test]
    fn infer_uncommitted_when_uncommitted_files_exist() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        repo.create_and_commit_file("on-branch.md").unwrap();
        repo.create_unstaged_file("untracked.md");
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo), &Ignores::empty());
        pretty::assert_eq!(have.summary(), vec![("untracked.md", Changed)]);
    }

    #[test]
    fn infer_branch_when_clean_tree_has_branch_changes() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        repo.create_and_commit_file("on-branch.md").unwrap();
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo), &Ignores::empty());
        pretty::assert_eq!(have.summary(), vec![("on-branch.md", Changed)]);
    }

    #[test]
    fn infer_uncommitted_when_only_uncommitted_deletions_exist() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        std::fs::remove_file(repo.file_path("on-main.md")).unwrap();
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo), &Ignores::empty());
        pretty::assert_eq!(have.summary(), vec![("on-main.md", Removed)]);
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
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo), &Ignores::empty());
        pretty::assert_eq!(have.summary(), vec![("on-main.md", Removed)]);
    }

    #[test]
    fn infer_all_when_clean_tree_on_main() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo), &Ignores::empty());
        pretty::assert_eq!(have, all());
    }

    #[test]
    fn infer_all_when_clean_tree_on_empty_feature_branch() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo), &Ignores::empty());
        pretty::assert_eq!(have, all());
    }

    #[test]
    fn resolve_uses_explicit_scope() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let explicit = ScopeFlag {
            scope: Some(CLIScope::All),
        };
        let have = determine_files(&explicit, None, Some(&repo), &Ignores::empty());
        pretty::assert_eq!(have, all());
        Ok(())
    }

    #[test]
    fn resolve_infers_when_scope_omitted() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let omitted = ScopeFlag { scope: None };
        let have = determine_files(&omitted, None, Some(&repo), &Ignores::empty());
        pretty::assert_eq!(have.summary(), vec![("untracked.md", Changed)]);
        Ok(())
    }

    #[test]
    fn uses_default_scope_when_flag_omitted() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let omitted = ScopeFlag { scope: None };
        let have = determine_files(
            &omitted,
            Some(CLIScope::All),
            Some(&repo),
            &Ignores::empty(),
        );
        pretty::assert_eq!(have, all());
        Ok(())
    }

    #[test]
    fn flag_overrides_default_scope() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let explicit = ScopeFlag {
            scope: Some(CLIScope::Uncommitted),
        };
        let have = determine_files(
            &explicit,
            Some(CLIScope::All),
            Some(&repo),
            &Ignores::empty(),
        );
        pretty::assert_eq!(have.summary(), vec![("untracked.md", Changed)]);
        Ok(())
    }

    #[test]
    fn git_failure_is_an_error() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        std::fs::remove_dir_all(dir.path().join(".git")).unwrap();
        let ignores = Ignores::empty();
        assert!(files_in_scope(CLIScope::Uncommitted, &repo, &ignores).is_err());
        assert!(files_in_scope(CLIScope::Branch, &repo, &ignores).is_err());
        assert!(infer_files(&repo, &ignores).is_err());
        Ok(())
    }

    #[test]
    fn git_failure_uses_all_files() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        std::fs::remove_dir_all(dir.path().join(".git")).unwrap();
        let want = all();
        for scope in [Some(CLIScope::Uncommitted), Some(CLIScope::Branch), None] {
            let flag = ScopeFlag { scope };
            let have = determine_files(&flag, None, Some(&repo), &Ignores::empty());
            pretty::assert_eq!(have, want);
        }
        Ok(())
    }
}
