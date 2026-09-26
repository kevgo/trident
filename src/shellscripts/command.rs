use std::path::{Path, PathBuf};
use std::process::Command;

// TODO: extract this stuff into the Conc crate.
// The conc CLI allows executing shell scripts inside a shell environment.
// Support for bash scripts on Windows is a missing feature for that.

/// Provides a `Command` instance that runs the given shell script in the most appropriate shell.
/// Unix uses `sh -c`.
/// Windows uses `cmd.exe /C` for ordinary commands and Git bash for .sh and .bash files.
#[must_use]
pub fn shell_command(expr: &str) -> Command {
    match git_bash_command(expr) {
        Some(bash_call) => bash_call,
        None => conc::shell_command(expr),
    }
}

/// runs a shell script through Git bash on Windows
fn git_bash_command(expr: &str) -> Option<Command> {
    if !is_shell_script_command(expr) {
        return None;
    }
    let bash = git_bash()?;
    let mut cmd = Command::new(bash);
    // `-c` keeps shell syntax (`args`, quotes, `&&`) working for the original command text.
    cmd.arg("-c");
    cmd.arg(expr);
    Some(cmd)
}

fn is_shell_script_command(command: &str) -> bool {
    let Some(program) = first_program(command) else {
        return false;
    };
    is_shell_script_path(&program)
}

fn first_program(command: &str) -> Option<String> {
    shlex::split(command)?.into_iter().next()
}

fn is_shell_script_path(program: &str) -> bool {
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    let extension = Path::new(name).extension();
    extension.is_some_and(|ext| ext.eq_ignore_ascii_case("sh") || ext.eq_ignore_ascii_case("bash"))
}

fn git_bash() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        locate_git_bash()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// locates `bash.exe` from a Git for Windows install
#[cfg(windows)]
fn locate_git_bash() -> Option<PathBuf> {
    if let Ok(git) = which::which("git")
        && let Some(bash) = bash_alongside_git(&git)
    {
        return Some(bash);
    }
    if let Ok(bash) = which::which("bash")
        && is_git_bash(&bash)
    {
        return Some(prefer_launcher(bash));
    }
    standard_git_bash()
}

/// walks up from `git.exe` to the Git install that contains `bash.exe`
#[cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Windows locates Git bash by walking up from git.exe"
    )
)]
fn bash_alongside_git(git: &Path) -> Option<PathBuf> {
    let mut dir = git.parent();
    let mut usr_bash = None;
    for _ in 0..6 {
        let Some(current) = dir else {
            break;
        };
        let launcher = current.join("bin").join("bash.exe");
        if launcher.is_file() && !is_usr_bin(&launcher) {
            return Some(launcher);
        }
        if usr_bash.is_none() {
            let candidate = current.join("usr").join("bin").join("bash.exe");
            if candidate.is_file() {
                usr_bash = Some(candidate);
            }
        }
        dir = current.parent();
    }
    usr_bash
}

/// `...\usr\bin\bash.exe` is Git's real bash, not the `bin\bash.exe` launcher
#[cfg_attr(
    not(windows),
    allow(dead_code, reason = "used while locating Git bash on Windows")
)]
fn is_usr_bin(path: &Path) -> bool {
    let Some(bin_dir) = path.parent() else {
        return false;
    };
    let Some(usr_dir) = bin_dir.parent() else {
        return false;
    };
    bin_dir
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("bin"))
        && usr_dir
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("usr"))
}

#[cfg(windows)]
fn is_git_bash(path: &Path) -> bool {
    is_bash_filename(path)
        && path
            .components()
            .any(|component| component.as_os_str().eq_ignore_ascii_case("Git"))
}

#[cfg(windows)]
fn is_bash_filename(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        name.eq_ignore_ascii_case("bash.exe") || name.eq_ignore_ascii_case("bash")
    })
}

