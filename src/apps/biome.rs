use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::config::{Application, ApplicationSection, Config, Operation};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Lint, Result, Tool};
use big_s::S;
use std::fmt::Display;

pub struct Biome;

impl Tool for Biome {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
        // detected_stacks.has_file(StackType::Json, "biome.json")
        //     || detected_stacks.has_file(StackType::Unknown, "biome.jsonc")
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.biome.as_ref()?)
    }
}

impl Display for Biome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Biome")
    }
}

impl Lint for Biome {
    fn lint_commands(
        &self,
        stack: &DetectedStack,
        config: &Config,
    ) -> Result<&'static dyn rta::applications::AppDefinition> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("lint"));
        args.extend(files.into_strings());
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("lint {} ({self})", stack.stack),
            app: &rta::applications::Biome {},
            args,
            version: None,
        })?;
        Ok(executable)
    }
}

impl Fix for Biome {
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
        let mut args = Vec::with_capacity(files.len() + 2);
        args.push(S("format"));
        args.push(S("--write"));
        args.extend(files.into_strings());
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("fix {} ({self})", stack.stack),
            app: &rta::applications::Biome {},
            args,
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
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 3);
        args.push(S("lint"));
        args.push(S("--write"));
        args.push(S("--unsafe"));
        args.extend(files.into_strings());
        let executable = get_rta_command(&GetRTACmdArgs {
            name: format!("unsafe-fix {} ({self})", stack.stack),
            app: &rta::applications::Biome {},
            args,
            version: None,
        })?;
        Ok(executable.into_iter().collect())
    }
}
