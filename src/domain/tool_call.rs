use std::process::Command;

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

    /// the runnable to execute
    pub runnable: Executable,
}

impl ToolCall {
    pub fn to_executable(&self) -> conc::Executable {
        let run_method = self.app.run_method(self.version, self.platform);
        conc::Executable {
            name: format!("{} ({})", self.operation, self.name_or_command()),
            command: conc::shell_command(&self.command),
        }
    }

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

/// anything that Trident can execute
///
/// Either an RTA app (a built-in tool)
/// or a shell script (a user-provided script)
#[derive(Clone, Debug, PartialEq)]
pub enum Executable {
    RTAApp {
        /// the RTA app to execute
        app: &'static dyn rta::applications::AppDefinition,

        /// the version of the RTA app to execute
        version: rta::Version,

        /// arguments to the command
        args: Vec<String>,
    },

    /// a user-provided shell script
    ShellScript(String),
}

impl Executable {
    pub fn to_command(&self) -> Command {
        match self {
            Executable::RTAApp { app, version, args } => {
                let platform = rta::platform::detect(log);
                let run_method = app.run_method(version, platform);

                conc::Executable {
                    name: format!("{} ({})", operation, name_or_command()),
                    command: conc::shell_command(&command),
                }
            }
        }
    }
}
