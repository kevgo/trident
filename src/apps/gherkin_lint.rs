use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, Operation, Result, Tool};
use crate::subshell::ToolCall;
use std::fmt::Display;

pub struct GherkinLint;

impl Tool for GherkinLint {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
        // let Some(other_stack) = detected_stacks.with_type(StackType::Unknown) else {
        //     return false;
        // };
        // other_stack.files.contains(".gherkin-lintrc")
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.gherkin_lint.as_ref()?)
    }
}

impl Display for GherkinLint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("gherkin-lint")
    }
}

impl Lint for GherkinLint {
    fn lint_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Option<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let command = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::GherkinLint {},
            args: files.into_strings(),
        })?;
        Ok(command.map(|command| ToolCall {
            operation: Operation::Lint,
            stack: Some(stack.stack.stack_type()),
            name: self.to_string(),
            command,
        }))
    }
}
