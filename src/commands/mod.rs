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
use crate::domain::{DetectedStacks, FileChanges, Ignores, Result};
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

// How does an architecture that reflects reality look like?
//
// There are dedicatde types for each distinct piece of information:
//
// - FilesInWorkspace: contains all files in the workspace
// - FileChanges: files that are changed in the Git repository (path + change type)
// - FilesToLint: list of file paths that will be linted by a specific command. Derived from FileChanges and/or FilesInWorkspace depending on scope. Stacks with removed files process as all files.
// - FilesToFix: list of file paths that will be fixed by a specific command. Derived from FileChanges and/or FilesInWorkspace depending on scope. Only changed files, without the removed ones.
//
// Each command (lint, fix, pitstop) requests just the information types it needs.
//
// - commands get a list of file paths to process (FilesToProcess type), not a list of FileChanges (file path + change type)
// - if scope == all files: load FilesInWorkspace type, convert to FilesToProcess (remove ignores from config)
// - if explicit scope given:
//   - load FileChanges from Git
//   - convert to FilesToProcess. This is different per command:
//     - lint: if a stack has removed files --> process all files in that stack
//     - fix: process only the changed files in that stack (remove deleted files)
// - if scope inferred:
//   - load FileChanges from Git
//   - derive the list of files to process from it
//
// When printing metadata to STDERR, don't print the counts of all files. Print the count of files that will get processed.

/// arguments for [`scoped_stacks`]
pub(crate) struct DiscoverStacksArgs<'a> {
    /// value of the CLI `--scope` flag
    pub(crate) scope_flag: ScopeFlag,

    /// the scope to use when `--scope` is omitted, if none is given, use the smallest change: uncommitted files, or branch, or all files
    pub(crate) default_scope: Option<CLIScope>,

    /// the Git repository in the current directory
    pub(crate) repo: Option<&'a Repo>,

    /// the files to ignore
    pub(crate) ignores: &'a Ignores,
}

/// provides the stacks of all files that the scope changes, including the ones it deletes or renames away
///
/// This is what the user sees as the files in scope.
/// Fixes process the files in it that still exist in the workspace.
/// Lints process [`DetectedStacks::with_stacks_of_removed_files`] of it.
pub(crate) fn discover_stacks(
    DiscoverStacksArgs {
        scope_flag,
        default_scope,
        repo,
        ignores,
    }: DiscoverStacksArgs,
) -> DetectedStacks {
    let files = determine_files(&scope_flag, default_scope, repo, ignores);
    stacks::from_changes(&files, ignores)
}

/// Determines the files that are changed according to the given scope.
/// Falls back to all files in the workspace if Git doesn't work.
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

/// provides all changed files according the given scope
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
    // try uncommitted changes
    let uncommitted = repo.uncommitted_changes()?;
    if !uncommitted.is_empty() {
        return Ok(uncommitted);
    }
    // here there are no uncommitted changes --> try the branch changes
    let branch = repo.branch_committed_changes()?;
    if !branch.is_empty() {
        return Ok(branch);
    }
    // here the branch is empty --> check all files
    Ok(all_files(ignores))
}

/// provides all files in the workspace
fn all_files(ignores: &Ignores) -> FileChanges {
    FileChanges::from_existing(stacks::workspace_files(ignores))
}

#[cfg(test)]
mod tests {
    use super::{all_files, determine_files, files_in_scope, infer_files};
    use crate::cli::input::{CLIScope, ScopeFlag};
    use crate::domain::FileStatus::{Changed, Removed};
    use crate::domain::{DetectedStacks, File, FileChanges, Ignores, Result};
    use crate::git::{GitCommandExt, Repo};
    use crate::stacks;
    use tempfile::TempDir;

    /// all files in the given stacks, including the removed ones
    fn filenames(stacks: &DetectedStacks) -> Vec<File> {
        let mut files: Vec<File> = stacks
            .into_iter()
            .flat_map(|stack| stack.files.all().cloned())
            .collect();
        files.sort();
        files
    }

    /// the files in the given stacks that exist in the workspace
    fn changed_filenames(stacks: &DetectedStacks) -> Vec<File> {
        let mut files: Vec<File> = stacks
            .into_iter()
            .flat_map(|stack| stack.files.changed().cloned())
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
        let stacks = stacks::from_changes(&files, &Ignores::empty());
        pretty::assert_eq!(changed_filenames(&stacks), vec![File::from("untracked.md")]);
        pretty::assert_eq!(
            filenames(&stacks),
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
        let stacks = stacks::from_changes(&files, &Ignores::empty());
        let want = vec![File::from("untracked.md")];
        pretty::assert_eq!(filenames(&stacks), want);
        pretty::assert_eq!(
            filenames(&stacks.with_stacks_of_removed_files(&Ignores::empty())),
            want
        );
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
