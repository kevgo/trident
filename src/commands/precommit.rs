use crate::apps::delete_empty_folders::DeleteEmptyFolders;
use crate::apps::{delete_empty_folders, keep_sorted};
use crate::cli::input::{RunArgs, ShowExt};
use crate::cli::output::print_metadata;
use crate::commands::fix::add_custom_fixes;
use crate::commands::run_sequences;
use crate::config::{Application, Config, Operation};
use crate::domain::{DetectedStacks, Result, Runnables, StackType, UserError, fingerprint};
use crate::git;
use crate::stacks;
use ahash::AHashMap;
use std::process::ExitCode;

pub fn precommit(args: &RunArgs) -> Result<ExitCode> {
    if let Err(err) = run(args) {
        err.print();
    }
    Ok(ExitCode::SUCCESS)
}

fn run(args: &RunArgs) -> Result<()> {
    // step 1: load the config
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let repo = git::Repo::load().ok_or(UserError::NoGitRepository)?;
    let show = args.show.unwrap_or(conc::Show::Names);
    let sequential = args.sequential;

    // step 2: discover the staged files and their stacks
    let staged = repo.staged()?;
    if staged.is_empty() {
        return Ok(());
    }
    let stacks = stacks::from_staged(&staged, &ignores);
    if show.display_metadata() {
        print_metadata(&stacks);
    }

    // step 3: fingerprint the staged files before running the fixes
    let staged_files = staged.all();
    let before = fingerprint::scan_files(&staged_files);

    // step 4: discover the fixes to run
    let fixes = determine_precommit_fixes(&config, &stacks)?;
    if show.display_metadata() {
        eprintln!("running {} tools", fixes.len());
    }
    let Runnables {
        global,
        stack_specific,
    } = fixes;

    // step 5: run the global fixes
    if let Some(global) = global {
        let _exit_code = run_sequences(vec![global], show, sequential);
    }

    // step 6: run the stack-specific fixes
    let _exit_code = run_sequences(stack_specific.into_values().collect(), show, sequential);

    // step 7: stage the files whose fixes actually changed their content
    let after = fingerprint::scan_files(&staged_files);
    let changed = fingerprint::changed(&before, &after);
    repo.stage_files(&changed)?;
    Ok(())
}

/// determines the fixes to run in the precommit command
/// based on the given staged stacks
pub fn determine_precommit_fixes(
    config: &Config,
    staged_stacks: &DetectedStacks,
) -> Result<Runnables> {
    // global fixes
    let mut global = Vec::new();
    if config.operation_enabled(&DeleteEmptyFolders {}, Operation::Fix)
        && let Some(delete_empty_folders) = delete_empty_folders::format_command()?
    {
        global.push(delete_empty_folders);
    }

    // stack-specific fixes
    let mut stacks_executables: AHashMap<StackType, Vec<conc::Executable>> = AHashMap::new();
    for staged_stack in staged_stacks {
        let stack_type = staged_stack.stack.stack_type();
        let stack_config = config.stack_config(stack_type);
        let stack_executables = stacks_executables.entry(stack_type).or_default();
        let stack_fixes = stack_config.and_then(|stack_config| stack_config.fix.as_ref());
        if let Some(override_fixes) = stack_fixes.and_then(|fix| fix.replace.as_ref()) {
            stack_executables.extend(
                override_fixes
                    .iter()
                    .map(|tool| tool.to_executable(Operation::Fix, stack_type)),
            );
        } else {
            for default_fix in staged_stack.stack.fixes() {
                if config.operation_enabled(default_fix.as_ref(), Operation::Fix)
                    && default_fix.enabled_when().enabled_on_disk()
                {
                    stack_executables.extend(default_fix.fix_commands(staged_stack, config)?);
                }
            }
        }
        if let Some(additions) = stack_fixes.and_then(|fix| fix.add.as_ref()) {
            stack_executables.extend(
                additions
                    .iter()
                    .map(|tool| tool.to_executable(Operation::Fix, stack_type)),
            );
        }
    }

    // custom fixes
    if let Some(custom_fixes) = &config.global_fixes {
        add_custom_fixes(custom_fixes, &mut global);
    }

    // keep-sorted
    if let Some(keep_sorted_config) = config.keep_sorted() {
        let fix_op = keep_sorted_config.operation(Operation::Fix);
        if keep_sorted_config.enabled() && fix_op.is_none_or(Application::enabled) {
            let mut keep_sorted_ignores = keep_sorted_config.ignore_files().to_vec();
            if let Some(op_config) = fix_op {
                keep_sorted_ignores.extend_from_slice(op_config.ignore_files());
            }
            let sort_result = keep_sorted::fix_commands(keep_sorted::FixCommandsArgs {
                detected_stacks: staged_stacks,
                global_ignores: config.ignore_files.as_ref(),
                keep_sorted_ignores: &keep_sorted_ignores,
            })?;
            for (stack_type, executable) in sort_result {
                stacks_executables
                    .entry(stack_type)
                    .or_default()
                    .push(executable);
            }
        }
    }

    // step 6: convert to runnables and return
    let mut stack_specific: AHashMap<StackType, conc::Sequence> = AHashMap::new();
    for (stack_type, stack_executables) in stacks_executables {
        if let Some(stack_sequences) = conc::Sequence::from_vec(stack_executables) {
            stack_specific.insert(stack_type, stack_sequences);
        }
    }
    let global = if global.is_empty() {
        None
    } else {
        conc::Sequence::from_vec(global)
    };
    Ok(Runnables {
        global,
        stack_specific,
    })
}
