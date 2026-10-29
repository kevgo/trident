use crate::domain::{Operation, StackType};
use std::process::Command;

/// Definition for a linter
pub struct LintDefinition {
    /// name of the linter app
    pub name: &'static str,

    /// the runnable to execute
    pub command: Command,

    pub stack: StackType,
}

impl LintDefinition {
    pub fn to_conc_executable(self) -> conc::Executable {
        let LintDefinition {
            name,
            command,
            stack,
        } = self;
        conc::Executable {
            name: format!("lint {} ({})", stack, name),
            command,
        }
    }
}

/// A Trident-specific definition for calling a tool.
///
/// Tests are defined by name ("unit", "cuke"),
/// but the resulting tool definition will have a name like "test (unit)" or "test (cuke)".
#[derive(Clone, Debug, PartialEq)]
pub struct ToolDefinition {
    pub operation: Operation,

    /// display name
    pub name: Option<String>,

    /// the runnable to execute
    pub executable: Runnable,
}

impl ToolDefinition {
    /// provides the name of the tool, or the command if the tool has no name
    fn name_or_command(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.command)
    }

    /// Converts this tool into an executable whose printed name matches built-in tools:
    /// `{operation} {stack} ({name})`.
    #[must_use]
    pub fn to_executable(&self, operation: Operation, stack: StackType) -> conc::Executable {
        let tool_name = self.name.as_deref().unwrap_or(&self.command);
        conc::Executable {
            name: format!("{operation} {stack} ({tool_name})"),
            command: conc::shell_command(&self.command),
        }
    }

    #[must_use]
    pub fn to_sequence(self, operation: Operation, stack: StackType) -> conc::Sequence {
        conc::Sequence::one(self.to_executable(operation, stack))
    }
}
