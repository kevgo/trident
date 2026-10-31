use crate::apps::{GetRTACmdArgs2, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Operation, Result, Tool};
use crate::subshell::ToolCall;
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
    fn fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 {
            app: &rta::applications::Yamlfmt {},
            args: files.into_strings(),
        })?
        else {
            return Ok(vec![]);
        };
        Ok(vec![ToolCall {
            operation: Operation::Fix,
            stack: Some(stack.stack.stack_type()),
            name: self.to_string(),
            command,
        }])
    }

    fn unsafe_fix_commands(
        &self,
        _stack: &DetectedStack,
        _config: &Config,
    ) -> Result<Vec<ToolCall>> {
        Ok(vec![])
    }
}
