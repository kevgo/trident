use crate::world::TridentWorld;
use contains_lines::{contains_lines, contains_lines_matching};
use cucumber::gherkin::Step;
use cucumber::then;
use regex::Regex;
use std::path::Path;
use test_helpers::{docstring_body, snapshots, standardize_newlines};
use tokio::fs;
use tokio::process::Command;

#[then("all files are unchanged")]
async fn all_files_unchanged(world: &mut TridentWorld) {
    for original in &world.original_files {
        let filepath = world.dir.join(&original.name);
        let have = fs::read_to_string(filepath).await.unwrap_or_else(|err| {
            panic!(
                "cannot read file '{}', which should still exist: {}",
                original.name, err
            );
        });
        assert_eq!(
            normalize_file(&have).trim(),
            normalize_file(&original.content).trim(),
            "file '{}' was modified\n\nORIGINAL:\n{}\n\nNEW:\n{have}",
            original.name,
            original.content
        );
    }
}

#[then(expr = "file {string} does not exist")]
async fn file_does_not_exist(world: &mut TridentWorld, filename: String) {
    let filepath = world.dir.join(&filename);
    let exists = fs::try_exists(filepath).await.unwrap();
    assert!(!exists);
}

#[then(expr = "file {string} is unchanged")]
async fn file_is_unchanged(world: &mut TridentWorld, filename: String) {
    let original = world
        .original_files
        .iter()
        .find(|f| f.name == filename)
        .expect("file not found in original files");
    let filepath = world.dir.join(&original.name);
    let have = fs::read_to_string(filepath).await.unwrap_or_else(|err| {
        panic!(
            "cannot read file '{}', which should still exist: {}",
            original.name, err
        );
    });
    assert_eq!(
        normalize_file(&have).trim(),
        normalize_file(&original.content).trim(),
        "file '{}' was modified\n\nORIGINAL:\n{}\n\nNEW:\n{have}",
        original.name,
        original.content
    );
}

#[then(expr = "file {string} now has an additional line matching")]
async fn file_has_additional_line_matching(
    world: &mut TridentWorld,
    step: &Step,
    filename: String,
) {
    let want = docstring_body(step.docstring.as_ref().unwrap());
    let have_old = world.original_file_content(&filename).unwrap_or_default();
    let have_new = world.current_file_content(&filename).await.unwrap();
    if let Err(problem) = test_helpers::has_additional_lines(have_old, &have_new, &want) {
        panic!("file '{filename}' {problem}\n\nHAVE:\n{have_new}\n\n");
    }
}

#[then(expr = "file {string} now has content")]
async fn file_has_content(world: &mut TridentWorld, step: &Step, filename: String) {
    let want = docstring_body(step.docstring.as_ref().unwrap());
    let filepath = world.dir.join(&filename);
    let have = standardize_newlines(&fs::read_to_string(filepath).await.unwrap());
    let want = standardize_newlines(docstring_body(&want));
    pretty::assert_eq!(have, want, "\n\nHAVE:\n{have}\n\nWANT:\n{want}\n\n");
}

#[then(expr = "file {string} now matches these lines")]
async fn file_matches_lines(world: &mut TridentWorld, step: &Step, filename: String) {
    let want = normalize_file(docstring_body(step.docstring.as_ref().unwrap()));
    let want = want.lines();
    let filepath = world.dir.join(&filename);
    let have = normalize_file(&fs::read_to_string(filepath).await.unwrap());
    for (want_line, have_line) in want.trim().lines().zip(have.trim().lines()) {
        if want_line.contains(".*") {
            let regex = Regex::new(want_line).unwrap();
            assert!(
                regex.is_match(have_line),
                "\n\nHAVE:\n{have_line}\n\nWANT:\n{want_line}\n\n"
            );
        } else {
            assert_eq!(
                have_line, want_line,
                "\n\nHAVE:\n{have_line}\n\nWANT:\n{want_line}\n\n"
            );
        }
    }
}

#[then(expr = "file {string} is executable")]
async fn file_is_executable(world: &mut TridentWorld, filename: String) {
    let filepath = world.dir.join(&filename);
    let metadata = fs::metadata(&filepath).await.unwrap();
    assert_executable(&metadata, &filename);
}

