use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config, Operation};
use crate::domain::{DetectedStack, EnabledWhen, Lint, Result, StackType, Tool};
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
    ) -> Result<Option<conc::Executable>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("type-check {} ({self})", stack.stack),
            app: &rta::applications::Pyright {},
            args: files.into_strings(),
            version: None,
        })?;
        let Some(executable) = executable else {
            return Ok(None);
        };
        Ok(Some(executable))
    }
}
