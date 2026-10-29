use rta::applications::AppDefinition;

use crate::apps::get_rta_command2;
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, LintDefinition, Result, StackType, Tool};
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
    fn lint_commands(
        &self,
        _stack: &DetectedStack,
        _config: &Config,
    ) -> Result<Option<LintDefinition>> {
        let action_lint = rta::applications::ActionLint {};
        let command = get_rta_command2(&super::GetRTACmd2Args {
            app: &action_lint,
            args: vec![],
        })?;
        let Some(command) = command else {
            return Ok(None);
        };
        Ok(Some(LintDefinition {
            name: action_lint.name().as_str(),
            command,
        }))
    }
}
