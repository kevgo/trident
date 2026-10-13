use super::{discover_stacks, lint, run_sequences};
use crate::cli::input::{RunArgs, Scope, ShowExt};
use crate::cli::output::print_metadata;
use crate::config::Config;
use crate::domain::Result;
use crate::git;
use std::process::ExitCode;

pub fn post_edit(args: &RunArgs) -> Result<ExitCode> {
    // step 1: load the config
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let git_repo = git::Repo::load();
    let show = args.show.unwrap_or(conc::Show::Failed);

    // step 2: discover the files and their stacks
    let stacks = discover_stacks(Scope::Uncommitted, git_repo.as_ref(), &ignores)?;
    if show.display_metadata() {
        print_metadata(&stacks);
    }

    // step 3: discover the lints to run
    let lints = lint::determine_lints(&config, &stacks, git_repo.as_ref())?;
    if show.display_metadata() {
        eprintln!("running {} tools", lints.len());
    }

    // step 4: run the lints
    if lints.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }
    let exit_code = run_sequences(lints.into_sequences(), show, args.sequential);
    Ok(exit_code)
}
