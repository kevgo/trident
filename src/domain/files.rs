use crate::domain::File;

/// a list of files that a tool should process
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Files(Vec<File>);

impl Files {
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
}

impl FromIterator<File> for Files {
    fn from_iter<I: IntoIterator<Item = File>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a> IntoIterator for &'a Files {
    type Item = &'a File;
    type IntoIter = std::slice::Iter<'a, File>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
