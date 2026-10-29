use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{
    DetectedStack, EnabledWhen, Fix, Lint, LintDefinition, Operation, Result, Tool,
};
use big_s::S;
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
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("check"));
        args.extend(files.into_strings());
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("lint {} (ruff)", stack.stack),
            app: &rta::applications::Ruff {},
            args,
            version: None,
        })?;
        let Some(executable) = executable else {
            return Ok(None);
        };
        Ok(Some(executable))
    }
}

impl Fix for Ruff {
    fn fix_commands(
        &self,
        stack: &DetectedStack,
        config: &Config,
    ) -> Result<Vec<conc::Executable>> {
        let mut executables = Vec::with_capacity(2);

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
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("fix {} (ruff)", stack.stack),
            app: &rta::applications::Ruff {},
            args,
            version: None,
        })?;
        if let Some(executable) = executable {
            executables.push(executable);
        }

        // run "ruff format"
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("format"));
        args.extend(files.into_strings());
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("format {} (ruff)", stack.stack),
            app: &rta::applications::Ruff {},
            args,
            version: None,
        })?;
        if let Some(executable) = executable {
            executables.push(executable);
        }

        Ok(executables)
    }

    fn unsafe_fix_commands(
        &self,
        stack: &DetectedStack,
        config: &Config,
    ) -> Result<Vec<conc::Executable>> {
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
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("unsafe-fix {} (ruff)", stack.stack),
            app: &rta::applications::Ruff {},
            args,
            version: None,
        })?;
        if let Some(executable) = executable {
            executables.push(executable);
        }

        // run "ruff format"
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("format"));
        args.extend(files.into_strings());
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("format {} (ruff)", stack.stack),
            app: &rta::applications::Ruff {},
            args,
            version: None,
        })?;
        if let Some(executable) = executable {
            executables.push(executable);
        }

        Ok(executables)
    }
}
