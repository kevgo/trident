use super::{discover_stacks, resolve_scope};
use crate::apps::delete_empty_folders;
use crate::apps::delete_empty_folders::DeleteEmptyFolders;
use crate::apps::keep_sorted;
use crate::cli::input::{RunArgsWithScope, ShowExt};
use crate::cli::output::print_metadata;
use crate::config::{Application, Config, Operation, ToolDefinition};
use crate::domain::{DetectedStacks, Result, Runnables, StackType};
use crate::git::Repo;
use ahash::AHashMap;
use std::process::ExitCode;

pub fn fix(args: &RunArgsWithScope) -> Result<ExitCode> {
    // step 1: load the config
    let config = Config::load()?;
    let ignores = config.ignores()?;
    let show = args.run.show.unwrap_or(conc::Show::Names);
    let error_on_output = false;
    let stderr_to_stdout = true;

    // step 2: discover the stacks
    let repo = Repo::load();
    let scope = resolve_scope(&args.scope, repo.as_ref());
    let stacks = discover_stacks(scope, repo.as_ref(), &ignores)?;
    if show.display_metadata() {
        print_metadata(&stacks);
    }

    // step 3: discover the fixes to run
    let fixes = determine_fixes(&config, &stacks)?;
    if show.display_metadata() {
        eprintln!("running {} tools", fixes.len());
    }
    let Runnables {
        global,
        stack_specific,
    } = fixes;

    // step 4: run the global fixes
    if let Some(global) = global {
        let exit_code = conc::run(conc::RunArgs {
            sequences: vec![global],
            error_on_output,
            stderr_to_stdout,
            show,
        });
        if exit_code != ExitCode::SUCCESS {
            return Ok(exit_code);
        }
    }

    // step 5: run the stack-specific fixes
    let exit_code = conc::run(conc::RunArgs {
        sequences: stack_specific.into_values().collect(),
        error_on_output,
        show,
        stderr_to_stdout,
    });
    Ok(exit_code)
}

pub fn determine_fixes(config: &Config, detected_stacks: &DetectedStacks) -> Result<Runnables> {
    // global fixes
    let mut global = Vec::new();
    if config.operation_enabled(&DeleteEmptyFolders {}, Operation::Fix)
        && let Some(delete_empty_folders) = delete_empty_folders::format_command()?
    {
        global.push(delete_empty_folders);
    }

    // stack-specific fixes
    let mut stacks_executables: AHashMap<StackType, Vec<conc::Executable>> = AHashMap::new();
    for detected_stack in detected_stacks {
        let stack_type = detected_stack.stack.stack_type();
        let stack_config = config.stack_config(stack_type);
        let stack_executables = stacks_executables.entry(stack_type).or_default();
        // schedule either the override fixes or the default fixes
        let stack_fixes = stack_config.and_then(|stack_config| stack_config.fix.as_ref());
        if let Some(overrides) = stack_fixes.and_then(|sf| sf.replace.as_ref()) {
            stack_executables.extend(
                overrides
                    .iter()
                    .map(|tool| tool.to_executable(Operation::Fix, stack_type)),
            );
        } else {
            for default_fix in detected_stack.stack.fixes() {
                if config.operation_enabled(default_fix.as_ref(), Operation::Fix)
                    && default_fix.enabled_when().enabled_on_disk()
                {
                    stack_executables.extend(default_fix.fix_commands(detected_stack, config)?);
                }
            }
        }
        // schedule the additional fixes
        if let Some(additions) = stack_fixes.and_then(|stack_fixes| stack_fixes.add.as_ref()) {
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
                detected_stacks,
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

    // convert to runnables and return
    let mut stack_specific = AHashMap::new();
    for (stack_type, stack_executables) in stacks_executables {
        if let Some(stack_sequence) = conc::Sequence::from_vec(stack_executables) {
            stack_specific.insert(stack_type, stack_sequence);
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

/// adds the custom fixes defined in the config file to the global fix collection
pub(crate) fn add_custom_fixes(
    custom_fixes: &[ToolDefinition],
    global: &mut Vec<conc::Executable>,
) {
    for fix in custom_fixes {
        global.push(conc::Executable {
            name: fix.name.clone().unwrap_or_else(|| fix.command.clone()),
            command: conc::shell_command(&fix.command),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::add_custom_fixes;
    use crate::config::ToolDefinition;
    use big_s::S;

    fn executable_names(executables: &[conc::Executable]) -> Vec<&str> {
        executables
            .iter()
            .map(|executable| executable.name.as_str())
            .collect()
    }

    #[test]
    fn custom_fixes_go_global() {
        let mut global = Vec::new();
        add_custom_fixes(
            &[
                ToolDefinition {
                    name: Some(S("global fix")),
                    command: S("echo global"),
                },
                ToolDefinition {
                    name: None,
                    command: S("echo unnamed"),
                },
            ],
            &mut global,
        );
        pretty::assert_eq!(
            executable_names(&global),
            vec!["global fix", "echo unnamed"]
        );
    }
}
