mod compare_lines_any_order;
mod compare_regexes_any_order;
mod content_lines;
mod docstring_body;
mod has_additional_lines;
mod newlines;
mod platform_newlines;
mod remove_line_matching;
mod remove_lines;
pub mod snapshots;

pub use compare_lines_any_order::compare_lines_any_order;
pub use compare_regexes_any_order::compare_regexes_any_order;
use content_lines::content_lines;
pub use docstring_body::docstring_body;
pub use has_additional_lines::has_additional_lines;
pub use newlines::{platform_newlines, standardize_newlines};
use remove_line_matching::remove_line_matching;
use remove_lines::remove_lines;
