#![allow(clippy::needless_pass_by_value)]

mod dot_writer;
mod given_steps;
mod jitter;
mod run_that_app_file;
mod then_steps;
mod when_steps;
mod world;

use cucumber::{World, WriterExt as _, cli, event, runner};
use dot_writer::DotWriter;
use std::borrow::Cow;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use test_helpers::snapshots;
use world::TridentWorld;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let had_failures = Arc::new(AtomicBool::new(false));
    let cli = cli::Opts::<_, runner::basic::Cli, _>::parsed();
    if cli.runner.concurrency == Some(1) {
        // no concurrent scenarios, hence no need to spread out file system activity
        jitter::disable();
    }
    TridentWorld::cucumber()
        .before(|feature, _rule, _scenario, world| {
            world.feature_path.clone_from(&feature.path);
            Box::pin(async {})
        })
        .after(|feature, _rule, scenario, ev, world| {
            Box::pin(async move {
                if !matches!(ev, event::ScenarioFinished::StepPassed) {
                    // the scenario already reports a failure
                    return;
                }
                let Some(world) = world else { return };
                if run_that_app_file::is_asserted_by(scenario) {
                    return;
                }
                if let Err(err) = run_that_app_file::verify_unchanged(world).await {
                    let path = match feature.path.as_ref() {
                        Some(path) => path.to_string_lossy(),
                        None => Cow::Borrowed("(unknown)"),
                    };
                    panic!(
                        "{path}:{line}  Scenario unexpectedly changed file run-that-app: {err}",
                        line = scenario.position.line,
                    );
                }
            })
        })
        // .max_concurrent_scenarios(1)
        .with_writer(DotWriter::new(Arc::clone(&had_failures)).normalized())
        .with_cli(cli)
        .run("features")
        .await;
    if snapshots::enabled() {
        snapshots::flush();
    }
    if had_failures.load(Ordering::SeqCst) {
        std::process::exit(1);
    }
}
