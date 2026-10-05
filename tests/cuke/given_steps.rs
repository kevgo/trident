use crate::jitter::jitter;
use crate::world::{ExistingFile, TridentWorld};
use cucumber::gherkin::Step;
use cucumber::given;
use test_helpers::docstring_body;
use tokio::fs;
use tokio::process::Command;

#[given(expr = "a committed file {string} with content")]
async fn a_committed_file_with_content(world: &mut TridentWorld, step: &Step, filename: String) {
    a_file_with_content(world, step, filename.clone()).await;
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
    let content = docstring_body(step.docstring.as_ref().unwrap());
    let filepath = world.dir.join(&filename);
    let parent = filepath.parent().unwrap();
    if parent != world.dir {
        fs::create_dir_all(parent)
            .await
            .unwrap_or_else(|_| panic!("cannot create parent '{}'", parent.display()));
    }
    fs::write(&filepath, content.as_bytes())
        .await
        .unwrap_or_else(|_| panic!("cannot write to file '{}'", filepath.display()));
    world.original_files.push(ExistingFile {
        name: filename,
        content,
    });
}

#[given(expr = "I change file {string} to")]
async fn i_change_file_to(world: &mut TridentWorld, step: &Step, filename: String) {
    let filepath = world.dir.join(&filename);
    assert!(
        filepath.exists(),
        "file '{}' does not exist",
        filepath.display()
    );
    a_file_with_content(world, step, filename).await;
}

#[given(expr = "a Git repository")]
async fn a_git_repository(world: &mut TridentWorld) {
    jitter().await;
    Command::new("git")
        .arg("init")
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
    a_file_with_content(world, step, filename).await;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
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
    #[cfg(windows)]
    absolute_path.set_extension("exe");
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
