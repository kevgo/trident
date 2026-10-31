use crate::apps::{GetRTACmdArgs, GetRTACmdArgs2, get_rta_command, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Operation, Result, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use rta::applications::AppDefinition;
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
        let gofumpt = rta::applications::Gofumpt {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 {
            app: &gofumpt,
            args,
        })?
        else {
            return Ok(vec![]);
        };
        Ok(vec![ToolCall {
            operation: Operation::Fix,
            stack: Some(stack.stack.stack_type()),
            name: gofumpt.name().to_string(),
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
