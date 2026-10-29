use super::{discover_stacks, resolve_scope};
use crate::cli::input::{RunArgsWithScope, ShowExt};
use crate::cli::output::print_metadata;
use crate::config::Config;
use crate::domain::{DetectedStacks, Operation, Result, StackType};
use crate::git::Repo;
use ahash::AHashMap;
use std::process::ExitCode;

pub fn fix_unsafe(args: &RunArgsWithScope) -> Result<ExitCode> {
    // step 1: load the config
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let error_on_output = false;
    let sequential = args.run.sequential;
    let stderr_to_stdout = true;
    let show = args.run.show.unwrap_or(conc::Show::Names);

    // step 2: discover the stacks
    let repo = Repo::load();
    let scope = resolve_scope(&args.scope, repo.as_ref());
    let stacks = discover_stacks(scope, repo.as_ref(), &ignores)?;
    if show.display_metadata() {
        print_metadata(&stacks);
    }

    // step 3: discover the unsafe fixes to run
    let unsafe_fixes = determine_unsafe_fixes(&stacks, &config)?;
    if show.display_metadata() {
        eprintln!("running {} tools", unsafe_fixes.len());
    }

    // step 4: run the fixes
    let exit_code = conc::run(conc::RunArgs {
        sequences: unsafe_fixes,
        error_on_output,
        sequential,
        show,
        stderr_to_stdout,
    });
    Ok(exit_code)
}

pub fn determine_unsafe_fixes(
    stacks: &DetectedStacks,
    config: &Config,
) -> Result<Vec<conc::Sequence>> {
    let mut stacks_executables: AHashMap<StackType, Vec<conc::Executable>> = AHashMap::new();
    for stack in stacks {
        let stack_executables = stacks_executables
            .entry(stack.stack.stack_type())
            .or_default();
        for fix in stack.stack.fixes() {
            if config.operation_enabled(fix.as_ref(), Operation::FixUnsafe)
                && stacks.stack_enabled(&fix.enabled_when())
            {
                stack_executables.extend(fix.unsafe_fix_commands(stack, config)?);
            }
        }
    }
    let mut result = Vec::new();
    for (_stack_type, mut stack_executables) in stacks_executables {
        if !stack_executables.is_empty() {
            let first = stack_executables.remove(0);
            result.push(conc::Sequence::many(first, stack_executables));
        }
    }
    Ok(result)
}
