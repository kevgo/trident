mod branch_changed_files;
mod branch_committed_files;
mod command;
mod commit_empty;
mod committed_files;
mod create_and_commit_file;
mod create_and_switch_to_branch;
mod create_unstaged_file;
mod current_branch;
mod default_branch;
mod diff;
mod has_ref;
mod last_commit_message;
mod merge_base;
mod rename_current_branch;
mod repo;
mod stage;
mod staged;
mod status;
mod switch;
mod uncommitted;
mod zerostring;

pub(crate) use command::GitCommandExt;
pub(crate) use repo::Repo;
#[cfg(test)]
pub(crate) use staged::StagedFiles;
pub(crate) use zerostring::ZeroString;