#[cfg(unix)]
fn assert_executable(metadata: &std::fs::Metadata, filename: &str) {
    use std::os::unix::fs::PermissionsExt;
    let is_executable = metadata.permissions().mode() & 0o111 != 0;
    assert!(is_executable, "file '{filename}' is not executable");
}

/// Windows has no Unix execute bits; Git hooks and shell scripts still work without them.
#[cfg(not(unix))]
fn assert_executable(metadata: &std::fs::Metadata, filename: &str) {
    assert!(metadata.is_file(), "file '{filename}' is not a file");
}

#[then("it does not print")]
fn it_does_not_print(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no output");
    let stdout = normalize_output(output.stdout.as_slice());
    assert!(
        !stdout.contains(want),
        "output should not contain '{want}'\n\nHAVE:\n{stdout}",
    );
    let stderr = normalize_output(output.stderr.as_slice());
    assert!(
        !stderr.contains(want),
        "output should not contain '{want}'\n\nHAVE:\n{stderr}",
    );
}

#[then("it does not print any of these lines")]
fn it_does_not_print_the_lines(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no command run");
    let stdout = normalize_output(output.stdout.as_slice());
    for want_line in want.lines() {
        assert!(!stdout.contains(want_line), "STDOUT contains '{want_line}'");
    }
}

#[then("it prints")]
fn it_prints(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no command run");
    let stdout = normalize_output(&strip_ansi_escapes::strip(&output.stdout));
    pretty::assert_eq!(stdout.trim(), normalize_file(want));
}

#[then("it prints nothing to STDOUT")]
fn it_prints_nothing_to_stdout(world: &mut TridentWorld) {
    let output = world.output.as_ref().expect("no command run");
    let stripped = strip_ansi_escapes::strip(&output.stdout);
    let stdout = str::from_utf8(&stripped).expect("non-UTF-8 output");
    pretty::assert_eq!(stdout, "");
}

#[then("it prints nothing to STDERR")]
fn it_prints_nothing_to_stderr(world: &mut TridentWorld) {
    let output = world.output.as_ref().expect("no command run");
    let stripped = strip_ansi_escapes::strip(&output.stderr);
    let stderr = str::from_utf8(&stripped).expect("non-UTF-8 output");
    pretty::assert_eq!(stderr, "");
}

#[then("it prints to STDERR")]
fn it_prints_to_stderr(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no command run");
    let stderr = normalize_output(&strip_ansi_escapes::strip(&output.stderr));
    pretty::assert_eq!(stderr.trim(), normalize_file(want));
}

#[then("it prints the block")]
fn it_prints_the_block(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no command run");
    let stdout = normalize_output(&strip_ansi_escapes::strip(&output.stdout));
    assert!(
        stdout.contains(want),
        "output does not contain the block\n\nHAVE:\n{stdout}\n\n"
    );
}

#[then("it prints the block matching")]
fn it_prints_the_block_matching(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no command run");
    let stdout = normalize_output(&strip_ansi_escapes::strip(&output.stdout));
    assert!(
        Regex::new(want).unwrap().is_match(&stdout),
        "output does not match the block\n\nHAVE:\n{stdout}\n\nWANT (regex):\n{want}\n\n"
    );
}

#[then("it prints the lines")]
fn it_prints_the_lines(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no command run");
    let stdout = normalize_output(&strip_ansi_escapes::strip(&output.stdout));
    if snapshots::enabled() {
        if stdout != want {
            let path = world
                .feature_path
                .clone()
                .expect("the feature file path is unknown, is the `before` hook wired up?");
            snapshots::queue_update(snapshots::SnapshotEdit {
                path,
                step_line: step.position.line,
                new_content: stdout.to_string(),
            });
        }
        return;
    }
    let missing = contains_lines(&stdout, want);
    assert!(
        missing.is_empty(),
        "STDOUT is missing lines:\n\nHAVE:\n{stdout}\n\nWANT:\n{want}\n\nMISSING:\n{}",
        missing.join("\n")
    );
}

#[then("it prints the lines matching")]
fn it_prints_the_lines_matching(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no command run");
    let stripped = strip_ansi_escapes::strip(&output.stdout);
    let stdout = str::from_utf8(&stripped).expect("non-UTF-8 output");
    let missing = contains_lines_matching(stdout, want).unwrap();
    assert!(
        missing.is_empty(),
        "STDOUT is missing lines:\n\nHAVE:\n{stdout}\n\nWANT:\n{want}\n\nMISSING:\n{}",
        missing.join("\n")
    );
}

