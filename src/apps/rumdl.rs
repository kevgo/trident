use crate::apps::{GetRTACmdArgs, GetRTACmdArgs2, get_rta_command, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Lint, Operation, Result, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use rta::applications::AppDefinition;
use std::fmt::Display;

pub struct Rumdl;

impl Tool for Rumdl {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
        // let Some(toml_stack) = detected_stacks.with_type(StackType::Toml) else {
        //     return false;
        // };
        // toml_stack
        //     .files
        //     .contains_any(&["rumdl.toml", ".rumdl.toml", ".config/rumdl.toml"])
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.rumdl.as_ref()?)
    }
}

impl Display for Rumdl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("rumdl")
    }
}

impl Lint for Rumdl {
    fn lint_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Option<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("check"));
        args.extend(files.into_strings());
        let rumdl = rta::applications::Rumdl {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 {
            app: &rta::applications::Rumdl {},
            args,
        })?
        else {
            return Ok(None);
        };
        Ok(Some(ToolCall {
            operation: Operation::Lint,
            stack: Some(stack.stack.stack_type()),
            name: rumdl.name().to_string(),
            command,
        }))
    }
}

impl Fix for Rumdl {
    fn fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("fmt"));
        args.extend(files.into_strings());
        let rumdl = rta::applications::Rumdl {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 { app: &rumdl, args })? else {
            return Ok(vec![]);
        };
        Ok(vec![ToolCall {
            operation: Operation::Fix,
            stack: Some(stack.stack.stack_type()),
            name: rumdl.name().to_string(),
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
