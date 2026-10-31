use crate::config::Config;
use crate::domain::{DetectedStack, Result, Tool};
use crate::subshell::ToolCall;

/// a fix that Trident can run
pub trait Fix: Tool {
    /// Provides the shell command to make this tool fix the given `PopulatedStack`.
    /// The fix can look at the files in `all_stacks` to determine if it should run.
    /// If it runs, the fix should only format the files in the given `DetectedStack`,
    /// not find all the files to fix by itself.
    /// This allows running all fixes in parallel.
    fn fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>>;

    /// Provides the shell command to make this tool fix the given `PopulatedStack`
    /// in an advanced way that requires review.
    /// The fix can look at the files in `all_stacks` to determine if it should run.
    /// If it runs, the fix should only format the files in the given `DetectedStack`,
    /// not find all the files to fix by itself.
    /// This allows running all fixes in parallel.
    fn unsafe_fix_commands(&self, stack: &DetectedStack, config: &Config) -> Result<Vec<ToolCall>>;
}
