use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Lint, Operation, Result, StackType, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use std::fmt::Display;

pub struct Tikibase;

impl Tool for Tikibase {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::FilePresent {
            filename: "tikibase.json",
            stack_type: StackType::Json,
        }
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.tikibase.as_ref()?)
    }
}

impl Display for Tikibase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("tikibase")
    }
}

impl Lint for Tikibase {
    fn lint_commands(&self, stack: &DetectedStack, _config: &Config) -> Result<Option<ToolCall>> {
        let mut args = Vec::with_capacity(stack.files.len() + 1);
        args.push(S("check"));
        let command = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::Tikibase {},
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

impl Fix for Tikibase {
    fn fix_commands(&self, stack: &DetectedStack, _config: &Config) -> Result<Vec<ToolCall>> {
        let mut args = Vec::with_capacity(stack.files.len() + 1);
        args.push(S("fix"));
        let command = get_rta_command(&GetRTACmdArgs {
            app: &rta::applications::Tikibase {},
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
