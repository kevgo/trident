use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Operation, Result, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use std::fmt::Display;

pub struct Gofumpt;

impl Tool for Gofumpt {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.gofumpt.as_ref()?)
    }
}

impl Display for Gofumpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("gofumpt")
    }
}

impl Fix for Gofumpt {
    fn fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 2);
        args.push(S("-l"));
        args.push(S("-w"));
        args.extend(files.into_strings());
        let command = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::Gofumpt {},
            args,
        })?;
        Ok(command
            .into_iter()
            .map(|command| ToolCall {
                operation: Operation::Fix,
                stack: Some(stack.stack.stack_type()),
                name: self.to_string(),
                command,
            })
            .collect())
    }

    fn unsafe_fix_commands(
        &self,
        _stack: &DetectedStack,
        _config: &Config,
    ) -> Result<Vec<ToolCall>> {
        Ok(vec![])
    }
}
