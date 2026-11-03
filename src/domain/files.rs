use crate::domain::File;
use crate::domain::Ignores;
use std::convert::Into;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Files(Vec<File>);

impl Files {
    #[must_use]
    pub fn empty() -> Self {
        Self(vec![])
    }

    #[must_use]
    pub fn contains<AS: AsRef<str>>(&self, file: AS) -> bool {
        let file = file.as_ref();
        self.0.iter().any(|self_file| self_file.as_str() == file)
    }

    #[must_use]
    pub fn contains_file(&self, file: &File) -> bool {
        self.0.contains(file)
    }

    #[must_use]
    pub fn into_strings(self) -> Vec<String> {
        self.0.into_iter().map(|file| file.to_string()).collect()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, file: File) {
        self.0.push(file);
    }

    /// adds the given files that this collection doesn't contain yet and sorts the result
    pub fn merge(&mut self, other: &Files) {
        self.0.extend_from_slice(&other.0);
        self.0.sort_unstable();
        self.0.dedup();
    }

    /// provides a Files collection containing the files in this collection without the given files
    #[must_use]
    pub fn remove(&self, ignores: &Ignores) -> Files {
        let files: Vec<File> = self
            .0
            .iter()
            .filter(|file| !ignores.matches_self_or_parent(file.as_ref()))
            .cloned()
            .collect();
        Files(files)
    }

    pub fn sort_unstable(&mut self) {
        self.0.sort_unstable();
    }
}

impl<'a> IntoIterator for &'a Files {
    type Item = &'a File;
    type IntoIter = std::slice::Iter<'a, File>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Vec<PathBuf>> for Files {
    fn from(paths: Vec<PathBuf>) -> Self {
        Self(paths.into_iter().map(Into::into).collect())
    }
}

impl From<&Vec<String>> for Files {
    fn from(paths: &Vec<String>) -> Self {
        Self(paths.iter().map(Into::into).collect())
    }
}

#[cfg(test)]
impl From<Vec<&str>> for Files {
    fn from(paths: Vec<&str>) -> Self {
        Self(paths.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {

    mod merge {
        use super::super::Files;

        #[test]
        fn adds_missing_files_without_duplicates() {
            let mut files = Files::from(vec!["b.rs", "c.rs"]);
            files.merge(&Files::from(vec!["c.rs", "a.rs"]));
            let want = Files::from(vec!["a.rs", "b.rs", "c.rs"]);
            assert_eq!(files, want);
        }

        #[test]
        fn empty_other() {
            let mut files = Files::from(vec!["a.rs"]);
            files.merge(&Files::empty());
            assert_eq!(files, Files::from(vec!["a.rs"]));
        }
    }

    mod remove {
        use super::super::Files;
        use crate::domain::Ignores;
        use big_s::S;
        use std::path::Path;

        #[test]
        fn matching_removes() {
            let files = Files::from(vec!["a.rs", "b.rs", "c.rs"]);
            let exclude = Ignores::new(&[S("b.rs")], Path::new("./")).unwrap();
            let have = files.remove(&exclude);
            let want = Files::from(vec!["a.rs", "c.rs"]);
            assert_eq!(have, want);
        }

        #[test]
        fn empty_exclude() {
            let files = Files::from(vec!["a.rs", "b.rs"]);
            let have = files.remove(&Ignores::new(&[], Path::new("./")).unwrap());
            let want = files;
            assert_eq!(have, want);
        }

        #[test]
        fn remove_all() {
            let files = Files::from(vec!["a.rs", "b.rs"]);
            let ignores = Ignores::new(&[S("a.rs"), S("b.rs")], Path::new("./")).unwrap();
            let have = files.remove(&ignores);
            let want = Files::empty();
            assert_eq!(have, want);
        }

        #[test]
        fn non_matching_removes() {
            let files = Files::from(vec!["a.rs", "b.rs"]);
            let ignores = Ignores::new(&[S("c.rs")], Path::new("./")).unwrap();
            let have = files.remove(&ignores);
            let want = files;
            assert_eq!(have, want);
        }

        #[test]
        fn empty_files() {
            let ignores = Ignores::new(&[S("a.rs")], Path::new("./")).unwrap();
            let have = Files::empty().remove(&ignores);
            let want = Files::empty();
            assert_eq!(have, want);
        }

        #[test]
        fn glob_extension() {
            let files = Files::from(vec!["a.rs", "b.rs", "c.toml"]);
            let ignores = Ignores::new(&[S("*.rs")], Path::new("./")).unwrap();
            let have = files.remove(&ignores);
            let want = Files::from(vec!["c.toml"]);
            assert_eq!(have, want);
        }

        #[test]
        fn glob_extension_matches_nested() {
            let files = Files::from(vec!["a.rs", "src/b.rs", "src/nested/c.rs", "readme.md"]);
            let ignores = Ignores::new(&[S("*.rs")], Path::new("./")).unwrap();
            let have = files.remove(&ignores);
            let want = Files::from(vec!["readme.md"]);
            assert_eq!(have, want);
        }

        #[test]
        fn glob_recursive() {
            let files = Files::from(vec![
                "style.css",
                "style.min.css",
                "dist/app.min.css",
                "dist/app.css",
            ]);
            let ignores = Ignores::new(&[S("**/*.min.css")], Path::new("./")).unwrap();
            let have = files.remove(&ignores);
            let want = Files::from(vec!["style.css", "dist/app.css"]);
            assert_eq!(have, want);
        }

        #[test]
        fn glob_directory() {
            let files = Files::from(vec!["main.rs", "vendor/lib.rs", "vendor/nested/foo.rs"]);
            let ignores = Ignores::new(&[S("vendor/")], Path::new("./")).unwrap();
            let have = files.remove(&ignores);
            let want = Files::from(vec!["main.rs"]);
            assert_eq!(have, want);
        }

        #[test]
        fn glob_single_directory() {
            let files = Files::from(vec!["a.rs", "src/a.rs", "src/nested/a.rs"]);
            let ignores = Ignores::new(&[S("src/*.rs")], Path::new("./")).unwrap();
            let have = files.remove(&ignores);
            let want = Files::from(vec!["a.rs", "src/nested/a.rs"]);
            assert_eq!(have, want);
        }
    }
}
