use crate::git::Repo;
#[cfg(test)]
use std::fs;

impl Repo {
    #[cfg(test)]
    pub(crate) fn create_unstaged_file(&self, name: &str) {
        fs::write(self.file_path(name), "content").unwrap();
    }
}
