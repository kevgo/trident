//! helper functions that run "git status"

use crate::domain::Result;
use crate::git::GitCommandExt;
use crate::git::Repo;
use crate::git::ZeroString;

impl Repo {
    /// runs `git status` and returns its stdout
    pub(crate) fn status(&self, extra_args: &[&str]) -> Result<GitStatusOutput> {
        Ok(self
            .git_command()
            .args(["status", "--porcelain=v1", "-z"])
            .args(extra_args)
            .run_stdout_zero()?
            .into())
    }
}

/// output of `git status --porcelain=v1 -z`
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GitStatusOutput(ZeroString);

impl From<&str> for GitStatusOutput {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<String> for GitStatusOutput {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<ZeroString> for GitStatusOutput {
    fn from(value: ZeroString) -> Self {
        Self(value)
    }
}

impl GitStatusOutput {
    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// destination records, skipping rename/copy original paths
    pub(crate) fn records(&self) -> impl Iterator<Item = Record<'_>> {
        self.records_with_orig_paths().map(|(record, _)| record)
    }

    /// the paths of the files that this status output reports as deleted or renamed away
    pub(crate) fn removed_paths(&self) -> impl Iterator<Item = &str> {
        self.records_with_orig_paths()
            .flat_map(|(record, orig_path)| removed_paths_of(&record, orig_path))
    }

    /// the paths of the staged files that this status output reports as deleted or renamed away
    pub(crate) fn staged_removed_paths(&self) -> impl Iterator<Item = &str> {
        self.records_with_orig_paths()
            .filter(|(record, _)| record.index != ' ')
            .flat_map(|(record, orig_path)| removed_paths_of(&record, orig_path))
    }

    /// all records together with the original path of renames and copies
    fn records_with_orig_paths(&self) -> impl Iterator<Item = (Record<'_>, Option<&str>)> {
        let mut lines = self.0.lines();
        std::iter::from_fn(move || {
            let line = lines.next()?;
            let record = Record::parse(line)?;
            // Rename/copy entries are `XY dest\0orig\0`.
            let orig_path = if record.has_orig_path() {
                lines.next()
            } else {
                None
            };
            Some((record, orig_path))
        })
    }
}

/// provides the paths that the given record reports as deleted or renamed away
fn removed_paths_of<'a>(
    record: &Record<'a>,
    orig_path: Option<&'a str>,
) -> impl Iterator<Item = &'a str> + use<'a> {
    let deleted = record.is_deleted().then_some(record.path);
    let renamed_away = orig_path.filter(|_| record.is_renamed());
    deleted.into_iter().chain(renamed_away)
}

/// a record from `git status --porcelain=v1 -z`
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Record<'a> {
    /// X status: how the index (staging area) differs from HEAD
    pub index: char,
    /// Y status: how the working tree differs from the index
    pub worktree: char,
    /// file path, for rename/copy this is the destination path
    pub path: &'a str,
}

impl Record<'_> {
    /// parses the XY status prefix and path from a porcelain line
    pub(crate) fn parse(line: &str) -> Option<Record<'_>> {
        if line.len() < 3 {
            return None;
        }
        let mut chars = line.chars();
        let Some(index) = chars.next() else {
            log_unexpected_line(line);
            return None;
        };
        let Some(worktree) = chars.next() else {
            log_unexpected_line(line);
            return None;
        };
        if !Self::is_known_status(index) || !Self::is_known_status(worktree) {
            log_unexpected_line(line);
            return None;
        }
        let Some(space) = chars.next() else {
            log_unexpected_line(line);
            return None;
        };
        if space != ' ' {
            log_unexpected_line(line);
            return None;
        }
        Some(Record {
            index,
            worktree,
            path: chars.as_str(),
        })
    }

    /// provides whether this record has an original file path
    fn has_orig_path(&self) -> bool {
        // only rename/copy entries have an original file path
        matches!(self.index, 'R' | 'C') || matches!(self.worktree, 'R' | 'C')
    }

    fn is_known_status(status: char) -> bool {
        matches!(
            status,
            'A' | 'M' | 'R' | 'C' | 'T' | ' ' | 'D' | 'U' | '?' | '!'
        )
    }

    /// provides whether this record describes the deletion of a file
    pub fn is_deleted(&self) -> bool {
        self.index == 'D' || self.worktree == 'D'
    }

    pub fn is_uncommitted(&self) -> bool {
        if self.index == '!' || self.worktree == '!' || self.worktree == 'D' {
            return false;
        }
        if self.worktree == 'D' {
            // the file doesn't exist anymore, so there is no content to process
            return false;
        }
        is_present_change(self.index) || is_present_change(self.worktree)
    }

    /// provides whether the file at this record's path was deleted
    fn is_deleted(&self) -> bool {
        self.index == 'D' || self.worktree == 'D'
    }

    /// provides whether the file at this record's original path was renamed
    fn is_renamed(&self) -> bool {
        self.index == 'R' || self.worktree == 'R'
    }
}

