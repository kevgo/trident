use crate::apps::{GetRTACmd2Args, GetRTACmdArgs, get_rta_command, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{
    DetectedStack, EnabledWhen, Fix, Lint, LintDefinition, Result, StackType, Tool,
};
use big_s::S;
use rta::applications::AppDefinition;
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
    fn lint_commands(
        &self,
        stack: &DetectedStack,
        _config: &Config,
    ) -> Result<Option<LintDefinition>> {
        let mut args = Vec::with_capacity(stack.files.len() + 1);
        args.push(S("check"));
        let tikibase = rta::applications::Tikibase {};
        let command = get_rta_command2(&GetRTACmd2Args {
            app: &tikibase,
            args,
        })?;
        Ok(command.map(|command| LintDefinition {
            name: tikibase.name().as_str(),
            command,
            stack: stack.stack.stack_type(),
        }))
    }
}

impl Fix for Tikibase {
    fn fix_commands(
        &self,
        stack: &DetectedStack,
        _config: &Config,
    ) -> Result<Vec<conc::Executable>> {
        let mut args = Vec::with_capacity(stack.files.len() + 1);
        args.push(S("fix"));
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("fix {} ({self})", stack.stack),
            app: &rta::applications::Tikibase {},
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
