use crate::domain::{Operation, StackType};

/// A Trident-specific definition for calling a tool.
///
/// Tests are defined by name ("unit", "cuke"),
/// but the resulting tool definition will have a name like "test (unit)" or "test (cuke)".
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    pub operation: Operation,

    /// display name
    pub name: Option<String>,

    /// command to execute
    pub command: String,
}

impl ToolCall {
    #[must_use]
    pub fn to_sequence(self) -> conc::Sequence {
        conc::Sequence::one(conc::Executable {
            name: format!("{} ({})", self.operation, self.name_or_command()),
            command: conc::shell_command(&self.command),
        })
    }

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
}
