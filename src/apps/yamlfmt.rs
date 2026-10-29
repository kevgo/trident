use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Operation, Result, Tool};
use std::fmt::Display;

pub struct Yamlfmt;

impl Tool for Yamlfmt {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.yamlfmt.as_ref()?)
    }
}

impl Display for Yamlfmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("yamlfmt")
    }
}

impl Fix for Yamlfmt {
    fn fix_commands(
        &self,
        stack: &DetectedStack,
        config: &Config,
    ) -> Result<Vec<conc::Executable>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("fix {} ({self})", stack.stack),
            app: &rta::applications::Yamlfmt {},
            args: files.into_strings(),
            version: None,
        })?;
        Ok(executable.into_iter().collect())
    }

    fn unsafe_fix_commands(
        &self,
        _stack: &DetectedStack,
        _config: &Config,
    ) -> Result<Vec<conc::Executable>> {
        Ok(vec![])
    }
}
