use crate::config::{Application, ApplicationSection};
use crate::domain::{EnabledWhen, Tool};
use crate::git;
use crate::subshell::ToolCall;
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

/// provides the command that checks the Git changes for whitespace errors,
/// if the current directory is a Git repository
#[must_use]
pub(crate) fn lint_command(repo: &git::Repo) -> ToolCall {
    let mut command = repo.git_command();
    command.args(["diff", "HEAD", "--check"]);
    ToolCall {
        operation: crate::domain::Operation::Lint,
        stack: None,
        name: S("lint Git diff markers (git diff HEAD --check)"),
        command,
    }
}
