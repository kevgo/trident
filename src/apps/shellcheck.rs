use crate::apps::{GetRTACmd2Args, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, LintDefinition, Operation, Result, Tool};
use rta::applications::AppDefinition;
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
    fn lint_commands(
        &self,
        stack: &DetectedStack,
        config: &Config,
    ) -> Result<Option<LintDefinition>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let shellcheck = rta::applications::ShellCheck {};
        let command = get_rta_command2(&GetRTACmd2Args {
            app: &shellcheck,
            args: files.into_strings(),
        })?;
        Ok(command.map(|command| LintDefinition {
            name: shellcheck.name().as_str(),
            command,
            stack: stack.stack.stack_type(),
        }))
    }
}
