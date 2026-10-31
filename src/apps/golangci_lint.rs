use crate::apps::{GetRTACmdArgs2, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, Operation, Result, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use rta::applications::AppDefinition;
use std::fmt::Display;

pub struct GolangciLint;

impl Tool for GolangciLint {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
        // if let Some(yml_stack) = detected_stacks.with_type(StackType::Yml)
        //     && yml_stack
        //         .files
        //         .contains_any(&[".golangci.yml", ".golangci.yaml"])
        // {
        //     return true;
        // }
        // if let Some(toml_stack) = detected_stacks.with_type(StackType::Toml)
        //     && toml_stack.files.contains(".golangci.toml")
        // {
        //     return true;
        // }
        // if let Some(json_stack) = detected_stacks.with_type(StackType::Json)
        //     && json_stack.files.contains(".golangci.json")
        // {
        //     return true;
        // }
        // false
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.golangci_lint.as_ref()?)
    }
}

impl Display for GolangciLint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("golangci-lint")
    }
}

impl Lint for GolangciLint {
    fn lint_commands(&self, stack: &DetectedStack, _config: &Config) -> Result<Option<ToolCall>> {
        let golangci_lint = rta::applications::GolangCiLint {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 {
            app: &golangci_lint,
            args: vec![S("run"), S("--allow-parallel-runners")],
        })?
        else {
            return Ok(None);
        };
        Ok(Some(ToolCall {
            operation: Operation::Lint,
            stack: Some(stack.stack.stack_type()),
            name: golangci_lint.name().to_string(),
            command,
        }))
    }
}
