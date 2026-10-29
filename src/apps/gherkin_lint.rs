use crate::apps::{GetRTACmd2Args, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, LintDefinition, Operation, Result, Tool};
use rta::applications::AppDefinition;
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
        let gherkin_lint = rta::applications::GherkinLint {};
        let command = get_rta_command2(&GetRTACmd2Args {
            app: &gherkin_lint,
            args: files.into_strings(),
        })?;
        Ok(command.map(|command| LintDefinition {
            name: gherkin_lint.name().as_str(),
            command,
            stack: stack.stack.stack_type(),
        }))
    }
}
