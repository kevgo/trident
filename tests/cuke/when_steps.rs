use crate::git_commands;
use crate::jitter::jitter;
use crate::world::TridentWorld;
use cucumber::when;
use std::time::Duration;
use tokio::process::Command;

#[when(expr = "inspect the workspace")]
async fn inspect_workspace(world: &mut TridentWorld) {
    // print visibly to the user even though this runs inside Cucumber
    // repeating a few times to break out of the cucumber formatter that deletes the current line
    println!("workspace: {}", world.dir.display());
    println!("workspace: {}", world.dir.display());
    println!("workspace: {}", world.dir.display());
    // pause for 1 minute
    tokio::time::sleep(Duration::from_hours(1)).await;
}

#[when(expr = "executing {string}")]
async fn executing(world: &mut TridentWorld, command: String) {
    jitter().await;
    let mut args = command.split_ascii_whitespace();
    let executable = args.next().expect("executable is required");
    assert!(executable == "trident", "can only execute 'trident'");
    let mut absolute_path = world.cwd.join("target/release/trident");
    if std::env::consts::OS == "windows" {
        absolute_path.set_extension("exe");
    }
    let mut cmd = Command::new(absolute_path);
    cmd.args(args);
    cmd.current_dir(&world.dir);
    match tokio::fs::remove_file(&world.git_log).await {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => panic!("cannot delete the Git log: {err}"),
    }
    git_commands::configure(&mut cmd, &world.git_log);
    let output = cmd
        .output()
        .await
        .unwrap_or_else(|_| panic!("cannot find the '{executable}' executable"));
    world.output = Some(output);
    world.git_commands = Some(git_commands::logged(&world.git_log));
}
