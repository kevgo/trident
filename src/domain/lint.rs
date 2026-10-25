use crate::config::Config;
use crate::domain::{DetectedStack, Result, Tool, ToolCall};

/// a lint that Trident can run
pub trait Lint: Tool {
    /// Provides the shell command to run this lint for the given `PopulatedStack`.
    /// The lint can look at the files in `all_stacks` to determine if it should run.
    /// If it runs, the lint should only verify the files in the given `DetectedStack`,
    /// not find all the files to lint by itself.
    /// This is for performance reasons. Finding files requires expensive OS calls.
    fn lint_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Option<ToolCall>>;
}
