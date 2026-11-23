//! A test double for the Git executable.
//!
//! The end-to-end tests place this binary, named `git`, first in the `PATH` of the Trident process they test.
//! It appends each invocation to the log file named by [`LOG_FILE_VAR`]
//! and then runs the real Git executable named by [`REAL_GIT_VAR`].

/// name of the environment variable that contains the path of the file to log Git invocations into
pub const LOG_FILE_VAR: &str = "TRIDENT_GIT_DOUBLE_LOG";

/// name of the environment variable that contains the path of the real Git executable
pub const REAL_GIT_VAR: &str = "TRIDENT_GIT_DOUBLE_REAL_GIT";

/// provides the log entry for a Git invocation with the given arguments
#[must_use]
pub fn log_entry<S: AsRef<str>>(args: &[S]) -> String {
    let mut result = String::from("git");
    for arg in args {
        result.push(' ');
        result.push_str(&quote(arg.as_ref()));
    }
    result
}

/// quotes the given argument the way a POSIX shell would need it
fn quote(arg: &str) -> String {
    let is_safe = |c: char| c.is_ascii_alphanumeric() || "@%+=:,./_-^~".contains(c);
    if !arg.is_empty() && arg.chars().all(is_safe) {
        return arg.to_string();
    }
    format!("'{}'", arg.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {

    mod log_entry {
        use super::super::log_entry;

        #[test]
        fn no_args() {
            let give: [&str; 0] = [];
            assert_eq!(log_entry(&give), "git");
        }

        #[test]
        fn simple_args() {
            let give = ["diff", "--name-only", "-z", "HEAD"];
            assert_eq!(log_entry(&give), "git diff --name-only -z HEAD");
        }

        #[test]
        fn arg_with_space() {
            let give = ["add", "my file.md"];
            assert_eq!(log_entry(&give), "git add 'my file.md'");
        }

        #[test]
        fn arg_with_single_quote() {
            let give = ["commit", "-m", "it's done"];
            assert_eq!(log_entry(&give), r"git commit -m 'it'\''s done'");
        }

        #[test]
        fn empty_arg() {
            let give = ["config", ""];
            assert_eq!(log_entry(&give), "git config ''");
        }
    }
}
