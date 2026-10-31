use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, Operation, Result, Tool};
use crate::subshell::ToolCall;
use std::fmt::Display;

pub struct ShellCheck;

impl Tool for ShellCheck {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.shellcheck.as_ref()?)
    }
}

impl Display for ShellCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShellCheck")
    }
}

impl Lint for ShellCheck {
    fn lint_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Option<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let Some(command) = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::ShellCheck {},
            args: files.into_strings(),
        })?
        else {
            return Ok(None);
        };
        Ok(Some(ToolCall {
            operation: Operation::Lint,
            stack: Some(stack.stack.stack_type()),
            name: self.to_string(),
            command,
        }))
    }
}
