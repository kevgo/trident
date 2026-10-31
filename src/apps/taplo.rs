use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Lint, Operation, Result, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use std::fmt::Display;

pub struct Taplo;

impl Tool for Taplo {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.taplo.as_ref()?)
    }
}

impl Display for Taplo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Taplo")
    }
}

impl Lint for Taplo {
    fn lint_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Option<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("lint"));
        args.extend(files.into_strings());
        let command = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::Taplo {},
            args,
        })?;
        Ok(command.map(|command| ToolCall {
            operation: Operation::Lint,
            stack: Some(stack.stack.stack_type()),
            name: self.to_string(),
            command,
        }))
    }
}

impl Fix for Taplo {
    fn fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("format"));
        args.extend(files.into_strings());
        let Some(command) = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::Taplo {},
            args,
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

    fn unsafe_fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::FixUnsafe)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let mut args = Vec::with_capacity(files.len() + 2);
        args.push(S("format"));
        args.push(S("--force"));
        args.extend(files.into_strings());
        let command = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::Taplo {},
            args,
        })?;
        Ok(command
            .into_iter()
            .map(|command| ToolCall {
                operation: Operation::FixUnsafe,
                stack: Some(stack.stack.stack_type()),
                name: self.to_string(),
                command,
            })
            .collect())
    }
}
