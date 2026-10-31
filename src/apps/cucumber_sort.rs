use crate::apps::{GetRTACmdArgs2, get_rta_command2};
use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Fix, Operation, Result, StackType, Tool};
use crate::subshell::ToolCall;
use big_s::S;
use rta::applications::AppDefinition;
use std::fmt::Display;

pub struct CucumberSort;

impl Tool for CucumberSort {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::FilePresent {
            filename: "cucumber-sort.json",
            stack_type: StackType::Json,
        }
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.cucumber_sort.as_ref()?)
    }
}

impl Display for CucumberSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("cucumber-sort")
    }
}

impl Fix for CucumberSort {
    fn fix_commands(&self, stack: &DetectedStack, _config: &Config) -> Result<Vec<ToolCall>> {
        let cucumber_sort = rta::applications::CucumberSort {};
        let Some(command) = get_rta_command2(&GetRTACmdArgs2 {
            app: &cucumber_sort,
            args: vec![S("format")],
        })?
        else {
            return Ok(vec![]);
        };
        Ok(vec![ToolCall {
            operation: Operation::Fix,
            stack: Some(stack.stack.stack_type()),
            name: cucumber_sort.name().to_string(),
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
