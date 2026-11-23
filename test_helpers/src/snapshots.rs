use itertools::Itertools;
use std::env;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

/// represents a golden snapshot for the `Then it prints:` step
#[derive(Debug)]
pub struct SnapshotEdit {
    /// the .feature file to update
    pub path: PathBuf,

    /// the 1-based line number of the `Then it prints:` step whose docstring to replace
    pub step_line: usize,

    /// the new docstring content (without the surrounding triple quotes)
    pub new_content: String,
}

/// snapshot edits collected during the test run, flushed to disk once it finishes
static SNAPSHOT_EDITS: LazyLock<Mutex<Vec<SnapshotEdit>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// indicates whether the test run should update the golden snapshots
#[must_use]
pub fn enabled() -> bool {
    env::var("TRIDENT_UPDATE_SNAPSHOTS").is_ok_and(|value| !value.is_empty())
}

/// queues a snapshot update to be applied after the test run finishes
pub fn queue_update(edit: SnapshotEdit) {
    SNAPSHOT_EDITS.lock().unwrap().push(edit);
}

/// applies all queued snapshot updates to their .feature files
pub fn flush() {
    let mut edits = std::mem::take(&mut *SNAPSHOT_EDITS.lock().unwrap());
    // Group edits by file and apply them bottom-to-top so that rewriting a
    // docstring never shifts the line numbers of edits still to be applied.
    edits.sort_by(|a, b| a.path.cmp(&b.path).then(b.step_line.cmp(&a.step_line)));
    for (path, group) in &edits.into_iter().chunk_by(|edit| edit.path.clone()) {
        let mut group: Vec<SnapshotEdit> = group.collect();
        // Filter duplicate edits for the same step
        group.dedup_by(|a, b| a.step_line == b.step_line && a.new_content == b.new_content);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read '{}': {err}", path.display()))
            .lines()
            .map(str::to_string)
            .collect();
        for edit in group {
            apply_edit(&mut lines, &edit);
        }
        let mut output = lines.join("\n");
        output.push('\n');
        std::fs::write(&path, output)
            .unwrap_or_else(|err| panic!("cannot write '{}': {err}", path.display()));
    }
}

/// replaces the docstring of the `Then it prints:` step at `edit.step_line` with the new content
fn apply_edit(lines: &mut Vec<String>, edit: &SnapshotEdit) {
    // The docstring opens at the first `"""` line after the step line.
    let search_start = edit.step_line.saturating_sub(1);
    let open = (search_start..lines.len())
        .find(|&i| lines[i].trim() == "\"\"\"")
        .unwrap_or_else(|| {
            panic!(
                "cannot find docstring start for step on line {} in {}",
                edit.step_line,
                edit.path.display()
            )
        });
    let close = (open + 1..lines.len())
        .find(|&i| lines[i].trim() == "\"\"\"")
        .unwrap_or_else(|| {
            panic!(
                "cannot find docstring end for step on line {} in {}",
                edit.step_line,
                edit.path.display()
            )
        });
    // Preserve the indentation of the docstring delimiters.
    let indent: String = lines[open]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    let new_body: Vec<String> = edit
        .new_content
        .lines()
        .map(|line| format!("{indent}{line}"))
        .collect();
    lines.splice(open + 1..close, new_body);
}

#[cfg(test)]
mod tests {

    mod apply_edit {
        use crate::snapshots::{SnapshotEdit, apply_edit};
        use std::path::PathBuf;

        #[test]
        fn happy_path() {
            let mut have = vec![
                "  Scenario: example".to_string(),
                "    Given a file with content".to_string(),
                "      \"\"\"".to_string(),
                "      setup content".to_string(),
                "      \"\"\"".to_string(),
                "    When executing \"trident\"".to_string(),
                "    Then it prints".to_string(),
                "      \"\"\"".to_string(),
                "      old line 1".to_string(),
                "      old line 2".to_string(),
                "      \"\"\"".to_string(),
                "    And the exit code is 0".to_string(),
            ];
            let edit = SnapshotEdit {
                path: PathBuf::from("test.feature"),
                step_line: 7,
                new_content: "new line 1\nnew line 2".to_string(),
            };
            apply_edit(&mut have, &edit);
            let want = vec![
                "  Scenario: example".to_string(),
                "    Given a file with content".to_string(),
                "      \"\"\"".to_string(),
                "      setup content".to_string(),
                "      \"\"\"".to_string(),
                "    When executing \"trident\"".to_string(),
                "    Then it prints".to_string(),
                "      \"\"\"".to_string(),
                "      new line 1".to_string(),
                "      new line 2".to_string(),
                "      \"\"\"".to_string(),
                "    And the exit code is 0".to_string(),
            ];
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn preserves_indentation() {
            let mut lines = vec![
                "    Then it prints the lines".to_string(),
                "        \"\"\"".to_string(),
                "        captured output".to_string(),
                "        \"\"\"".to_string(),
            ];
            let edit = SnapshotEdit {
                path: PathBuf::from("indented.feature"),
                step_line: 1,
                new_content: "unindented one\nunindented two".to_string(),
            };
            apply_edit(&mut lines, &edit);
            assert_eq!(
                lines,
                vec![
                    "    Then it prints the lines".to_string(),
                    "        \"\"\"".to_string(),
                    "        unindented one".to_string(),
                    "        unindented two".to_string(),
                    "        \"\"\"".to_string(),
                ]
            );
        }

        #[test]
        fn can_remove_docstring() {
            let mut lines = vec![
                "  Scenario: example".to_string(),
                "    When executing \"trident\"".to_string(),
                "    Then it prints".to_string(),
                "      \"\"\"".to_string(),
                "      old line 1".to_string(),
                "      old line 2".to_string(),
                "      \"\"\"".to_string(),
                "    And the exit code is 0".to_string(),
            ];

            let edit = SnapshotEdit {
                path: PathBuf::from("empty.feature"),
                step_line: 3,
                new_content: String::new(),
            };
            apply_edit(&mut lines, &edit);
            assert_eq!(
                lines,
                vec![
                    "  Scenario: example".to_string(),
                    "    When executing \"trident\"".to_string(),
                    "    Then it prints".to_string(),
                    "      \"\"\"".to_string(),
                    "      \"\"\"".to_string(),
                    "    And the exit code is 0".to_string(),
                ]
            );
        }

        #[test]
        #[should_panic(expected = "cannot find docstring start")]
        fn no_docstring() {
            let mut lines = vec![
                "    Then it prints".to_string(),
                "      no docstring here".to_string(),
            ];
            let edit = SnapshotEdit {
                path: PathBuf::from("missing-start.feature"),
                step_line: 1,
                new_content: "new output".to_string(),
            };

            apply_edit(&mut lines, &edit);
        }

        #[test]
        #[should_panic(expected = "cannot find docstring end")]
        fn unclosed_docstring() {
            let mut lines = vec![
                "    Then it prints".to_string(),
                "      \"\"\"".to_string(),
                "      never closed".to_string(),
            ];
            let edit = SnapshotEdit {
                path: PathBuf::from("missing-end.feature"),
                step_line: 1,
                new_content: "new output".to_string(),
            };

            apply_edit(&mut lines, &edit);
        }
    }
}
