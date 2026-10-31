use crate::apps::{GetRTACmdArgs2, get_rta_command2, ripgrep};
use crate::domain::{DetectedStacks, Result, StackType};
use crate::domain::{File, Operation};
use crate::subshell::ToolCall;
use ahash::AHashMap;
use big_s::S;

const MARKER: &str = "keep-sorted end";

/// provides one `Executable` per `StackType`
/// that sorts detected files of that stack
/// that contain a "keep-sorted end" marker.
pub fn fix_commands(
    FixCommandsArgs {
        detected_stacks,
        global_ignores,
        keep_sorted_ignores,
    }: FixCommandsArgs,
) -> Result<Vec<(StackType, ToolCall)>> {
    // step 2: find all files that contain the "keep-sorted end" marker
    let mut ignores = vec![];
    if let Some(global_ignores) = global_ignores {
        ignores.extend(global_ignores.iter().cloned());
    }
    ignores.extend_from_slice(keep_sorted_ignores);
    let matches = ripgrep::files_with_matches(MARKER, &ignores)?;
    if matches.is_empty() {
        return Ok(vec![]);
    }

    // step 3: keep only marker-bearing files that are in the detected stacks
    let mut grouped: AHashMap<StackType, Vec<File>> = AHashMap::new();
    for found in matches {
        if let Some(stack_type) = detected_stacks.stack_type_for_file(&found) {
            grouped.entry(stack_type).or_default().push(found);
        }
    }

    // step 4: create the executables that run keep-sorted for each stack
    let mut result = Vec::with_capacity(grouped.len());
    for (stack_type, mut files) in grouped {
        files.sort_unstable();
        let args = files.into_iter().map(Into::into).collect();
        let command = get_rta_command2(&GetRTACmdArgs2 {
            app: &rta::applications::KeepSorted {},
            args,
        })?;
        if let Some(command) = command {
            let tool_call = ToolCall {
                operation: Operation::Fix,
                stack: Some(stack_type),
                name: S("keep-sorted"),
                command,
            };
            result.push((stack_type, tool_call));
        }
    }
    Ok(result)
}

pub struct FixCommandsArgs<'a> {
    pub detected_stacks: &'a DetectedStacks,
    pub global_ignores: Option<&'a Vec<String>>,
    pub keep_sorted_ignores: &'a [String],
}
