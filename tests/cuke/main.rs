#![allow(clippy::needless_pass_by_value)]

mod dot_writer;
mod given_steps;
mod jitter;
mod run_that_app_file;
mod then_steps;
mod when_steps;
mod world;

use cucumber::tag::Ext as _;
use cucumber::{World, WriterExt as _, cli, event, gherkin, runner};
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
    let mut cli = cli::Opts::<_, runner::basic::Cli, _>::parsed();
    // Cucumber ignores custom filters when the CLI provides a tag or name filter,
    // so we take them over and apply them together with the platform filter.
    let tags_filter = cli.tags_filter.take();
    let name_filter = cli.re_filter.take();
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
        .filter_run("features", move |feature, rule, scenario| {
            let tags = scenario_tags(feature, rule, scenario);
            matches_platform(tags.clone())
                && name_filter
                    .as_ref()
                    .is_none_or(|re| re.is_match(&scenario.name))
                && tags_filter.as_ref().is_none_or(|filter| filter.eval(tags))
        })
        .await;
    if snapshots::enabled() {
        snapshots::flush();
    }
    if had_failures.load(Ordering::SeqCst) {
        std::process::exit(1);
    }
}

/// provides all tags that apply to the given scenario, including the ones of its feature and rule
fn scenario_tags<'a>(
    feature: &'a gherkin::Feature,
    rule: Option<&'a gherkin::Rule>,
    scenario: &'a gherkin::Scenario,
) -> impl Iterator<Item = &'a String> + Clone {
    feature
        .tags
        .iter()
        .chain(rule.into_iter().flat_map(|rule| &rule.tags))
        .chain(scenario.tags.iter())
}

/// indicates whether a scenario with the given tags should run on the current platform:
/// scenarios tagged `@windows` run only on Windows, scenarios tagged `@unix` only elsewhere
fn matches_platform<'a>(mut tags: impl Iterator<Item = &'a String>) -> bool {
    tags.all(|tag| match tag.as_str() {
        "windows" => cfg!(windows),
        "unix" => !cfg!(windows),
        _ => true,
    })
}