#[then("it prints the lines to STDERR")]
fn it_prints_the_lines_to_stderr(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let output = world.output.as_ref().expect("no command run");
    let stderr = normalize_output(&strip_ansi_escapes::strip(&output.stderr));
    let missing = contains_lines(&stderr, want);
    assert!(
        missing.is_empty(),
        "STDERR is missing lines:\n\nHAVE:\n{stderr}\n\nWANT:\n{want}\n\nMISSING:\n{}",
        missing.join("\n")
    );
}

#[then("it prints only these lines in any order")]
fn prints_lines_any_order(world: &mut TridentWorld, step: &Step) {
    let want_text = normalize_file(docstring_body(step.docstring.as_ref().unwrap()));
    let mut want = want_text.lines().collect::<Vec<&str>>();
    let output = world.output.as_ref().expect("no command run");
    let stdout = normalize_output(&strip_ansi_escapes::strip(&output.stdout));
    let mut have = stdout.lines().collect::<Vec<&str>>();
    let compare_result = test_helpers::compare_lines_any_order(&mut have, &mut want);
    assert!(
        compare_result.success(),
        "{}\nHAVE:\n{stdout}",
        compare_result.message()
    );
}

#[then(expr = "the exit code is {int}")]
fn exit_code(world: &mut TridentWorld, want: i32) {
    let have = world.exit_code();
    // golangci-lint reports typecheck failures as exit code 7 on Windows
    // and as exit code 1 on Unix. Both mean the lint failed.
    if cfg!(windows) && want == 1 && have == 7 {
        return;
    }
    assert_eq!(have, want);
}

#[then(expr = "the staged changes are")]
async fn the_staged_changes_are(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let have = staged_changes(&world.dir).await;
    assert_eq!(have.trim(), want.trim());
}

#[then(expr = "the unstaged changes are")]
async fn the_unstaged_changes_are(world: &mut TridentWorld, step: &Step) {
    let want = step.docstring.as_ref().unwrap().trim();
    let have = unstaged_changes(&world.dir).await;
    assert_eq!(have.trim(), want.trim());
}

#[then(expr = "there are no staged changes")]
async fn there_are_no_staged_changes(world: &mut TridentWorld) {
    let want = "";
    let have = staged_changes(&world.dir).await;
    assert_eq!(have.trim(), want.trim());
}

#[then(expr = "there are no unstaged changes")]
async fn there_are_no_unstaged_changes(world: &mut TridentWorld) {
    let want = "";
    let have = unstaged_changes(&world.dir).await;
    assert_eq!(have.trim(), want.trim());
}

async fn staged_changes(dir: &Path) -> String {
    let output = Command::new("git")
        .arg("diff")
        .arg("--staged")
        .current_dir(dir)
        .output()
        .await
        .unwrap();
    let stdout = strip_ansi_escapes::strip(&output.stdout);
    normalize_output(
        String::from_utf8_lossy(&stdout)
            .replace("\n \n", "\n\n")
            .replace("\n\n", "\n")
            .as_bytes(),
    )
}

async fn unstaged_changes(dir: &Path) -> String {
    let output = Command::new("git")
        .arg("diff")
        .current_dir(dir)
        .output()
        .await
        .unwrap();
    let stdout = strip_ansi_escapes::strip(&output.stdout);
    normalize_output(
        String::from_utf8_lossy(&stdout)
            .replace("\n \n", "\n\n")
            .replace("\n\n", "\n")
            .as_bytes(),
    )
}

fn docstring_body(content: &str) -> &str {
    let content = content.strip_prefix('\r').unwrap_or(content);
    content.strip_prefix('\n').unwrap_or(content)
}

fn normalize_file(text: &str) -> String {
    text.replace('\r', "")
}

/// Makes command output comparable with the Unix-oriented feature files.
fn normalize_output(text: &[u8]) -> String {
    let text = String::from_utf8_lossy(text).replace('\r', "");
    #[cfg(windows)]
    let text = text
        .replace('\\', "/")
        .replace(".exe", "")
        .lines()
        .filter(|line| !line.starts_with("Installed ") || !line.contains(" packages in "))
        .collect::<Vec<_>>()
        .join("\n");
    text
}
