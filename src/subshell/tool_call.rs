use crate::domain::{Operation, StackType};
use std::process::Command;

/// something that can run
pub struct ToolCall {
    operation: Operation,

    // global tools don't belong to a stack
    stack: Option<StackType>,

    name: String,

    /// the command to execute
    command: Command,
}

impl Into<conc::Executable> for ToolCall {
    fn into(self) -> conc::Executable {
        let ToolCall {
            operation,
            stack,
            name,
            command,
        } = self;
        let conc_name = match stack {
            Some(stack) => format!("{operation} {stack} ({name})"),
            None => format!("{operation} all ({name})"),
        };
        conc::Executable {
            name: conc_name,
            command,
        }
    }
}
