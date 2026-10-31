use crate::apps::{GetRTACmdArgs, GetRTACmdArgs2, get_rta_command, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Lint, Operation, Result, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use rta::applications::AppDefinition;
use std::fmt::Display;

pub struct Ruff;

impl Tool for Ruff {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
        //     .contains_any(&["ruff.toml", "ruff.toml.json"])
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.ruff.as_ref()?)
    }
}

impl Display for Ruff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ruff")
    }
}

impl Lint for Ruff {
    fn lint_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Option<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("check"));
        args.extend(files.into_strings());
        let ruff = rta::applications::Ruff {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 { app: &ruff, args })? else {
            return Ok(None);
        };
        Ok(Some(ToolCall {
            operation: Operation::Lint,
            stack: Some(stack.stack.stack_type()),
            name: ruff.name().to_string(),
            command,
        }))
    }
}

impl Fix for Ruff {
    fn fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let mut commands = Vec::with_capacity(2);

        // NOTE: Ruff has separate commands for formatting and linting
        // until https://github.com/astral-sh/ruff/issues/8232 ships.

        // run "ruff format --check"
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 2);
        args.push(S("check"));
        args.push(S("--fix"));
        args.extend(files.clone().into_strings());
        let ruff = rta::applications::Ruff {};
        let command = get_rta_command2(&GetRTACmdArgs2 { app: &ruff, args })?;
        if let Some(command) = command {
            commands.push(ToolCall {
                operation: Operation::Fix,
                stack: Some(stack.stack.stack_type()),
                name: ruff.name().to_string(),
                command,
            });
        }

        // run "ruff format"
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("format"));
        args.extend(files.into_strings());
        let command = get_rta_command2(&GetRTACmdArgs2 { app: &ruff, args })?;
        if let Some(command) = command {
            commands.push(ToolCall {
                operation: Operation::Fix, // TODO: change to Operation::Format
                stack: Some(stack.stack.stack_type()),
                name: ruff.name().to_string(),
                command,
            });
        }

        Ok(commands)
    }

    fn unsafe_fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let mut executables = Vec::with_capacity(2);

        // run "ruff format --check"
        let ignores = config.ignores_for(self, Operation::FixUnsafe)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 3);
        args.push(S("check"));
        args.push(S("--fix"));
        args.push(S("--unsafe-fixes"));
        args.extend(files.clone().into_strings());
        let ruff = rta::applications::Ruff {};
        let command = get_rta_command2(&GetRTACmdArgs2 {
            app: &rta::applications::Ruff {},
            args,
        })?;
        if let Some(command) = command {
            executables.push(ToolCall {
                operation: Operation::FixUnsafe,
                stack: Some(stack.stack.stack_type()),
                name: ruff.name().to_string(),
                command,
            });
        }

        // run "ruff format"
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("format"));
        args.extend(files.into_strings());
        let command = get_rta_command2(&GetRTACmdArgs2 {
            app: &rta::applications::Ruff {},
            args,
        })?;
        if let Some(command) = command {
            executables.push(ToolCall {
                operation: Operation::Fix, // TODO: change to Operation::Format
                stack: Some(stack.stack.stack_type()),
                name: ruff.name().to_string(),
                command,
            });
        }

        Ok(executables)
    }
}
