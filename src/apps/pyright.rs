use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, Operation, Result, StackType, Tool};
use crate::subshell::ToolCall;
use std::fmt::Display;

pub struct Pyright;

impl Tool for Pyright {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::FilePresent {
            filename: "pyrightconfig.json",
            stack_type: StackType::Json,
        }
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.pyright.as_ref()?)
    }
}

impl Display for Pyright {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pyright")
    }
}

impl Lint for Pyright {
    fn lint_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Option<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let command = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::Pyright {},
            args: files.into_strings(),
        })?;
        Ok(command.map(|command| ToolCall {
            operation: Operation::TypeCheck,
            stack: Some(stack.stack.stack_type()),
            name: self.to_string(),
            command,
        }))
    }
}
