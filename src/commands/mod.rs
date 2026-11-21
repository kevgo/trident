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
    // TODO: implement this more straightforward and step by step
    //
    // The current implementation is weird:
    //
    // - determine only the changed files, but not all files in the workspace
    //   - this gets stored in FileScope
    //   - if no files changed: return a placeholder to use all files instead of loading the files
    // - so now we sometimes have files, and sometimes not
    // - now group these changed files into stacks
    //   - if we have the files: group them directly
    //   - if we don't have files: load them from the workspace and group them
    //
    // This can be streamlined.
    //
    // step 1: determine the files to process
    //   - based on the given scope, either using Git or from the filesystem
    //   - Git error: print warning and use filesystem
    // step 2: group these file changes into stacks
    let files = match determine_files(&scope_flag, default_scope, repo) {
        Ok(files) => files,
        Err(err) => {
            err.print();
            println!("processing all files instead");
            FileScope::All
        }
    };
    discover_stacks(&files, &ignores)
}

/// the files that a command should process
#[derive(Debug, Eq, PartialEq)]
enum FileScope {
    /// process all files in the workspace
    All,

    /// process only these changed files
    Changes(FileChanges),
}

/// provides the files in the scope given via `--scope`,
/// otherwise in the default scope, otherwise in the inferred scope
fn determine_files(
    scope_flag: &ScopeFlag,
    default_scope: Option<CLIScope>,
    repo: Option<&Repo>,
) -> Result<FileScope> {
    let Some(repo) = repo else {
        return Ok(FileScope::All);
    };
    match scope_flag.scope.or(default_scope) {
        Some(scope) => files_in_scope(scope, repo),
        None => infer_files(repo),
    }
}

/// provides the files in the given scope
fn files_in_scope(scope: CLIScope, repo: &Repo) -> Result<FileScope> {
    Ok(match scope {
        CLIScope::All => FileScope::All,
        CLIScope::Uncommitted => FileScope::Changes(repo.uncommitted_changes()?),
        CLIScope::Branch => FileScope::Changes(repo.branch_changes()?),
    })
}

/// provides the uncommitted files if there are any,
/// otherwise the files changed on the current branch if there are any,
/// otherwise all files
fn infer_files(repo: &Repo) -> Result<FileScope> {
    let uncommitted = repo.uncommitted_changes()?;
    if !uncommitted.is_empty() {
        return Ok(FileScope::Changes(uncommitted));
    }
    // here there are no uncommitted changes --> the branch changes are the committed ones
    let branch = repo.branch_committed_changes()?;
    if branch.is_empty() {
        return Ok(FileScope::All);
    }
    Ok(FileScope::Changes(branch))
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

/// provides the stacks for the given files
fn discover_stacks(files: &FileScope, ignores: &Ignores) -> ScopedStacks {
    let changes = match files {
        FileScope::All => return ScopedStacks::all(ignores),
        FileScope::Changes(changes) => changes,
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
    use super::{FileScope, determine_files, discover_stacks, files_in_scope};
    use crate::cli::input::{CLIScope, ScopeFlag};
    use crate::domain::{DetectedStacks, File, Ignores, Result};
    use crate::git::FileStatus::{self, Changed, Removed};
    use crate::git::{GitCommandExt, Repo};
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
        let files = files_in_scope(CLIScope::Uncommitted, &repo)?;
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
        let files = files_in_scope(CLIScope::Branch, &repo)?;
        let want = vec![("on-branch.md", Changed), ("untracked.md", Changed)];
        pretty::assert_eq!(summary(&files), Some(want));
        Ok(())
    }

    #[test]
    fn every_scope_is_all_without_git_repo() -> Result<()> {
        for scope in [CLIScope::All, CLIScope::Uncommitted, CLIScope::Branch] {
            let flag = ScopeFlag { scope: Some(scope) };
            pretty::assert_eq!(determine_files(&flag, None, None)?, FileScope::All);
            let no_flag = ScopeFlag { scope: None };
            pretty::assert_eq!(
                determine_files(&no_flag, Some(scope), None)?,
                FileScope::All
            );
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
        let files = files_in_scope(CLIScope::Uncommitted, &repo)?;
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
        let files = files_in_scope(CLIScope::Uncommitted, &repo)?;
        let stacks = discover_stacks(&files, &Ignores::empty());
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
            determine_files(&no_flag, None, None).unwrap(),
            FileScope::All
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
        let have = determine_files(&no_flag, None, Some(&repo)).unwrap();
        pretty::assert_eq!(summary(&have), Some(vec![("untracked.md", Changed)]));
    }

    #[test]
    fn infer_branch_when_clean_tree_has_branch_changes() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        repo.create_and_commit_file("on-branch.md").unwrap();
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo)).unwrap();
        pretty::assert_eq!(summary(&have), Some(vec![("on-branch.md", Changed)]));
    }

    #[test]
    fn infer_uncommitted_when_only_uncommitted_deletions_exist() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        std::fs::remove_file(repo.file_path("on-main.md")).unwrap();
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo)).unwrap();
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
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo)).unwrap();
        pretty::assert_eq!(summary(&have), Some(vec![("on-main.md", Removed)]));
    }

    #[test]
    fn infer_all_when_clean_tree_on_main() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo)).unwrap();
        pretty::assert_eq!(have, FileScope::All);
    }

    #[test]
    fn infer_all_when_clean_tree_on_empty_feature_branch() {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path()).unwrap();
        repo.create_and_commit_file("on-main.md").unwrap();
        repo.create_and_switch_to_branch("feature").unwrap();
        let no_flag = ScopeFlag { scope: None };
        let have = determine_files(&no_flag, None, Some(&repo)).unwrap();
        pretty::assert_eq!(have, FileScope::All);
    }

    #[test]
    fn resolve_uses_explicit_scope() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let explicit = ScopeFlag {
            scope: Some(CLIScope::All),
        };
        let have = determine_files(&explicit, None, Some(&repo))?;
        pretty::assert_eq!(have, FileScope::All);
        Ok(())
    }

    #[test]
    fn resolve_infers_when_scope_omitted() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let omitted = ScopeFlag { scope: None };
        let have = determine_files(&omitted, None, Some(&repo))?;
        pretty::assert_eq!(summary(&have), Some(vec![("untracked.md", Changed)]));
        Ok(())
    }

    #[test]
    fn uses_default_scope_when_flag_omitted() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_file("untracked.md");
        let omitted = ScopeFlag { scope: None };
        let have = determine_files(&omitted, Some(CLIScope::All), Some(&repo))?;
        pretty::assert_eq!(have, FileScope::All);
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
        let have = determine_files(&explicit, Some(CLIScope::All), Some(&repo))?;
        pretty::assert_eq!(summary(&have), Some(vec![("untracked.md", Changed)]));
        Ok(())
    }

    #[test]
    fn git_failure_is_an_error() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        std::fs::remove_dir_all(dir.path().join(".git")).unwrap();
        for scope in [Some(CLIScope::Uncommitted), Some(CLIScope::Branch), None] {
            let flag = ScopeFlag { scope };
            assert!(determine_files(&flag, None, Some(&repo)).is_err());
        }
        Ok(())
    }
}
