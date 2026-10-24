use crate::config::ToolDefinition;
use crate::domain::StackType;
use ahash::AHashMap;

#[derive(Debug)]
pub struct Runnables {
    /// fixes that affect all files
    pub global: Option<Vec<ToolDefinition>>,

    /// fixes that affect stack-specific files
    pub stack_specific: AHashMap<StackType, Vec<ToolDefinition>>,
}

impl Runnables {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.global.is_none() && self.stack_specific.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        let mut result = self.global.as_ref().map_or(0, |global| global.len());
        for (_, sequence) in &self.stack_specific {
            result += sequence.len();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ToolDefinition;

    fn tool_def() -> ToolDefinition {
        ToolDefinition {
            name: None,
            command: "foo".to_string(),
        }
    }

    mod len {
        use super::super::Runnables;
        use super::tool_def;
        use crate::domain::StackType;
        use ahash::AHashMap;

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
                global: Some(vec![tool_def()]),
                stack_specific: AHashMap::new(),
            };
            let have = give.len();
            let want = 1;
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn multiple_globals() {
            let give = Runnables {
                global: Some(vec![tool_def(), tool_def()]),
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
                    (StackType::Rust, vec![tool_def()]),
                    (StackType::Markdown, vec![tool_def(), tool_def()]),
                ]),
            };
            let have = give.len();
            let want = 3;
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn all_fields_set() {
            let give = Runnables {
                global: Some(vec![tool_def(), tool_def()]),
                stack_specific: AHashMap::from([(StackType::Rust, vec![tool_def()])]),
            };
            let have = give.len();
            let want = 3;
            pretty::assert_eq!(have, want);
        }
    }
}
