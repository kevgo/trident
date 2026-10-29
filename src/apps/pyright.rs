use rta::applications::AppDefinition;

use crate::apps::{GetRTACmd2Args, GetRTACmdArgs, get_rta_command, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{
    DetectedStack, EnabledWhen, Lint, LintDefinition, Operation, Result, StackType, Tool,
};
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
        let pyright = rta::applications::Pyright {};
        let command = get_rta_command2(&GetRTACmd2Args {
            app: &pyright,
            args: files.into_strings(),
        })?;
        Ok(command.map(|command| LintDefinition {
            name: pyright.name().as_str(),
            command,
        }))
    }
}
