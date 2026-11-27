use crate::domain::{EnabledWhen, File, FileChanges, FileStatus, Ignores, Stack, StackType};
use crate::stacks;
use std::path::Path;

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

    /// provides these stacks, with all workspace files added to the stacks that contain removed files
    ///
    /// Removing a file can break other files in its stack, for example the ones importing it.
    /// Lints therefore need to check all files of such stacks.
    #[must_use]
    pub fn with_stacks_of_removed_files(self, ignores: &Ignores) -> DetectedStacks {
        self.with_stacks_of_removed_files_in(Path::new("./"), ignores)
    }

    /// provides these stacks, with all files under `dir` added to the stacks that contain removed files
    #[must_use]
    pub fn with_stacks_of_removed_files_in(self, dir: &Path, ignores: &Ignores) -> DetectedStacks {
        if !self.has_removed_files() {
            return self;
        }
        let mut workspace = stacks::discover_all_in(dir, ignores);
        let stacks = self
            .0
            .into_iter()
            .map(|stack| {
                if !stack.files.has_removed() {
                    return stack;
                }
                let Some(workspace_stack) = workspace.remove_stack(stack.stack.stack_type()) else {
                    return stack;
                };
                DetectedStack {
                    stack: stack.stack,
                    files: stack.files.merge(workspace_stack.files),
                }
            })
            .collect();
        DetectedStacks(stacks)
    }

    /// removes the stack with the given type from these stacks and provides it
    fn remove_stack(&mut self, stack_type: StackType) -> Option<DetectedStack> {
        let index = self
            .0
            .iter()
            .position(|s| s.stack.stack_type() == stack_type)?;
        Some(self.0.remove(index))
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
    mod with_stacks_of_removed_files {
        use crate::domain::{DetectedStacks, File, FileChanges, FileStatus, Ignores};
        use crate::stacks::from_changes;
        use std::fs;
        use tempfile::TempDir;

        /// the stacks of the given changed and removed files
        fn stacks(changed: &[&str], removed: &[&str]) -> DetectedStacks {
            let changes: FileChanges = changed
                .iter()
                .map(|file| (File::from(*file), FileStatus::Changed))
                .chain(
                    removed
                        .iter()
                        .map(|file| (File::from(*file), FileStatus::Removed)),
                )
                .collect();
            from_changes(&changes, &Ignores::empty())
        }

        fn file_names(stacks: &DetectedStacks) -> Vec<String> {
            let mut names: Vec<String> = stacks
                .with_changed_files()
                .flat_map(|stack| stack.files.changed().map(ToString::to_string))
                .collect();
            names.sort();
            names
        }

        #[test]
        fn no_removed_files() {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("other.md"), "").unwrap();
            let affected = stacks(&["changed.md"], &[]);
            let have = affected.with_stacks_of_removed_files_in(dir.path(), &Ignores::empty());
            pretty::assert_eq!(have, stacks(&["changed.md"], &[]));
        }

        #[test]
        fn adds_all_files_of_the_removed_stack_only() {
            let dir = TempDir::new().unwrap();
            let in_stack = dir.path().join("in_stack.md");
            fs::write(&in_stack, "").unwrap();
            fs::write(dir.path().join("other_stack.py"), "").unwrap();
            let affected = stacks(&["changed.toml"], &["removed.md"]);
            let have = affected.with_stacks_of_removed_files_in(dir.path(), &Ignores::empty());
            let mut want = vec![
                in_stack.to_string_lossy().to_string(),
                "changed.toml".to_string(),
            ];
            want.sort();
            pretty::assert_eq!(file_names(&have), want);
        }

        #[test]
        fn keeps_changed_files_not_discoverable_in_the_workspace() {
            let dir = TempDir::new().unwrap();
            let affected = stacks(&[".hidden/changed.md"], &["removed.md"]);
            let have = affected.with_stacks_of_removed_files_in(dir.path(), &Ignores::empty());
            pretty::assert_eq!(file_names(&have), vec![".hidden/changed.md"]);
        }

        #[test]
        fn removed_stack_has_no_remaining_files() {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("other_stack.py"), "").unwrap();
            let affected = stacks(&[], &["removed.md"]);
            let have = affected.with_stacks_of_removed_files_in(dir.path(), &Ignores::empty());
            pretty::assert_eq!(have, stacks(&[], &["removed.md"]));
            pretty::assert_eq!(have.with_changed_files().count(), 0);
        }
    }

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