fn is_present_change(status: char) -> bool {
    matches!(status, 'A' | 'M' | 'R' | 'C' | 'T' | '?')
}

fn log_unexpected_line(line: &str) {
    println!("unexpected line in output of \"git status --porcelain=v1 -z\": {line}");
}

#[cfg(test)]
mod tests {

    mod record {
        use super::super::Record;
        use maplit::hashmap;

        #[test]
        fn has_orig_path() {
            let tests = hashmap! {
                "R  new.rs" => true,
                "C  copy.rs" => true,
                "RM new.rs" => true,
                "CR copy.rs" => true,
                " R renamed.rs" => true,
                " C copied.rs" => true,
                "M  file.rs" => false,
                "MM file.rs" => false,
                "?? file.rs" => false,
                "A  file.rs" => false,
                " D file.rs" => false,
            };
            for (give, want) in tests {
                let have = Record::parse(give).expect(give);
                assert_eq!(have.has_orig_path(), want, "{give}");
            }
        }

        #[test]
        fn parse() {
            let tests = hashmap! {
                "MM file.rs" => Some(Record { index: 'M', worktree: 'M', path: "file.rs" }),
                "M  my file.txt" => Some(Record { index: 'M', worktree: ' ', path: "my file.txt" }),
                "?? file\"quote.txt" => Some(Record { index: '?', worktree: '?', path: "file\"quote.txt" }),
                "R  new file.txt" => Some(Record { index: 'R', worktree: ' ', path: "new file.txt" }),
                "XY file.rs" => None,
                "M" => None,
                "" => None,
            };
            for (give, want) in tests {
                let have = Record::parse(give);
                assert_eq!(have, want, "{give}");
            }
        }
    }

    mod is_uncommitted {
        use super::super::Record;
        use maplit::hashmap;

        #[test]
        fn is_uncommitted() {
            let tests = hashmap! {
                "M  file.rs" => true,
                " M file.rs" => true,
                "MM file.rs" => true,
                "A  file.rs" => true,
                "R  file.rs" => true,
                "?? file.rs" => true,
                "D  file.rs" => false,
                " D file.rs" => false,
                "MD file.rs" => false,
                "AD file.rs" => false,
                "RD file.rs" => false,
                "!! file.rs" => false,
            };
            for (give, want) in tests {
                let record = Record::parse(give).expect(give);
                assert_eq!(record.is_uncommitted(), want, "{give}");
            }
        }
    }

    mod removed_paths {
        use super::super::GitStatusOutput;

