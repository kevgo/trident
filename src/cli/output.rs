use crate::domain::{DetectedStacks, StackType};
use crate::subshell::ToolCall;
use ahash::AHashSet;
use itertools::Itertools;

/// prints how many files of which stack get processed by the given tool calls
///
/// Only stacks that have at least one stack-specific tool call scheduled are listed.
pub fn print_metadata<'a>(
    stacks: &DetectedStacks,
    tool_calls: impl IntoIterator<Item = &'a ToolCall>,
) {
    if let Some(text) = metadata_text(stacks, tool_calls) {
        eprintln!("{text}");
    }
}

fn metadata_text<'a>(
    stacks: &DetectedStacks,
    tool_calls: impl IntoIterator<Item = &'a ToolCall>,
) -> Option<String> {
    let processed_stacks: AHashSet<StackType> = tool_calls
        .into_iter()
        .filter_map(|tool_call| tool_call.stack)
        .collect();
    let files_texts: Vec<String> = stacks
        .into_iter()
        .filter(|stack| processed_stacks.contains(&stack.stack.stack_type()))
        .map(|stack| format!("{} {}", stack.files.len(), stack.stack))
        .collect();
    if files_texts.is_empty() {
        return None;
    }
    Some(files_texts.iter().join(", "))
}

#[cfg(test)]
mod tests {
    use super::metadata_text;
    use crate::domain::{DetectedStack, DetectedStacks, Files, Operation, StackType};
    use crate::stacks::{Markdown, Rust, Unknown};
    use crate::subshell::ToolCall;
    use std::path::PathBuf;

    fn detected_stacks() -> DetectedStacks {
        DetectedStacks::new(vec![
            DetectedStack {
                stack: Box::new(Markdown {}),
                files: Files::from(vec![PathBuf::from("README.md")]),
            },
            DetectedStack {
                stack: Box::new(Rust {}),
                files: Files::from(vec![PathBuf::from("main.rs"), PathBuf::from("lib.rs")]),
            },
            DetectedStack {
                stack: Box::new(Unknown {}),
                files: Files::from(vec![PathBuf::from("LICENSE")]),
            },
        ])
    }

    fn tool_call(stack: Option<StackType>) -> ToolCall {
        ToolCall {
            operation: Operation::Lint,
            stack,
            name: "true".to_string(),
            command: conc::shell_command("true"),
        }
    }

    #[test]
    fn multiple_processed_stacks() {
        let tool_calls = [
            tool_call(Some(StackType::Rust)),
            tool_call(Some(StackType::Markdown)),
        ];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, Some("1 Markdown, 2 Rust".to_string()));
    }

    #[test]
    fn single_processed_stack() {
        let tool_calls = [
            tool_call(Some(StackType::Rust)),
            tool_call(Some(StackType::Rust)),
            tool_call(None),
        ];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, Some("2 Rust".to_string()));
    }

    #[test]
    fn no_stack_specific_tool_calls() {
        let tool_calls = [tool_call(None)];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, None);
    }

    #[test]
    fn no_tool_calls() {
        let tool_calls: [ToolCall; 0] = [];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, None);
    }

    #[test]
    fn multiple_processed_stacks_including_unknown() {
        let tool_calls = [
            tool_call(Some(StackType::Markdown)),
            tool_call(Some(StackType::Rust)),
            tool_call(Some(StackType::Unknown)),
        ];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, Some("1 Markdown, 2 Rust, 1 other".to_string()));
    }

    #[test]
    fn sorts_stacks_alphabetically() {
        let tool_calls = [
            tool_call(Some(StackType::Unknown)),
            tool_call(Some(StackType::Rust)),
            tool_call(Some(StackType::Markdown)),
        ];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, Some("1 Markdown, 2 Rust, 1 other".to_string()));
    }

    #[test]
    fn multiple_tool_calls_per_stack_count_files_once() {
        let tool_calls = [
            tool_call(Some(StackType::Markdown)),
            tool_call(Some(StackType::Markdown)),
            tool_call(Some(StackType::Markdown)),
        ];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, Some("1 Markdown".to_string()));
    }

    #[test]
    fn mixed_global_and_stack_specific_tool_calls() {
        let tool_calls = [
            tool_call(None),
            tool_call(Some(StackType::Rust)),
            tool_call(None),
            tool_call(Some(StackType::Markdown)),
        ];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, Some("1 Markdown, 2 Rust".to_string()));
    }

    #[test]
    fn tool_call_for_undetected_stack() {
        let tool_calls = [tool_call(Some(StackType::Python))];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, None);
    }

    #[test]
    fn tool_calls_for_detected_and_undetected_stacks() {
        let tool_calls = [
            tool_call(Some(StackType::Python)),
            tool_call(Some(StackType::Rust)),
            tool_call(Some(StackType::Go)),
        ];
        let have = metadata_text(&detected_stacks(), &tool_calls);
        pretty::assert_eq!(have, Some("2 Rust".to_string()));
    }

    #[test]
    fn no_detected_stacks() {
        let tool_calls = [
            tool_call(Some(StackType::Rust)),
            tool_call(Some(StackType::Markdown)),
        ];
        let have = metadata_text(&DetectedStacks::new(vec![]), &tool_calls);
        pretty::assert_eq!(have, None);
    }
}
