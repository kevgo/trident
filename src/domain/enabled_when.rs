use std::path::Path;

use crate::domain::StackType;

/// describes under which conditions a tool is enabled
pub enum EnabledWhen {
    /// the tool is always enabled
    Always,

    /// the tool is enabled when a file with the given name
    FilePresent {
        filename: &'static str,
        stack_type: StackType,
    },

    // the tool is enabled when a folder with the given name exists
    // and it contains at least one file of the given type
    FolderContainingFileOfType {
        folder: &'static str,
        file_type: StackType,
    },
}

impl EnabledWhen {
    /// whether the tool is enabled based on files present on disk
    ///
    /// This is useful when the detected stacks contains only a subset of workspace files
    /// (uncommitted, staged, or changed on the current branch).
    #[must_use]
    pub fn enabled_on_disk(&self) -> bool {
        match self {
            Self::Always => true,
            Self::FilePresent { filename, .. } => Path::new(filename).exists(),
            Self::FolderContainingFileOfType { folder, .. } => Path::new(folder).exists(),
        }
    }
}
