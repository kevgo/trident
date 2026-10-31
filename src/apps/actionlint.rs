use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, Operation, Result, StackType, Tool};
use crate::subshell::ToolCall;
use std::fmt::Display;

pub struct Actionlint;

impl Tool for Actionlint {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::FolderContainingFileOfType {
            folder: ".github/workflows",
            file_type: StackType::Yml,
        }
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.actionlint.as_ref()?)
    }
}

impl Display for Actionlint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Actionlint")
    }
}

impl Lint for Actionlint {
    fn lint_commands(&self, stack: &DetectedStack, _config: &Config) -> Result<Option<ToolCall>> {
        let command = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::ActionLint {},
            args: vec![],
        })?;
        Ok(command.map(|command| ToolCall {
            operation: Operation::Lint,
            stack: Some(stack.stack.stack_type()),
            name: self.to_string(),
            command,
        }))
    }
}
