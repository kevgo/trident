use crate::apps::{GetRTACmdArgs, get_rta_command};
use crate::domain::{File, Result, UserError};
use big_s::S;
use std::path::Path;

/// provides the paths (relative to the current directory) of all files that contain `pattern`
pub fn files_with_matches(pattern: &str, ignores: &[String]) -> Result<Vec<File>> {
    files_with_matches_in(pattern, None, ignores)
}

fn files_with_matches_in(
    pattern: &str,
    path: Option<&Path>,
    ignores: &[String],
) -> Result<Vec<File>> {
    let mut args = vec![S("--files-with-matches"), S("--fixed-strings")];
    for ignore in ignores {
        args.push(format!("--glob=!{ignore}"));
    }
    args.push(pattern.to_string());
    let Some(executable) = get_rta_command(&GetRTACmdArgs {
        name: S("ripgrep"),
        app: &rta::applications::RipGrep {},
        args,
        version: None,
    })?
    else {
        return Ok(vec![]);
    };
    let mut command = executable.command;
    if let Some(path) = path {
        command.current_dir(path);
    }
    let output = command
        .output()
        .map_err(|err| UserError::CannotRunRipgrep {
            msg: err.to_string(),
        })?;
    check_exit_code(output.status.code(), &output.stderr)?;
    Ok(parse_stdout(&String::from_utf8_lossy(&output.stdout)))
}

/// exit code 0: matches found, exit code 1: no matches found
fn check_exit_code(code: Option<i32>, stderr: &[u8]) -> Result<()> {
    match code {
        Some(0 | 1) => Ok(()),
        _ => Err(UserError::CannotRunRipgrep {
            msg: String::from_utf8_lossy(stderr).into_owned(),
        }),
    }
}

fn parse_stdout(stdout: &str) -> Vec<File> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(File::from)
        .collect()
}

#[cfg(test)]
mod tests {

    mod parse_stdout {
        use super::super::parse_stdout;

        #[test]
        fn splits_paths() {
            let have = parse_stdout("a.txt\nb.txt\n");
            assert_eq!(have, vec!["a.txt".into(), "b.txt".into()]);
        }

        #[test]
        fn skips_empty_lines() {
            let have = parse_stdout("a.txt\n\nb.txt\n\n");
            assert_eq!(have, vec!["a.txt".into(), "b.txt".into()]);
        }

        #[test]
        fn empty() {
            pretty::assert_eq!(parse_stdout(""), vec![]);
        }
    }

    mod finds_matches_in {
        use super::super::files_with_matches_in;
        use big_s::S;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn finds_matching_files() {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("hit.txt"), "needle here").unwrap();
            fs::write(dir.path().join("miss.txt"), "nothing").unwrap();
            let mut have = files_with_matches_in("needle", Some(dir.path()), &[]).unwrap();
            have.sort();
            assert_eq!(have, vec!["hit.txt".into()]);
        }

        #[test]
        fn finds_no_matching_files() {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("miss.txt"), "nothing").unwrap();
            let have = files_with_matches_in("needle", Some(dir.path()), &[]).unwrap();
            assert!(have.is_empty());
        }

        #[test]
        fn finds_nested_matching_files() {
            #[cfg(not(windows))]
            let nested = "nested/hit.txt";
            #[cfg(windows)]
            let nested = "nested\\hit.txt";

            let dir = TempDir::new().unwrap();
            fs::create_dir_all(dir.path().join("nested")).unwrap();
            fs::write(dir.path().join(nested), "needle").unwrap();
            let have = files_with_matches_in("needle", Some(dir.path()), &[]).unwrap();
            assert_eq!(have, vec![nested.into()]);
        }

        #[test]
        fn ignores_files() {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("hit.txt"), "needle").unwrap();
            fs::write(dir.path().join("ignore.md"), "needle").unwrap();
            fs::write(dir.path().join("ignore.txt"), "needle").unwrap();
            let have =
                files_with_matches_in("needle", Some(dir.path()), &[S("*.md"), S("ignore.txt")])
                    .unwrap();
            assert_eq!(have, vec!["hit.txt".into()]);
        }
    }
}
