use crate::domain::{Operation, StackType};
use std::fmt::Debug;
use std::process::Command;

pub struct ToolCall {
    pub operation: Operation,
    pub stack: Option<StackType>,
    pub name: String,
    pub command: Command,
}

impl Debug for ToolCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ToolCall {
            operation,
            stack,
            name,
            command,
        } = self;
        f.debug_struct("ToolCall")
            .field("operation", &operation)
            .field("stack", &stack)
            .field("name", &name)
            .field("command", &command)
            .finish()
    }
}

impl From<ToolCall> for conc::Executable {
    fn from(value: ToolCall) -> Self {
        let ToolCall {
            operation,
            stack,
            name,
            command,
        } = value;
        conc::Executable {
            name: conc_name(operation, stack, &name),
            command,
        }
    }
}

fn conc_name(operation: Operation, stack: Option<StackType>, name: &str) -> String {
    match stack {
        Some(stack) => format!("{operation} {stack} ({name})"),
        None => format!("{operation} all ({name})"),
    }
}

#[cfg(test)]
mod tests {

    mod conc_name {
        use super::super::conc_name;
        use crate::domain::{Operation, StackType};

        #[test]
        fn with_stack() {
            let have = conc_name(Operation::Lint, Some(StackType::Rust), "clippy");
            let want = "lint Rust (clippy)";
            assert_eq!(have, want);
        }

        #[test]
        fn no_stack() {
            let have = conc_name(Operation::Fix, None, "git diff markers");
            let want = "fix all (git diff markers)";
            assert_eq!(have, want);
        }

        #[test]
        fn unsafe_fix() {
            let have = conc_name(Operation::FixUnsafe, Some(StackType::Css), "Biome");
            let want = "unsafe-fix CSS (Biome)";
            assert_eq!(have, want);
        }
    }

    mod debug {
        use crate::domain::{Operation, StackType};
        use crate::subshell::ToolCall;
        use big_s::S;
        use std::process::Command;

        #[test]
        fn with_stack() {
            let mut command = Command::new("echo");
            command.arg("one");
            command.arg("two");
            let give = ToolCall {
                operation: Operation::Fix,
                stack: Some(StackType::Cucumber),
                name: S("my name"),
                command,
            };
            let have = format!("{give:?}");
            let want = S(
                r#"ToolCall { operation: Fix, stack: Some(Cucumber), name: "my name", command: "echo" "one" "two" }"#,
            );
            assert_eq!(have, want);
        }

        #[test]
        fn no_stack() {
            let mut command = Command::new("echo");
            command.arg("one");
            command.arg("two");
            let give = ToolCall {
                operation: Operation::Fix,
                stack: None,
                name: S("my name"),
                command,
            };
            let have = format!("{give:?}");
            let want = S(
                r#"ToolCall { operation: Fix, stack: None, name: "my name", command: "echo" "one" "two" }"#,
            );
            assert_eq!(have, want);
        }
    }
}
