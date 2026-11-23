use git_double::{LOG_FILE_VAR, REAL_GIT_VAR, log_entry};
use std::env;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let Some(real_git) = env::var_os(REAL_GIT_VAR) else {
        eprintln!("git double: environment variable {REAL_GIT_VAR} is not set");
        return ExitCode::FAILURE;
    };
    if let Some(log_path) = env::var_os(LOG_FILE_VAR) {
        // write the entire entry at once so that entries of concurrent Git calls don't interleave
        let texts: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();
        let entry = format!("{}\n", log_entry(&texts));
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .unwrap_or_else(|err| {
                panic!(
                    "git double: cannot open log file {}: {err}",
                    log_path.display()
                )
            });
        log_file.write_all(entry.as_bytes()).unwrap_or_else(|err| {
            panic!(
                "git double: cannot write log file {}: {err}",
                log_path.display()
            )
        });
    }
    let mut command = Command::new(real_git);
    command.args(&args);
    // Git calls that Git itself makes, for example in hooks or `git town`, are not Trident's calls
    command.env_remove(LOG_FILE_VAR);
    run(command)
}

/// replaces this process with the given command
#[cfg(unix)]
fn run(mut command: Command) -> ExitCode {
    use std::os::unix::process::CommandExt as _;
    let err = command.exec();
    eprintln!("git double: cannot run the real Git: {err}");
    ExitCode::FAILURE
}

/// runs the given command and exits with its exit code
#[cfg(not(unix))]
fn run(mut command: Command) -> ExitCode {
    match command.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(err) => {
            eprintln!("git double: cannot run the real Git: {err}");
            ExitCode::FAILURE
        }
    }
}
