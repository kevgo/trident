use crate::config::{Application, ApplicationSection};
use crate::domain::{EnabledWhen, Tool};
use crate::git;
use big_s::S;
use std::fmt::Display;

pub struct GitDiffCheck;

impl Tool for GitDiffCheck {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.git_diff_check.as_ref()?)
    }
}

impl Display for GitDiffCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("git diff check")
    }
}

// TODO: make this a proper "Git" stack, which defines a linter called "lint Git (diff markers)"

/// provides the command that checks the Git changes for whitespace errors,
/// if the current directory is a Git repository
#[must_use]
pub(crate) fn lint_command(repo: &git::Repo) -> conc::Executable {
    let mut command = repo.git_command();
    command.args(["diff", "HEAD", "--check"]);
    conc::Executable {
        name: S("lint Git diff markers (git diff HEAD --check)"),
        command,
    }
}
