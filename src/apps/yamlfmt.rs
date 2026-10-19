use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config, Operation};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Result, Tool};
use big_s::S;
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
        f.write_str("Yamlfmt")
    }
}

impl Fix for Yamlfmt {
    fn fix_commands(
        &self,
        stack: &DetectedStack,
        config: &Config,
    ) -> Result<Vec<conc::Executable>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("fix {} ({self})", stack.stack),
            app: &rta::applications::Yamlfmt {},
            args: files.into_strings(),
            version: None,
        })?;
        Ok(executable.into_iter().collect())
    }

    fn unsafe_fix_commands(
        &self,
        stack: &DetectedStack,
        config: &Config,
    ) -> Result<Vec<conc::Executable>> {
        let ignores = config.ignores_for(self, Operation::FixUnsafe)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let mut args = Vec::with_capacity(files.len() + 2);
        args.push(S("format"));
        args.push(S("--force"));
        args.extend(files.into_strings());
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("unsafe-fix {} ({self})", stack.stack),
            app: &rta::applications::Taplo {},
            args,
            version: None,
        })?;
        Ok(executable.into_iter().collect())
    }
}
