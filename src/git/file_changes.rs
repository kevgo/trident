//! the files that a set of changes touches, and what happened to them

use crate::domain::File;
use crate::git::Repo;

/// what a set of changes did to a file, as seen from the workspace
///
/// The order matters: when several sources report conflicting statuses for the same file,
/// the first variant wins.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum FileStatus {
    /// the file was added or modified and exists in the workspace
    Changed,

    /// the file was deleted or renamed away and doesn't exist in the workspace
    Removed,
}

/// a file and how it was changed
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FileChange {
    file: File,
    status: FileStatus,
}

/// a set of files and how they were changed, sorted and without duplicates
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FileChanges(Vec<FileChange>);

impl FromIterator<(File, FileStatus)> for FileChanges {
    /// If the same file occurs several times, it counts as changed if any occurrence is.
    /// An example is a file that the index deletes and that exists as an untracked file.
    fn from_iter<I: IntoIterator<Item = (File, FileStatus)>>(iter: I) -> Self {
        let mut changes: Vec<FileChange> = iter
            .into_iter()
            .map(|(file, status)| FileChange { file, status })
            .collect();
        changes.sort_unstable();
        changes.dedup_by(|later, earlier| later.file == earlier.file);
        FileChanges(changes)
    }
}

impl FileChanges {
    /// determines the status of the given files by looking them up in the workspace
    pub(crate) fn from_paths(repo: &Repo, files: Vec<File>) -> FileChanges {
        files
            .into_iter()
            .map(|file| {
                let status = if repo.file_exists(&file) {
                    FileStatus::Changed
                } else {
                    FileStatus::Removed
                };
                (file, status)
            })
            .collect()
    }

    /// provides the given files, which all exist in the workspace, as changed files
    pub(crate) fn from_existing(files: Vec<File>) -> FileChanges {
        files
            .into_iter()
            .map(|file| (file, FileStatus::Changed))
            .collect()
    }

    /// provides whether this set contains the given file
    pub(crate) fn contains(&self, file: &File) -> bool {
        self.0
            .binary_search_by(|change| change.file.cmp(file))
            .is_ok()
    }

    /// adds the given changes, keeping the existing status of files that both contain
    pub(crate) fn merge(mut self, other: FileChanges) -> FileChanges {
        let added: Vec<FileChange> = other
            .0
            .into_iter()
            .filter(|change| !self.contains(&change.file))
            .collect();
        self.0.extend(added);
        self.0.sort_unstable();
        self
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
    fn from_iter_prefers_changed_for_duplicates() {
        let have: FileChanges = vec![
            (File::from("b.md"), Removed),
            (File::from("a.md"), Removed),
            (File::from("b.md"), Changed),
            (File::from("a.md"), Removed),
        ]
        .into_iter()
        .collect();
        pretty::assert_eq!(have.summary(), vec![("a.md", Removed), ("b.md", Changed)]);
    }

    #[test]
    fn contains() {
        let changes = FileChanges::from_existing(vec![File::from("b.md"), File::from("a.md")]);
        assert!(changes.contains(&File::from("a.md")));
        assert!(changes.contains(&File::from("b.md")));
        assert!(!changes.contains(&File::from("c.md")));
    }

    #[test]
    fn merge_keeps_own_status() {
        let own: FileChanges = vec![(File::from("b.md"), Removed), (File::from("c.md"), Changed)]
            .into_iter()
            .collect();
        let other = FileChanges::from_existing(vec![File::from("a.md"), File::from("b.md")]);
        let have = own.merge(other);
        pretty::assert_eq!(
            have.summary(),
            vec![("a.md", Changed), ("b.md", Removed), ("c.md", Changed)]
        );
    }

    #[test]
    fn empty() -> Result<()> {
        let dir = TempDir::new().unwrap();
        let repo = Repo::init(dir.path())?;
        assert!(FileChanges::from_paths(&repo, vec![]).is_empty());
        Ok(())
    }
}
