use crate::apps::{GetRTACmdArgs2, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Lint, Operation, Result, Tool};
use crate::subshell::ToolCall;
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
    fn lint_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Option<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Lint)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(None);
        }
        let mut args = Vec::with_capacity(files.len() + 1);
        args.push(S("lint"));
        args.extend(files.into_strings());
        let biome = rta::applications::Biome {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 { app: &biome, args })? else {
            return Ok(None);
        };
        Ok(Some(ToolCall {
            operation: Operation::Lint,
            stack: Some(stack.stack.stack_type()),
            name: self.to_string(),
            command,
        }))
    }
}

impl Fix for Biome {
    fn fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 2);
        args.push(S("format"));
        args.push(S("--write"));
        args.extend(files.into_strings());
        let biome = rta::applications::Biome {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 { app: &biome, args })? else {
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
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 3);
        args.push(S("lint"));
        args.push(S("--write"));
        args.push(S("--unsafe"));
        args.extend(files.into_strings());
        let biome = rta::applications::Biome {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 { app: &biome, args })? else {
            return Ok(vec![]);
        };
        Ok(vec![ToolCall {
            operation: Operation::FixUnsafe,
            stack: Some(stack.stack.stack_type()),
            name: self.to_string(),
            command,
        }])
    }
}
