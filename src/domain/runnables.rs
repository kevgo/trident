use crate::domain::StackType;
use crate::subshell::ToolCall;
use ahash::AHashMap;
use nonempty::NonEmpty;

#[derive(Debug)]
pub struct Runnables {
    /// fixes that affect all files
    pub global: Option<NonEmpty<ToolCall>>,

    /// fixes that affect stack-specific files
    pub stack_specific: AHashMap<StackType, NonEmpty<ToolCall>>,
}

impl Runnables {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.global.is_none() && self.stack_specific.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        let mut result = self.global.as_ref().map_or(0, NonEmpty::len);
        for (_, sequence) in &self.stack_specific {
            result += sequence.len();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Operation;
    use crate::subshell::ToolCall;

    fn executable() -> ToolCall {
        ToolCall {
            operation: Operation::Fix,
            stack: None,
            name: "true".to_string(),
            command: conc::shell_command("true"),
        }
    }

    mod len {
        use super::super::Runnables;
        use super::executable;
        use crate::domain::StackType;
        use ahash::AHashMap;
        use nonempty::nonempty;

        #[test]
        fn empty() {
            let give = Runnables {
                global: None,
                stack_specific: AHashMap::new(),
            };
            let have = give.len();
            let want = 0;
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn single_global() {
            let give = Runnables {
                global: Some(nonempty![executable()]),
                stack_specific: AHashMap::new(),
            };
            let have = give.len();
            let want = 1;
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn multiple_globals() {
            let give = Runnables {
                global: Some(nonempty![executable(), executable()]),
                stack_specific: AHashMap::new(),
            };
            let have = give.len();
            let want = 2;
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn stack_specific() {
            let give = Runnables {
                global: None,
                stack_specific: AHashMap::from([
                    (StackType::Rust, nonempty![executable()]),
                    (StackType::Markdown, nonempty![executable(), executable()]),
                ]),
            };
            let have = give.len();
            let want = 3;
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn all_fields_set() {
            let give = Runnables {
                global: Some(nonempty![executable(), executable()]),
                stack_specific: AHashMap::from([(StackType::Rust, nonempty![executable()])]),
            };
            let have = give.len();
            let want = 3;
            pretty::assert_eq!(have, want);
        }
    }
}
