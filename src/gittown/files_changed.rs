use crate::domain::File;
use crate::git::GitCommandExt;
use crate::git::Repo;

/// provides the files that the current branch has changed compared to its parent branch
///
/// Does not include uncommitted files.
/// Includes deleted files and both the old and new names of renamed files.
pub(crate) fn files_changed_on_current_branch(repo: &Repo) -> Option<Vec<File>> {
    let Ok(output) = repo
        .git_command()
        .args([
            "-c",
            "diff.renames=false",
            "town",
            "diff-parent",
            "--name-only",
            "--non-interactive",
        ])
        .run_stdout_trimmed()
    else {
        return None;
    };
    Some(parse_output(&output))
}

/// provides the files that the current branch deletes or renames away compared to its parent branch
///
/// does not include uncommitted files
pub(crate) fn files_removed_on_current_branch(repo: &Repo) -> Option<Vec<File>> {
    let Ok(output) = repo
        .git_command()
        // Without rename detection, Git reports the old name of a renamed file as deleted.
        // Git Town's Git calls inherit this setting.
        .args(["-c", "diff.renames=false"])
        .args([
            "town",
            "diff-parent",
            "--name-only",
            "--diff-filter=D",
            "--non-interactive",
        ])
        .run_stdout_trimmed()
    else {
        return None;
    };
    Some(parse_output(&output))
}

fn parse_output(output: &str) -> Vec<File> {
    output
        .lines()
        .filter(|line| !line.is_empty() && !is_git_town_command_echo(line))
        .map(File::from)
        .collect()
}

fn is_git_town_command_echo(line: &str) -> bool {
    line.contains("] git ")
}

#[cfg(test)]
mod tests {
    mod parse_output {
        use super::super::parse_output;
        use crate::domain::File;

        #[test]
        fn empty() {
            pretty::assert_eq!(parse_output(""), Vec::<File>::new());
        }

        #[test]
        fn skips_blank_lines() {
            let give = "\na.txt\n\nb.txt\n";
            let have = parse_output(give);
            let want = vec![File::from("a.txt"), File::from("b.txt")];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn skips_git_town_command_echo() {
            let give = "[feature] git diff --name-only --merge-base main feature\ncommitted.txt";
            let have = parse_output(give);
            let want = vec![File::from("committed.txt")];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn skips_ansi_styled_command_echo() {
            let give = "\u{1b}[1m[feature] git diff --name-only --merge-base main feature\u{1b}[0m\ncommitted.txt";
            let have = parse_output(give);
            let want = vec![File::from("committed.txt")];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn keeps_file_with_spaces() {
            let give = "my file.txt";
            let have = parse_output(give);
            let want = vec![File::from("my file.txt")];
            pretty::assert_eq!(have, want);
        }
    }

    mod files_removed_on_current_branch {
        use super::super::files_removed_on_current_branch;
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::GitCommandExt;
        use crate::git::Repo;
        use std::process::Stdio;
        use tempfile::TempDir;

        #[test]
        fn none_when_unconfigured() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            pretty::assert_eq!(files_removed_on_current_branch(&repo), None);
            Ok(())
        }

        #[test]
        fn lists_deleted_and_renamed_files() -> Result<()> {
            if which::which("git-town").is_err() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            repo.git_command()
                .args(["config", "git-town.main-branch", "main"])
                .run()?;
            repo.create_and_commit_file("deleted.txt")?;
            repo.create_and_commit_file("old.txt")?;
            repo.create_and_commit_file("changed.txt")?;
            repo.git_command()
                .args(["town", "append", "feature"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .run()?;
            repo.git_command().args(["rm", "deleted.txt"]).run()?;
            repo.git_command()
                .args(["mv", "old.txt", "new.txt"])
                .run()?;
            std::fs::write(repo.file_path("changed.txt"), "changed").unwrap();
            repo.git_command().args(["add", "changed.txt"]).run()?;
            repo.git_command()
                .args(["commit", "--quiet", "--message=Remove"])
                .run()?;
            let have = files_removed_on_current_branch(&repo);
            let want = Some(vec![File::from("deleted.txt"), File::from("old.txt")]);
            pretty::assert_eq!(have, want);
            Ok(())
        }
    }

    mod files_changed_on_current_branch {
        use super::super::files_changed_on_current_branch;
        use crate::domain::File;
        use crate::domain::Result;
        use crate::git::GitCommandExt;
        use crate::git::Repo;
        use std::path::Path;
        use std::process::Stdio;
        use tempfile::TempDir;

        fn gittown_repo(path: &Path) -> Result<Repo> {
            let repo = Repo::init(path)?;
            repo.git_command()
                .args(["config", "git-town.main-branch", "main"])
                .run()?;
            Ok(repo)
        }

        fn append_branch(repo: &Repo, name: &str) -> Result<()> {
            repo.git_command()
                .args(["town", "append", name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .run()
        }

        fn git_town_installed() -> bool {
            which::which("git-town").is_ok()
        }

        #[test]
        fn none_when_unconfigured() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = Repo::init(dir.path())?;
            let have = files_changed_on_current_branch(&repo);
            let want = None;
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn none_on_main_branch() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            let have = files_changed_on_current_branch(&repo);
            let want = None;
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn none_when_feature_has_no_parent() -> Result<()> {
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            repo.git_command()
                .args(["checkout", "--quiet", "-b", "feature"])
                .run()?;
            let have = files_changed_on_current_branch(&repo);
            let want = None;
            pretty::assert_eq!(have, want);
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
            let have = files_changed_on_current_branch(&repo);
            let want = Some(vec![File::from("committed.txt")]);
            pretty::assert_eq!(have, want);
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
            let have = files_changed_on_current_branch(&repo);
            let want = Some(Vec::<File>::new());
            pretty::assert_eq!(have, want);
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
            let have = files_changed_on_current_branch(&repo);
            let want = Some(vec![File::from("child.txt")]);
            pretty::assert_eq!(have, want);
            Ok(())
        }

        #[test]
        fn includes_file_with_spaces() -> Result<()> {
            if !git_town_installed() {
                return Ok(());
            }
            let dir = TempDir::new().unwrap();
            let repo = gittown_repo(dir.path())?;
            append_branch(&repo, "feature")?;
            repo.create_and_commit_file("my file.txt")?;
            let have = files_changed_on_current_branch(&repo);
            let want = Some(vec![File::from("my file.txt")]);
            pretty::assert_eq!(have, want);
            Ok(())
        }
    }
}
