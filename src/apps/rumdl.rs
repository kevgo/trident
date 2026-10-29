use crate::apps::{GetRTACmd2Args, GetRTACmdArgs, get_rta_command, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{
    DetectedStack, EnabledWhen, Fix, Lint, LintDefinition, Operation, Result, Tool,
};
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
        let rumdl = rta::applications::Rumdl {};
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("check"));
        args.extend(files.into_strings());
        let command = get_rta_command2(&GetRTACmd2Args { app: &rumdl, args })?;
        Ok(command.map(|command| LintDefinition {
            name: rumdl.name().as_str(),
            command,
            stack: stack.stack.stack_type(),
        }))
    }
}

impl Fix for Rumdl {
    fn fix_commands(
        &self,
        stack: &DetectedStack,
        config: &Config,
    ) -> Result<Vec<conc::Executable>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("fmt"));
        args.extend(files.into_strings());
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("fix {} ({self})", stack.stack),
            app: &rta::applications::Rumdl {},
            args,
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
