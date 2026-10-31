use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::domain::{EnabledWhen, Operation, Result, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use std::fmt::Display;

pub struct DeleteEmptyFolders;

impl Tool for DeleteEmptyFolders {
    fn enabled_when(&self) -> crate::domain::EnabledWhen {
        EnabledWhen::Always
    }

    fn config_section<'a>(
        &self,
        apps: &'a crate::config::ApplicationSection,
    ) -> Option<&'a dyn crate::config::Application> {
        Some(apps.delete_empty_folders.as_ref()?)
    }
}

impl Display for DeleteEmptyFolders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("delete empty folders")
    }
}

pub fn format_command() -> Result<Option<ToolCall>> {
    let command = get_rta_command(&GetRTACmdArgs {
        app: &rta::applications::DeleteEmptyFolders {},
        args: vec![],
    })?;
    Ok(command.map(|command| ToolCall {
        operation: Operation::Fix,
        stack: None,
        name: S("delete empty folders"),
        command,
    }))
}
