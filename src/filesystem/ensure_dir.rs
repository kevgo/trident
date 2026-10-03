use crate::domain::{Result, UserError};
use std::fs;
use std::path::Path;

/// ensures the given directory exists, creating it if it doesn't
pub fn ensure_dir(path: &Path) -> Result<()> {
    // fast path: the directory already exists
    if path.is_dir() {
        return Ok(());
    }
    // slow path: create the directory and all its missing parents
    fs::create_dir_all(path).map_err(|err| UserError::CannotCreateDirectory {
        path: path.to_path_buf(),
        err: err.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::ensure_dir;
    use crate::domain::UserError;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn creates_missing_directory() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hooks");
        ensure_dir(&path).unwrap();
        assert!(path.is_dir());
    }

    #[test]
    fn creates_nested_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a").join("b").join("c");
        ensure_dir(&path).unwrap();
        assert!(path.is_dir());
    }

    #[test]
    fn directory_already_exists() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hooks");
        fs::create_dir(&path).unwrap();
        ensure_dir(&path).unwrap();
        assert!(path.is_dir());
    }

    #[test]
    fn path_is_a_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hooks");
        fs::write(&path, "").unwrap();
        let have = ensure_dir(&path);
        match have {
            Err(UserError::CannotCreateDirectory {
                path: have_path,
                err,
            }) => {
                assert_eq!(have_path, path);
                assert!(!err.is_empty());
            }
            other => panic!("expected CannotCreateDirectory, got {other:?}"),
        }
    }
}
