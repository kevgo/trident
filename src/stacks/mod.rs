mod css;
mod cucumber;
mod dockerfile;
mod go;
mod json;
mod jsonc;
mod markdown;
mod python;
mod rust;
mod shell;
mod sql;
mod toml;
mod typescript;
mod unknown;
mod yml;

use crate::domain::{DetectedStack, DetectedStacks, File, Files, Ignores, Stack};
use crate::git::StagedFiles;
pub use css::Css;
pub use cucumber::Cucumber;
pub use dockerfile::Dockerfile;
pub use go::Go;
use ignore::WalkBuilder;
pub use json::Json;
pub use jsonc::JsonC;
pub use markdown::Markdown;
pub use python::Python;
pub use rust::Rust;
pub use shell::Shell;
pub use sql::Sql;
use std::path::Path;
pub use toml::Toml;
pub use typescript::Typescript;
pub use unknown::Unknown;
pub use yml::Yml;

/// provides all stacks that Trident supports
#[must_use]
pub fn all() -> Vec<Box<dyn Stack>> {
    vec![
        // keep-sorted start
        Box::new(Css {}),
        Box::new(Cucumber {}),
        Box::new(Dockerfile {}),
        Box::new(Go {}),
        Box::new(Json {}),
        Box::new(JsonC {}),
        Box::new(Markdown {}),
        Box::new(Python {}),
        Box::new(Rust {}),
        Box::new(Shell {}),
        Box::new(Sql {}),
        Box::new(Toml {}),
        Box::new(Typescript {}),
        Box::new(Yml {}),
        // keep-sorted end
        Box::new(Unknown {}),
    ]
}

/// provides the stacks for the given staged files
#[must_use]
pub fn from_staged(staged: &StagedFiles, ignores: &Ignores) -> DetectedStacks {
    from_files(staged.all(), ignores)
}

/// provides the stacks for the given files
#[must_use]
pub fn from_files<'a>(
    files: impl IntoIterator<Item = &'a File>,
    ignores: &Ignores,
) -> DetectedStacks {
    let all_stacks = all();
    let mut detected_stacks: Vec<DetectedStack> = all_stacks
        .into_iter()
        .map(|stack| DetectedStack {
            stack,
            files: Files::empty(),
        })
        .collect();
    for file in files {
        if ignores.matches_self_or_parent(file.as_ref()) {
            continue;
        }
        for detected_stack in &mut detected_stacks {
            if detected_stack.stack.matches(file.as_ref()) {
                detected_stack.files.push(file.clone());
                break;
            }
        }
    }
    let result = detected_stacks
        .into_iter()
        .filter(|stack| !stack.files.is_empty())
        .map(|mut stack| {
            stack.files.sort_unstable();
            stack
        })
        .collect();
    DetectedStacks::new(result)
}

/// adds all workspace files of the stacks that the given removed files belonged to
///
/// Removing a file can break other files in its stack, for example the ones importing it.
#[must_use]
pub fn include_stacks_of_removed_files(
    changed: DetectedStacks,
    removed: &[File],
    ignores: &Ignores,
) -> DetectedStacks {
    include_stacks_of_removed_files_in(Path::new("./"), changed, removed, ignores)
}

/// adds all files under `dir` of the stacks that the given removed files belonged to
#[must_use]
pub fn include_stacks_of_removed_files_in(
    dir: &Path,
    changed: DetectedStacks,
    removed: &[File],
    ignores: &Ignores,
) -> DetectedStacks {
    let removed_stacks = from_files(removed, ignores);
    if removed_stacks.is_empty() {
        return changed;
    }
    let mut files: Vec<File> = Vec::new();
    for changed_stack in &changed {
        files.extend(changed_stack.files.into_iter().cloned());
    }
    for workspace_stack in &discover_all_in(dir, ignores) {
        if removed_stacks
            .get_stack(workspace_stack.stack.stack_type())
            .is_some()
        {
            files.extend(workspace_stack.files.into_iter().cloned());
        }
    }
    files.sort_unstable();
    files.dedup();
    from_files(&files, ignores)
}

/// provides all stacks and their files that exist in the workspace
#[must_use]
pub fn discover_all(ignores: &Ignores) -> DetectedStacks {
    discover_all_in(Path::new("./"), ignores)
}

/// provides all stacks and their files found under `dir`
#[must_use]
pub fn discover_all_in(dir: &Path, ignores: &Ignores) -> DetectedStacks {
    let all_stacks = all();
    let mut detected_stacks: Vec<DetectedStack> = all_stacks
        .into_iter()
        .map(|stack| DetectedStack {
            stack,
            files: Files::empty(),
        })
        .collect();
    let ignores2 = ignores.clone();
    let walk = WalkBuilder::new(dir)
        // This code still ignores most hidden files (all except the ".github" folder)
        // using the filter below.
        .hidden(false)
        .filter_entry(move |entry| {
            let entry_path = entry.path();
            let Some(second_component) = entry_path.components().nth(1) else {
                // path has no second component
                return false;
            };
            let Some(first_char) = second_component
                .as_os_str()
                .to_string_lossy()
                .chars()
                .next()
            else {
                // second component is empty
                return false;
            };
            if first_char == '.' {
                // This is a hidden file.
                // Only include the ".github" folder.
                return entry_path.starts_with("./.github/");
            }
            // exclude files ignored in the config file
            !ignores2.matches_self(entry_path, entry.file_type().is_some_and(|ft| ft.is_dir()))
        })
        .build();
    for entry in walk {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        for detected_stack in &mut detected_stacks {
            if detected_stack.stack.matches(path) {
                detected_stack.files.push(path.into());
                break;
            }
        }
    }
    let result = detected_stacks
        .into_iter()
        .filter(|stack| !stack.files.is_empty())
        .map(|mut stack| {
            stack.files.sort_unstable();
            stack
        })
        .collect();
    DetectedStacks::new(result)
}

