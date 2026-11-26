use crate::domain::{EnabledWhen, File, FileChanges, FileStatus, Stack, StackType};

/// A stack that the files in scope belong to,
/// and how these files were changed.
///
/// A stack whose files were all removed from the workspace has no files to process,
/// but removing them can break other files of the stack.
#[must_use]
pub struct DetectedStack {
    pub stack: Box<dyn Stack>,
    pub files: FileChanges,
}

impl DetectedStack {
    /// provides whether this stack contains files that exist in the workspace,
    /// i.e. files that tools can process
    #[must_use]
    pub fn has_changed_files(&self) -> bool {
        self.files.has_changed()
    }
}

impl std::fmt::Debug for DetectedStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DetectedStack {\n")?;
        f.write_str("  stack: ")?;
        f.write_str(&self.stack.to_string())?;
        f.write_str("\n  files:\n")?;
        for (file, status) in self.files.iter() {
            f.write_str("    - ")?;
            f.write_str(file.as_str())?;
            if status == FileStatus::Removed {
                f.write_str(" (removed)")?;
            }
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

#[derive(Debug, Default, PartialEq, Eq)]
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
        stack
            .files
            .changed()
            .any(|changed| changed.as_str() == file)
    }

    /// indicates whether a folder with the given name exists in the files,
    /// and whether it contains at least one file of the given type
    #[must_use]
    pub fn has_folder_containing_file(&self, stack_type: StackType, name: &str) -> bool {
        let Some(stack) = self.get_stack(stack_type) else {
            return false;
        };
        stack.files.changed().any(|file| file.starts_with(name))
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// provides whether any of these stacks contains files that don't exist in the workspace anymore
    #[must_use]
    pub fn has_removed_files(&self) -> bool {
        self.0.iter().any(|stack| stack.files.has_removed())
    }

    /// provides the stacks that contain files that exist in the workspace,
    /// i.e. the stacks that have files for tools to process
    pub fn with_changed_files(&self) -> impl Iterator<Item = &DetectedStack> {
        self.0.iter().filter(|stack| stack.has_changed_files())
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

    /// the stack type whose detected files that exist in the workspace include this path
    #[must_use]
    pub fn stack_type_for_file(&self, file: &File) -> Option<StackType> {
        self.0
            .iter()
            .find(|detected_stack| detected_stack.files.contains_changed(file))
            .map(|detected_stack| detected_stack.stack.stack_type())
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
        use crate::domain::{
            DetectedStack, DetectedStacks, File, FileChanges, FileStatus, StackType,
        };
        use crate::stacks::Toml;

        #[test]
        fn returns_type_for_file_in_the_detected_list() {
            let stacks = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Toml {}),
                files: FileChanges::from_existing(vec![File::from("changed.toml")]),
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
                files: FileChanges::from_existing(vec![File::from("changed.toml")]),
            }]);
            pretty::assert_eq!(
                stacks.stack_type_for_file(&File::from("untouched.toml")),
                None
            );
        }

        #[test]
        fn ignores_removed_file_in_the_detected_list() {
            let stacks = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Toml {}),
                files: vec![(File::from("removed.toml"), FileStatus::Removed)]
                    .into_iter()
                    .collect(),
            }]);
            pretty::assert_eq!(
                stacks.stack_type_for_file(&File::from("removed.toml")),
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
