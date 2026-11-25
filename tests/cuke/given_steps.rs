use crate::filesystem::write_file;
use crate::jitter::jitter;
use crate::world::TridentWorld;
use cucumber::gherkin::Step;
use cucumber::given;
use test_helpers::Newlines;
use tokio::process::Command;

#[given(expr = "a committed file {string} with content")]
async fn a_committed_file_with_content(world: &mut TridentWorld, step: &Step, filename: String) {
    write_file(world, step, filename.clone(), Newlines::Platform).await;
    Command::new("git")
        .arg("add")
        .arg(&filename)
        .current_dir(&world.dir)
        .output()
        .await
        .unwrap();
    Command::new("git")
        .arg("-c")
        .arg("user.name=Test")
        .arg("-c")
        .arg("user.email=test@example.com")
        .arg("commit")
        .arg(format!("--message=Add {filename}"))
        .current_dir(&world.dir)
        .output()
        .await
        .unwrap();
}

#[given(expr = "a file {string} with content")]
async fn a_file_with_content(world: &mut TridentWorld, step: &Step, filename: String) {
    write_file(world, step, filename, Newlines::Platform).await;
}

#[given(expr = "a file {string} with unix newlines")]
async fn a_file_with_unix_newlines(world: &mut TridentWorld, step: &Step, filename: String) {
    write_file(world, step, filename, Newlines::Unix).await;
}

#[given(expr = "I change file {string} to")]
async fn i_change_file_to(world: &mut TridentWorld, step: &Step, filename: String) {
    let filepath = world.dir.join(&filename);
    assert!(
        filepath.exists(),
        "file '{}' does not exist",
        filepath.display()
    );
    write_file(world, step, filename, Newlines::Platform).await;
}

#[given(expr = "a Git repository")]
async fn a_git_repository(world: &mut TridentWorld) {
    jitter().await;
    Command::new("git")
        .arg("init")
        .arg("--initial-branch=main")
        .current_dir(&world.dir)
        .output()
        .await
        .unwrap();
    Command::new("git")
        .arg("-c")
        .arg("user.name=Test")
        .arg("-c")
        .arg("user.email=test@example.com")
        .arg("commit")
        .arg("--allow-empty")
        .arg("--message=initial")
        .current_dir(&world.dir)
        .output()
        .await
        .unwrap();
}

#[given(expr = "an executable file {string} with content")]
async fn an_executable_file_with_content(world: &mut TridentWorld, step: &Step, filename: String) {
    #[cfg(unix)]
    let filepath = world.dir.join(&filename);
    write_file(world, step, filename, Newlines::Unix).await;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        use tokio::fs;
        let mut perms = fs::metadata(&filepath).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&filepath, perms).await.unwrap();
    }
}

#[given(expr = "I ran {string}")]
async fn i_ran(world: &mut TridentWorld, command: String) {
    let mut args = command.split_ascii_whitespace();
    let executable = args.next().expect("executable is required");
    let mut absolute_path = if executable == "tools/rta" {
        world.cwd.join("tools").join("rta")
    } else {
        which::which(executable).unwrap()
    };
    if std::env::consts::OS == "windows" {
        absolute_path.set_extension("exe");
    }
    let mut cmd = Command::new(absolute_path);
    if executable == "git" {
        cmd.arg("-c")
            .arg("user.name=Test")
            .arg("-c")
            .arg("user.email=test@example.com");
    }
    cmd.args(args);
    cmd.current_dir(&world.dir);
    let output = cmd
        .output()
        .await
        .unwrap_or_else(|_| panic!("cannot find the '{executable}' executable"));
    assert!(
        output.status.success(),
        "command failed with {}\n\nOUTPUT:\n{}{}",
        output.status.code().unwrap(),
        str::from_utf8(&output.stdout).expect("non-UTF-8 output"),
        str::from_utf8(&output.stderr).expect("non-UTF-8 output"),
    );
}