#[cfg(test)]
mod tests {

    mod include_stacks_of_removed_files {
        use crate::domain::{DetectedStacks, File, Ignores};
        use crate::stacks::{from_files, include_stacks_of_removed_files_in};
        use std::fs;
        use tempfile::TempDir;

        fn file_names(stacks: &DetectedStacks) -> Vec<String> {
            stacks
                .into_iter()
                .flat_map(|stack| stack.files.into_iter().map(ToString::to_string))
                .collect()
        }

        #[test]
        fn no_removed_files() {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("other.md"), "").unwrap();
            let changed = from_files(&[File::from("changed.md")], &Ignores::empty());
            let have =
                include_stacks_of_removed_files_in(dir.path(), changed, &[], &Ignores::empty());
            pretty::assert_eq!(file_names(&have), vec!["changed.md"]);
        }

        #[test]
        fn adds_all_files_of_the_removed_stack_only() {
            let dir = TempDir::new().unwrap();
            let in_stack = dir.path().join("in_stack.md");
            fs::write(&in_stack, "").unwrap();
            fs::write(dir.path().join("other_stack.py"), "").unwrap();
            let changed = from_files(&[File::from("changed.toml")], &Ignores::empty());
            let have = include_stacks_of_removed_files_in(
                dir.path(),
                changed,
                &[File::from("removed.md")],
                &Ignores::empty(),
            );
            let mut want = vec![
                in_stack.to_string_lossy().to_string(),
                "changed.toml".to_string(),
            ];
            want.sort();
            let mut have_names = file_names(&have);
            have_names.sort();
            pretty::assert_eq!(have_names, want);
        }

        #[test]
        fn keeps_changed_files_not_discoverable_in_the_workspace() {
            let dir = TempDir::new().unwrap();
            let changed = from_files(&[File::from(".hidden/changed.md")], &Ignores::empty());
            let have = include_stacks_of_removed_files_in(
                dir.path(),
                changed,
                &[File::from("removed.md")],
                &Ignores::empty(),
            );
            pretty::assert_eq!(file_names(&have), vec![".hidden/changed.md"]);
        }

        #[test]
        fn removed_stack_has_no_remaining_files() {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("other_stack.py"), "").unwrap();
            let have = include_stacks_of_removed_files_in(
                dir.path(),
                from_files(&[], &Ignores::empty()),
                &[File::from("removed.md")],
                &Ignores::empty(),
            );
            assert!(have.is_empty());
        }
    }

    mod discover {
        use crate::domain::{DetectedStack, DetectedStacks, Files, Ignores};
        use crate::stacks::discover_all_in;
        use crate::stacks::{Go, Json, JsonC, Markdown, Unknown};
        use std::fs;
        use std::path::Path;
        use tempfile::TempDir;

        fn make_files(dir: &TempDir, paths: &[&str]) {
            for path in paths {
                let full = dir.path().join(path);
                if let Some(parent) = full.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(&full, "").unwrap();
            }
        }

        #[test]
        fn empty_directory() {
            let dir = TempDir::new().unwrap();
            let stacks = discover_all_in(dir.path(), &Ignores::empty());
            assert!(stacks.is_empty());
        }

        #[test]
        fn detects_multiple_stacks() {
            let dir = TempDir::new().unwrap();
            make_files(
                &dir,
                &[
                    "main.go",
                    "config.json",
                    "README.md",
                    "archive.tar",
                    "trident.jsonc",
                ],
            );
            let have = discover_all_in(dir.path(), &Ignores::empty());
            let root = dir.path();
            let want = DetectedStacks::new(vec![
                DetectedStack {
                    stack: Box::new(Go {}),
                    files: Files::from(vec![root.join("main.go")]),
                },
                DetectedStack {
                    stack: Box::new(Json {}),
                    files: Files::from(vec![root.join("config.json")]),
                },
                DetectedStack {
                    stack: Box::new(JsonC {}),
                    files: Files::from(vec![root.join("trident.jsonc")]),
                },
                DetectedStack {
                    stack: Box::new(Markdown {}),
                    files: Files::from(vec![root.join("README.md")]),
                },
                DetectedStack {
                    stack: Box::new(Unknown {}),
                    files: Files::from(vec![root.join("archive.tar")]),
                },
            ]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn nested_directories() {
            #[cfg(not(windows))]
            let nested = "src/nested/deep/main.go";
            #[cfg(windows)]
            let nested = "src\\nested\\deep\\main.go";

            let dir = TempDir::new().unwrap();
            make_files(&dir, &[nested]);
            let have = discover_all_in(dir.path(), &Ignores::empty());
            let root = dir.path();
            let want = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Go {}),
                files: Files::from(vec![root.join(nested)]),
            }]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn ignores_file() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &["main.go", "ignored.go"]);
            let ignores = Ignores::new(&["ignored.go".to_string()], Path::new("./")).unwrap();
            let have = discover_all_in(dir.path(), &ignores);
            let root = dir.path();
            let want = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Go {}),
                files: Files::from(vec![root.join("main.go")]),
            }]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn ignores_directory() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &["main.go", "vendor/lib.go"]);
            let ignores = Ignores::new(&["vendor/".to_string()], Path::new("./")).unwrap();
            let have = discover_all_in(dir.path(), &ignores);
            let root = dir.path();
            let want = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Go {}),
                files: Files::from(vec![root.join("main.go")]),
            }]);
            pretty::assert_eq!(have, want);
        }
    }
}