/// prefers `Git\bin\bash.exe`, which sets up the MSYS environment for a native Windows parent
#[cfg(windows)]
fn prefer_launcher(bash: PathBuf) -> PathBuf {
    if !is_usr_bin(&bash) {
        return bash;
    }
    let Some(root) = bash.ancestors().nth(3) else {
        return bash;
    };
    let launcher = root.join("bin").join("bash.exe");
    if launcher.is_file() { launcher } else { bash }
}

#[cfg(windows)]
fn standard_git_bash() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        candidates.push(
            PathBuf::from(program_files)
                .join("Git")
                .join("bin")
                .join("bash.exe"),
        );
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles(x86)") {
        candidates.push(
            PathBuf::from(program_files)
                .join("Git")
                .join("bin")
                .join("bash.exe"),
        );
    }
    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::{bash_alongside_git, is_shell_script_command, shell_command};
    use maplit::hashmap;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn shell_script_paths() {
        let tests = hashmap! {
            "tests/fail.sh" => true,
            r"tests\fail.sh" => true,
            "tests/fail.sh --flag" => true,
            "\"tests/my script.sh\"" => true,
            "hooks/post_write.bash" => true,
            "echo hello" => false,
            "echo tests/fail.sh" => false,
            "sh -c tests/fail.sh" => false,
        };
        for (expr, want) in tests {
            assert_eq!(is_shell_script_command(expr), want);
        }
    }

    #[test]
    fn finds_launcher_above_mingw_git() {
        let root = TempDir::new().unwrap();
        touch(&root.path().join("mingw64").join("bin").join("git.exe"));
        touch(&root.path().join("bin").join("bash.exe"));
        touch(&root.path().join("usr").join("bin").join("bash.exe"));
        let git = root.path().join("mingw64").join("bin").join("git.exe");
        let have = bash_alongside_git(&git).unwrap();
        let want = root.path().join("bin").join("bash.exe");
        assert_eq!(have, want);
    }

    #[test]
    fn finds_launcher_above_cmd_git() {
        let root = TempDir::new().unwrap();
        touch(&root.path().join("cmd").join("git.exe"));
        touch(&root.path().join("bin").join("bash.exe"));
        let git = root.path().join("cmd").join("git.exe");
        let have = bash_alongside_git(&git).unwrap();
        let want = root.path().join("bin").join("bash.exe");
        assert_eq!(have, want);
    }

    #[test]
    fn finds_usr_bash_when_launcher_is_missing() {
        let root = TempDir::new().unwrap();
        touch(&root.path().join("mingw64").join("bin").join("git.exe"));
        touch(&root.path().join("usr").join("bin").join("bash.exe"));
        let git = root.path().join("mingw64").join("bin").join("git.exe");
        let have = bash_alongside_git(&git).unwrap();
        let want = root.path().join("usr").join("bin").join("bash.exe");
        assert_eq!(have, want);
    }

    #[test]
    fn finds_launcher_next_to_bin_git() {
        let root = TempDir::new().unwrap();
        touch(&root.path().join("bin").join("git.exe"));
        touch(&root.path().join("bin").join("bash.exe"));
        let git = root.path().join("bin").join("git.exe");
        let have = bash_alongside_git(&git).unwrap();
        let want = root.path().join("bin").join("bash.exe");
        assert_eq!(have, want);
    }

    #[test]
    #[cfg(unix)]
    fn unix_shell_script_uses_sh() {
        let command = shell_command("tests/fail.sh");
        assert_eq!(command.get_program(), "sh");
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args, ["-c", "tests/fail.sh"]);
    }

    #[test]
    #[cfg(windows)]
    fn windows_ordinary_command_uses_cmd() {
        let command = shell_command("echo hello");
        assert_eq!(command.get_program(), "cmd.exe");
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args, ["/C", "echo hello"]);
    }

    #[test]
    #[cfg(windows)]
    fn windows_shell_script_uses_git_bash() {
        let bash = super::git_bash().expect("Git bash");
        let command = shell_command("tests/fail.sh");
        assert_eq!(command.get_program(), bash.as_os_str());
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args, ["-c", "tests/fail.sh"]);
    }

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "").unwrap();
    }
}
