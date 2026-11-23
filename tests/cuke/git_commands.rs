//! Lets the end-to-end tests see which Git commands Trident runs.
//!
//! Trident runs with a `PATH` whose first entry contains the `git_double` binary, named `git`.
//! This double logs its invocations and then runs the real Git executable.

use regex::Regex;
use std::env::{self, consts::EXE_SUFFIX};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, OnceLock};
use tempfile::TempDir;

/// the Git double installed for this test run
static INSTALLED: OnceLock<GitDouble> = OnceLock::new();

#[derive(Debug)]
pub struct GitDouble {
    /// the value of the PATH environment variable that makes the Git double available as `git`
    path_var: std::ffi::OsString,

    /// the real Git executable that the double forwards its calls to
    real_git: PathBuf,
}

/// builds the Git double and installs it as `git` into a temporary directory
///
/// The installation exists as long as the returned `TempDir` does.
pub fn install_double(project_dir: &Path) -> TempDir {
    let real_git = which::which("git").expect("cannot find the Git executable");
    build(project_dir);
    let bin_dir = tempfile::Builder::new()
        .prefix("trident-git-double-")
        .tempdir()
        .expect("cannot create directory for the Git double");
    let built_path = project_dir
        .join("target")
        .join("release")
        .join(format!("git_double{EXE_SUFFIX}"));
    let installed_path = bin_dir.path().join(format!("git{EXE_SUFFIX}"));
    fs::copy(&built_path, &installed_path).unwrap_or_else(|err| {
        panic!(
            "cannot copy the Git double from {} to {}: {err}",
            built_path.display(),
            installed_path.display()
        )
    });
    let original_path = env::var_os("PATH").unwrap_or_default();
    let path_var = env::join_paths(
        std::iter::once(bin_dir.path().to_path_buf()).chain(env::split_paths(&original_path)),
    )
    .expect("cannot create the PATH for the Git double");
    INSTALLED
        .set(GitDouble { path_var, real_git })
        .expect("the Git double is already installed");
    bin_dir
}

/// compiles the Git double, which Cargo doesn't do automatically because it is a separate package
fn build(project_dir: &Path) {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .args(["build", "--release", "--quiet", "--package=git_double"])
        .current_dir(project_dir)
        .status()
        .expect("cannot run Cargo to build the Git double");
    assert!(status.success(), "cannot build the Git double");
}

/// configures the given command to log the Git commands it runs into the given file
pub fn configure(command: &mut tokio::process::Command, log_file: &Path) {
    let double = INSTALLED.get().expect("the Git double is not installed");
    command
        .env("PATH", &double.path_var)
        .env(git_double::REAL_GIT_VAR, &double.real_git)
        .env(git_double::LOG_FILE_VAR, log_file);
}

/// provides the Git commands logged into the given file
pub fn logged(log_file: &Path) -> Vec<String> {
    match fs::read_to_string(log_file) {
        Ok(content) => content.lines().map(normalize).collect(),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(err) => panic!("cannot read Git log {}: {err}", log_file.display()),
    }
}

/// replaces the parts of the given Git command that change from test run to test run with placeholders
///
/// Commit SHAs change because they include the commit time.
fn normalize(command: &str) -> String {
    static SHA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b[0-9a-f]{40}\b").unwrap());
    SHA.replace_all(command, "{SHA}").into_owned()
}
