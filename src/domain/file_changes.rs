//! the files that a set of changes touches, and what happened to them

use crate::domain::{File, Files, Ignores};

/// what a set of changes did to a file, as seen from the workspace
///
/// The order matters: when several sources report conflicting statuses for the same file,
/// the first variant wins.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FileStatus {
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
pub struct FileChanges(Vec<FileChange>);

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
    /// provides the given files, which all exist in the workspace, as changed files
    #[must_use]
    pub fn from_existing(files: Vec<File>) -> FileChanges {
        files
            .into_iter()
            .map(|file| (file, FileStatus::Changed))
            .collect()
    }

    /// provides whether this set contains the given file
    #[must_use]
    pub fn contains(&self, file: &File) -> bool {
        self.status(file).is_some()
    }

    /// provides whether this set contains the given file as a file that exists in the workspace
    #[must_use]
    pub fn contains_changed(&self, file: &File) -> bool {
        self.status(file) == Some(FileStatus::Changed)
    }

    /// adds the given changes, keeping the existing status of files that both contain
    #[must_use]
    pub fn merge(mut self, other: FileChanges) -> FileChanges {
        let added: Vec<FileChange> = other
            .0
            .into_iter()
            .filter(|change| !self.contains(&change.file))
            .collect();
        self.0.extend(added);
        self.0.sort_unstable();
        self
    }

    /// provides all touched files and their status, including the removed ones
    pub fn iter(&self) -> impl Iterator<Item = (&File, FileStatus)> {
        self.0.iter().map(|change| (&change.file, change.status))
    }

    /// provides all touched files, including the removed ones
    pub fn all(&self) -> impl Iterator<Item = &File> {
        self.0.iter().map(|change| &change.file)
    }

    /// provides the touched files that exist in the workspace
    pub fn changed(&self) -> impl Iterator<Item = &File> {
        self.with_status(FileStatus::Changed)
    }

    /// provides the touched files that exist in the workspace and aren't ignored
    #[must_use]
    pub fn changed_except(&self, ignores: &Ignores) -> Files {
        self.changed()
            .filter(|file| !ignores.matches_self_or_parent(file.as_ref()))
            .cloned()
            .collect()
    }

    /// provides the touched files that don't exist in the workspace anymore
    pub fn removed(&self) -> impl Iterator<Item = &File> {
        self.with_status(FileStatus::Removed)
    }

    /// provides whether this set contains files that exist in the workspace
    #[must_use]
    pub fn has_changed(&self) -> bool {
        self.changed().next().is_some()
    }

    /// provides whether this set contains files that don't exist in the workspace anymore
    #[must_use]
    pub fn has_removed(&self) -> bool {
        self.removed().next().is_some()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// provides the number of touched files, including the removed ones
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn status(&self, file: &File) -> Option<FileStatus> {
        self.0
            .binary_search_by(|change| change.file.cmp(file))
            .ok()
            .map(|index| self.0[index].status)
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
    use crate::domain::File;

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
    fn empty() {
        assert!(FileChanges::from_existing(vec![]).is_empty());
    }

    mod changed_except {
        use crate::domain::{File, FileChanges, FileStatus, Files, Ignores};
        use big_s::S;
        use std::path::Path;

        fn changes(paths: Vec<&str>) -> FileChanges {
            FileChanges::from_existing(paths.into_iter().map(File::from).collect())
        }

        /// the given files, in the order that `FileChanges` provides them
        fn files_of(paths: Vec<&str>) -> Files {
            let mut files: Vec<File> = paths.into_iter().map(File::from).collect();
            files.sort();
            files.into_iter().collect()
        }

        #[test]
        fn skips_removed_files() {
            let files: FileChanges = vec![
                (File::from("a.rs"), FileStatus::Changed),
                (File::from("b.rs"), FileStatus::Removed),
            ]
            .into_iter()
            .collect();
            let have = files.changed_except(&Ignores::empty());
            let want = files_of(vec!["a.rs"]);
            assert_eq!(have, want);
        }

        #[test]
        fn matching_removes() {
            let files = changes(vec!["a.rs", "b.rs", "c.rs"]);
            let exclude = Ignores::new(&[S("b.rs")], Path::new("./")).unwrap();
            let have = files.changed_except(&exclude);
            let want = files_of(vec!["a.rs", "c.rs"]);
            assert_eq!(have, want);
        }

        #[test]
        fn empty_exclude() {
            let files = changes(vec!["a.rs", "b.rs"]);
            let have = files.changed_except(&Ignores::new(&[], Path::new("./")).unwrap());
            let want = files_of(vec!["a.rs", "b.rs"]);
            assert_eq!(have, want);
        }

        #[test]
        fn remove_all() {
            let files = changes(vec!["a.rs", "b.rs"]);
            let ignores = Ignores::new(&[S("a.rs"), S("b.rs")], Path::new("./")).unwrap();
            let have = files.changed_except(&ignores);
            let want = Files::default();
            assert_eq!(have, want);
        }

        #[test]
        fn non_matching_removes() {
            let files = changes(vec!["a.rs", "b.rs"]);
            let ignores = Ignores::new(&[S("c.rs")], Path::new("./")).unwrap();
            let have = files.changed_except(&ignores);
            let want = files_of(vec!["a.rs", "b.rs"]);
            assert_eq!(have, want);
        }

        #[test]
        fn empty_files() {
            let ignores = Ignores::new(&[S("a.rs")], Path::new("./")).unwrap();
            let have = FileChanges::default().changed_except(&ignores);
            let want = Files::default();
            assert_eq!(have, want);
        }

        #[test]
        fn glob_extension() {
            let files = changes(vec!["a.rs", "b.rs", "c.toml"]);
            let ignores = Ignores::new(&[S("*.rs")], Path::new("./")).unwrap();
            let have = files.changed_except(&ignores);
            let want = files_of(vec!["c.toml"]);
            assert_eq!(have, want);
        }

        #[test]
        fn glob_extension_matches_nested() {
            let files = changes(vec!["a.rs", "src/b.rs", "src/nested/c.rs", "readme.md"]);
            let ignores = Ignores::new(&[S("*.rs")], Path::new("./")).unwrap();
            let have = files.changed_except(&ignores);
            let want = files_of(vec!["readme.md"]);
            assert_eq!(have, want);
        }

        #[test]
        fn glob_recursive() {
            let files = changes(vec![
                "style.css",
                "style.min.css",
                "dist/app.min.css",
                "dist/app.css",
            ]);
            let ignores = Ignores::new(&[S("**/*.min.css")], Path::new("./")).unwrap();
            let have = files.changed_except(&ignores);
            let want = files_of(vec!["style.css", "dist/app.css"]);
            assert_eq!(have, want);
        }

        #[test]
        fn glob_directory() {
            let files = changes(vec!["main.rs", "vendor/lib.rs", "vendor/nested/foo.rs"]);
            let ignores = Ignores::new(&[S("vendor/")], Path::new("./")).unwrap();
            let have = files.changed_except(&ignores);
            let want = files_of(vec!["main.rs"]);
            assert_eq!(have, want);
        }

        #[test]
        fn glob_single_directory() {
            let files = changes(vec!["a.rs", "src/a.rs", "src/nested/a.rs"]);
            let ignores = Ignores::new(&[S("src/*.rs")], Path::new("./")).unwrap();
            let have = files.changed_except(&ignores);
            let want = files_of(vec!["a.rs", "src/nested/a.rs"]);
            assert_eq!(have, want);
        }
    }
}