        #[test]
        fn deleted_and_renamed_files() {
            let give = [
                "D  staged-delete.rs",
                " D unstaged-delete.rs",
                "MD modified-then-deleted.rs",
                "R  new.rs\0old.rs",
                "RD new2.rs\0old2.rs",
                "C  copy.rs\0original.rs",
                "M  modified.rs",
                "?? untracked.rs",
            ]
            .join("\0");
            let output = GitStatusOutput::from(give);
            let have: Vec<&str> = output.removed_paths().collect();
            let want = vec![
                "staged-delete.rs",
                "unstaged-delete.rs",
                "modified-then-deleted.rs",
                "old.rs",
                "new2.rs",
                "old2.rs",
            ];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn staged_only() {
            let give = [
                "D  staged-delete.rs",
                " D unstaged-delete.rs",
                "MD modified-then-deleted.rs",
                "R  new.rs\0old.rs",
                "M  modified.rs",
                "?? untracked.rs",
            ]
            .join("\0");
            let output = GitStatusOutput::from(give);
            let have: Vec<&str> = output.staged_removed_paths().collect();
            let want = vec!["staged-delete.rs", "modified-then-deleted.rs", "old.rs"];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn no_removed_files() {
            let output = GitStatusOutput::from("M  file.rs");
            assert_eq!(output.removed_paths().count(), 0);
        }
    }

    mod records {
        use super::super::GitStatusOutput;
        use super::super::Record;
        use maplit::hashmap;

        #[test]
        fn parse() {
            let tests = hashmap! {
                "MM file.rs" => Some(Record { index: 'M', worktree: 'M', path: "file.rs" }),
                "M  file.rs" => Some(Record { index: 'M', worktree: ' ', path: "file.rs" }),
                " M file.rs" => Some(Record { index: ' ', worktree: 'M', path: "file.rs" }),
                "?? file.rs" => Some(Record { index: '?', worktree: '?', path: "file.rs" }),
                "?? my file.txt" => Some(Record { index: '?', worktree: '?', path: "my file.txt" }),
                "?? file\"quote.txt" => Some(Record { index: '?', worktree: '?', path: "file\"quote.txt" }),
                "!! file.rs" => Some(Record { index: '!', worktree: '!', path: "file.rs" }),
                "UU file.rs" => Some(Record { index: 'U', worktree: 'U', path: "file.rs" }), // unmerged conflict in file
                "D  file.rs" => Some(Record { index: 'D', worktree: ' ', path: "file.rs" }),
                " D file.rs" => Some(Record { index: ' ', worktree: 'D', path: "file.rs" }),
                "A  file.rs" => Some(Record { index: 'A', worktree: ' ', path: "file.rs" }),
                " A file.rs" => Some(Record { index: ' ', worktree: 'A', path: "file.rs" }),
                "R  dir/new.rs" => Some(Record { index: 'R', worktree: ' ', path: "dir/new.rs" }), // renamed file (dest path)
                "C  dir/new.rs" => Some(Record { index: 'C', worktree: ' ', path: "dir/new.rs" }), // copied file (dest path)
                "R  new file.txt" => Some(Record { index: 'R', worktree: ' ', path: "new file.txt" }),
                "foo" => None,
                "foo bar" => None,
                "MM" => None,
                "" => None,
            };
            for (give, want) in tests {
                let have = Record::parse(give);
                pretty::assert_eq!(have, want, "{give}");
            }
        }

        #[test]
        fn skips_rename_and_copy_orig_paths() {
            let give = [
                "R  new file.txt\0old file.txt",
                "C  copy.rs\0original.rs",
                "?? some file.txt",
                "M  file.rs",
            ]
            .join("\0");
            let output = GitStatusOutput::from(give);
            let have = output.records().collect::<Vec<_>>();
            pretty::assert_eq!(
                have,
                vec![
                    Record {
                        index: 'R',
                        worktree: ' ',
                        path: "new file.txt"
                    },
                    Record {
                        index: 'C',
                        worktree: ' ',
                        path: "copy.rs"
                    },
                    Record {
                        index: '?',
                        worktree: '?',
                        path: "some file.txt"
                    },
                    Record {
                        index: 'M',
                        worktree: ' ',
                        path: "file.rs"
                    },
                ]
            );
        }

        #[test]
        fn skips_empty_entries() {
            let tests = hashmap! {
                "" => vec![],
                "\0" => vec![],
                "M  file.rs\0" => vec![Record { index: 'M', worktree: ' ', path: "file.rs" }],
            };
            for (give, want) in tests {
                let output = GitStatusOutput::from(give);
                let have = output.records().collect::<Vec<_>>();
                assert_eq!(have, want, "{give}");
            }
        }
    }
}
