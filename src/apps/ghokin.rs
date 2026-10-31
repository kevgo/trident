use crate::apps::{GetRTACmdArgs, GetRTACmdArgs2, get_rta_command, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Operation, Result, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use rta::applications::AppDefinition;
use std::fmt::Display;

pub struct Ghokin;

impl Tool for Ghokin {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.ghokin.as_ref()?)
    }
}

impl Display for Ghokin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ghokin")
    }
}

impl Fix for Ghokin {
    fn fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>> {
        let ignores = config.ignores_for(self, Operation::Fix)?;
        let files = &stack.files.remove(&ignores);
        if files.is_empty() {
            return Ok(vec![]);
        }
        let mut args = Vec::with_capacity(files.len() + 2);
        args.push(S("fmt"));
        args.push(S("replace"));
        for file in files {
            let filename = if file.starts_with("./") {
                file[2..].to_string()
            } else {
                file.to_string()
            };
            args.push(filename);
        }
        let ghokin = rta::applications::Ghokin {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 {
            app: &rta::applications::Ghokin {},
            args,
        })?
        else {
            return Ok(vec![]);
        };
        Ok(vec![ToolCall {
            operation: Operation::Fix,
            stack: Some(stack.stack.stack_type()),
            name: ghokin.name().to_string(),
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
