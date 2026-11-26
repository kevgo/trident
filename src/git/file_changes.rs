//! the files that a set of changes touches, and what happened to them

use crate::domain::File;
use crate::git::Repo;

/// what a set of changes did to a file, as seen from the workspace
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FileStatus {
    /// the file was added or modified and exists in the workspace
    Changed,

    /// the file was deleted or renamed away and doesn't exist in the workspace
    Removed,
}

/// a file and how it was changed
#[derive(Clone, Debug, Eq, PartialEq)]
struct FileChange {
    file: File,
    status: FileStatus,
}

/// a set of files and how they were changed, sorted and without duplicates
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FileChanges(Vec<FileChange>);

impl FileChanges {
    /// determines the status of the given files that Git reports as changed
    ///
    /// The status comes from the workspace rather than from Git's change type
    /// because several Git commands can report conflicting changes for the same file,
    /// for example a commit that deletes it and an untracked file that recreates it.
    /// The workspace is the source of truth for what the fixes and lints can process.
    pub(crate) fn from_paths(repo: &Repo, mut files: Vec<File>) -> FileChanges {
        files.sort_unstable();
        files.dedup();
        FileChanges(
            files
                .into_iter()
                .map(|file| {
                    let status = if repo.file_exists(&file) {
                        FileStatus::Changed
                    } else {
                        FileStatus::Removed
                    };
                    FileChange { file, status }
                })
                .collect(),
        )
    }

    /// provides the given files, which all exist in the workspace, as changed files
    pub(crate) fn from_existing(mut files: Vec<File>) -> FileChanges {
        files.sort_unstable();
        files.dedup();
        FileChanges(
            files
                .into_iter()
                .map(|file| FileChange {
                    file,
                    status: FileStatus::Changed,
                })
                .collect(),
        )
    }

    /// provides all touched files, including the removed ones
    pub(crate) fn all(&self) -> impl Iterator<Item = &File> {
        self.0.iter().map(|change| &change.file)
    }

    /// provides the touched files that exist in the workspace
    pub(crate) fn changed(&self) -> impl Iterator<Item = &File> {
        self.with_status(FileStatus::Changed)
    }

    /// provides the touched files that don't exist in the workspace anymore
    pub(crate) fn removed(&self) -> impl Iterator<Item = &File> {
        self.with_status(FileStatus::Removed)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn with_status(&self, status: FileStatus) -> impl Iterator<Item = &File> {
        self.0
            .iter()
            .filter(move |change| change.status == status)
            .map(|change| &change.file)
    }
}

#[cfg(test)]
impl FileChanges {
    /// provides the touched files with their status, for comparing in tests
    pub(crate) fn summary(&self) -> Vec<(&str, FileStatus)> {
        self.0
            .iter()
            .map(|change| (change.file.as_str(), change.status))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::FileChanges;
    use super::FileStatus::{Changed, Removed};
    use crate::domain::{File, Result};
    use crate::git::Repo;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn from_paths() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        repo.create_unstaged_files(&["existing.md", "other.md"]);
        fs::create_dir(repo.file_path("replaced-by-folder.md")).unwrap();
        let give = vec![
            File::from("other.md"),
            File::from("removed.md"),
            File::from("existing.md"),
            File::from("replaced-by-folder.md"),
            File::from("existing.md"),
        ];
        let have = FileChanges::from_paths(&repo, give);
        pretty::assert_eq!(
            have.summary(),
            vec![
                ("existing.md", Changed),
                ("other.md", Changed),
                ("removed.md", Removed),
                ("replaced-by-folder.md", Removed),
            ]
        );
        pretty::assert_eq!(
            have.changed().collect::<Vec<_>>(),
            vec![&File::from("existing.md"), &File::from("other.md")]
        );
        pretty::assert_eq!(
            have.removed().collect::<Vec<_>>(),
            vec![
                &File::from("removed.md"),
                &File::from("replaced-by-folder.md")
            ]
        );
        pretty::assert_eq!(have.all().count(), 4);
        Ok(())
    }

    #[test]
    fn empty() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        assert!(FileChanges::from_paths(&repo, vec![]).is_empty());
        Ok(())
    }
}
