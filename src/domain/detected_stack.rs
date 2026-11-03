use crate::domain::{EnabledWhen, File, Files, Stack, StackType};

/// A stack that was detected in the workspace,
/// and the workspace files belonging to it.
#[must_use]
pub struct DetectedStack {
    pub stack: Box<dyn Stack>, // TODO: change this to StackType
    pub files: Files,
}

impl std::fmt::Debug for DetectedStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DetectedStack {\n")?;
        f.write_str("  stack: ")?;
        f.write_str(&self.stack.to_string())?;
        f.write_str("\n  files:\n")?;
        for file in &self.files {
            f.write_str("    - ")?;
            f.write_str(file.as_str())?;
            f.write_str("\n")?;
        }
        f.write_str("}")?;
        Ok(())
    }
}

impl PartialEq for DetectedStack {
    fn eq(&self, other: &Self) -> bool {
        let stacks_match = self.stack.stack_type() == other.stack.stack_type();
        let files_match = self.files == other.files;
        stacks_match && files_match
    }
}

impl Eq for DetectedStack {}

#[derive(Debug, PartialEq, Eq)]
pub struct DetectedStacks(Vec<DetectedStack>);

impl DetectedStacks {
    #[must_use]
    pub fn new(stacks: Vec<DetectedStack>) -> Self {
        DetectedStacks(stacks)
    }

    #[must_use]
    pub fn contains_file(&self, stack_type: StackType, file: &str) -> bool {
        let Some(stack) = self.get_stack(stack_type) else {
            return false;
        };
        stack.files.contains(file)
    }

    /// indicates whether a folder with the given name exists in the files,
    /// and whether it contains at least one file of the given type
    #[must_use]
    pub fn has_folder_containing_file(&self, stack_type: StackType, name: &str) -> bool {
        let Some(stack) = self.get_stack(stack_type) else {
            return false;
        };
        for file in &stack.files {
            if file.starts_with(name) {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn get_stack(&self, stack_type: StackType) -> Option<&DetectedStack> {
        self.0.iter().find(|s| s.stack.stack_type() == stack_type)
    }

    #[must_use]
    pub fn stack_enabled(&self, enabled_when: &EnabledWhen) -> bool {
        match enabled_when {
            EnabledWhen::Always => true,
            EnabledWhen::FilePresent {
                filename,
                stack_type,
            } => self.contains_file(*stack_type, filename),
            EnabledWhen::FolderContainingFileOfType {
                file_type: stack_type,
                folder: name,
            } => self.has_folder_containing_file(*stack_type, name),
        }
    }

    /// the stack type whose detected files include this path
    #[must_use]
    pub fn stack_type_for_file(&self, file: &File) -> Option<StackType> {
        for detected_stack in &self.0 {
            if detected_stack.files.contains_file(file) {
                return Some(detected_stack.stack.stack_type());
            }
        }
        None
    }
}

impl IntoIterator for DetectedStacks {
    type Item = DetectedStack;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a DetectedStacks {
    type Item = &'a DetectedStack;
    type IntoIter = std::slice::Iter<'a, DetectedStack>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    mod stack_type_for_file {
        use crate::domain::{DetectedStack, DetectedStacks, File, Files, StackType};
        use crate::stacks::Toml;
        use std::path::PathBuf;

        #[test]
        fn returns_type_for_file_in_the_detected_list() {
            let stacks = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Toml {}),
                files: Files::from(vec![PathBuf::from("changed.toml")]),
            }]);
            pretty::assert_eq!(
                stacks.stack_type_for_file(&File::from("changed.toml")),
                Some(StackType::Toml)
            );
        }

        #[test]
        fn ignores_same_type_file_not_in_the_detected_list() {
            let stacks = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Toml {}),
                files: Files::from(vec![PathBuf::from("changed.toml")]),
            }]);
            pretty::assert_eq!(
                stacks.stack_type_for_file(&File::from("untouched.toml")),
                None
            );
        }

        #[test]
        fn none_when_no_stacks() {
            let stacks = DetectedStacks::new(vec![]);
            pretty::assert_eq!(stacks.stack_type_for_file(&File::from("file.toml")), None);
        }
    }
}
