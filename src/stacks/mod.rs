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

/// adds the stacks of the given deleted files to the given stacks
///
/// The deleted files don't become part of the stacks because tools cannot process them.
/// A stack that has only deleted files therefore has no files.
/// Tools that process individual files skip such a stack, all other tools run.
#[must_use]
pub fn with_deleted(
    stacks: DetectedStacks,
    deleted_files: &[File],
    ignores: &Ignores,
) -> DetectedStacks {
    let deleted = from_files(deleted_files, ignores);
    let mut existing: Vec<DetectedStack> = stacks.into_iter().collect();
    let mut result = Vec::with_capacity(existing.len());
    for stack in all() {
        let stack_type = stack.stack_type();
        if let Some(pos) = existing
            .iter()
            .position(|detected| detected.stack.stack_type() == stack_type)
        {
            result.push(existing.swap_remove(pos));
        } else if deleted.get_stack(stack_type).is_some() {
            result.push(DetectedStack {
                stack,
                files: Files::empty(),
            });
        }
    }
    DetectedStacks::new(result)
}

/// provides the given stacks with all the workspace files that belong to them
///
/// The given files stay part of the result even if a walk of the workspace doesn't find them,
/// for example changed hidden files.
#[must_use]
pub fn with_all_files(stacks: &DetectedStacks, ignores: &Ignores) -> DetectedStacks {
    with_all_files_in(Path::new("./"), stacks, ignores)
}

fn with_all_files_in(dir: &Path, stacks: &DetectedStacks, ignores: &Ignores) -> DetectedStacks {
    if stacks.is_empty() {
        return DetectedStacks::new(vec![]);
    }
    let workspace = discover_all_in(dir, ignores);
    let mut result = Vec::with_capacity(stacks.len());
    for stack in all() {
        let stack_type = stack.stack_type();
        let Some(given) = stacks.get_stack(stack_type) else {
            continue;
        };
        let mut files = given.files.clone();
        if let Some(workspace_stack) = workspace.get_stack(stack_type) {
            files.merge(&workspace_stack.files);
        }
        result.push(DetectedStack { stack, files });
    }
    DetectedStacks::new(result)
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

    mod with_all_files {
        use crate::domain::{DetectedStack, DetectedStacks, Files, Ignores};
        use crate::stacks::{Markdown, Rust, with_all_files_in};
        use std::fs;
        use std::path::{Path, PathBuf};
        use tempfile::TempDir;

        fn make_files(dir: &TempDir, paths: &[&str]) {
            for path in paths {
                fs::write(dir.path().join(path), "").unwrap();
            }
        }

        fn rust_stack(files: Vec<PathBuf>) -> DetectedStack {
            DetectedStack {
                stack: Box::new(Rust {}),
                files: Files::from(files),
            }
        }

        #[test]
        fn adds_the_other_files_of_the_stack() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &["changed.rs", "untouched.rs", "README.md"]);
            let root = dir.path();
            let given = DetectedStacks::new(vec![rust_stack(vec![root.join("changed.rs")])]);
            let have = with_all_files_in(root, &given, &Ignores::empty());
            let want = DetectedStacks::new(vec![rust_stack(vec![
                root.join("changed.rs"),
                root.join("untouched.rs"),
            ])]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn stack_with_only_deleted_files() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &["untouched.rs", "README.md"]);
            let root = dir.path();
            let given = DetectedStacks::new(vec![rust_stack(vec![])]);
            let have = with_all_files_in(root, &given, &Ignores::empty());
            let want = DetectedStacks::new(vec![rust_stack(vec![root.join("untouched.rs")])]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn stack_without_any_remaining_files() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &["README.md"]);
            let given = DetectedStacks::new(vec![rust_stack(vec![])]);
            let have = with_all_files_in(dir.path(), &given, &Ignores::empty());
            let want = DetectedStacks::new(vec![rust_stack(vec![])]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn keeps_given_files_that_the_walk_does_not_find() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &[".hidden.rs", "untouched.rs"]);
            let root = dir.path();
            let given = DetectedStacks::new(vec![rust_stack(vec![root.join(".hidden.rs")])]);
            let have = with_all_files_in(root, &given, &Ignores::empty());
            let want = DetectedStacks::new(vec![rust_stack(vec![
                root.join(".hidden.rs"),
                root.join("untouched.rs"),
            ])]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn skips_ignored_files() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &["changed.rs", "ignored.rs"]);
            let root = dir.path();
            let given = DetectedStacks::new(vec![rust_stack(vec![root.join("changed.rs")])]);
            let ignores = Ignores::new(&["ignored.rs".to_string()], Path::new("./")).unwrap();
            let have = with_all_files_in(root, &given, &ignores);
            let want = DetectedStacks::new(vec![rust_stack(vec![root.join("changed.rs")])]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn does_not_add_other_stacks() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &["README.md"]);
            let root = dir.path();
            let given = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Markdown {}),
                files: Files::empty(),
            }]);
            let have = with_all_files_in(root, &given, &Ignores::empty());
            let want = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Markdown {}),
                files: Files::from(vec![root.join("README.md")]),
            }]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn no_stacks() {
            let dir = TempDir::new().unwrap();
            make_files(&dir, &["README.md"]);
            let given = DetectedStacks::new(vec![]);
            let have = with_all_files_in(dir.path(), &given, &Ignores::empty());
            pretty::assert_eq!(have, DetectedStacks::new(vec![]));
        }
    }

    mod with_deleted {
        use crate::domain::{DetectedStack, DetectedStacks, File, Files, Ignores};
        use crate::stacks::{Markdown, Rust, Toml, from_files, with_deleted};
        use std::path::{Path, PathBuf};

        #[test]
        fn adds_stack_of_deleted_file_without_files() {
            let stacks = from_files(&[File::from("README.md")], &Ignores::empty());
            let have = with_deleted(stacks, &[File::from("gone.rs")], &Ignores::empty());
            let want = DetectedStacks::new(vec![
                DetectedStack {
                    stack: Box::new(Markdown {}),
                    files: Files::from(vec![PathBuf::from("README.md")]),
                },
                DetectedStack {
                    stack: Box::new(Rust {}),
                    files: Files::empty(),
                },
            ]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn keeps_the_order_of_the_stacks() {
            let stacks = from_files(&[File::from("Cargo.toml")], &Ignores::empty());
            let deleted = [File::from("gone.rs"), File::from("gone.md")];
            let have = with_deleted(stacks, &deleted, &Ignores::empty());
            let want = DetectedStacks::new(vec![
                DetectedStack {
                    stack: Box::new(Markdown {}),
                    files: Files::empty(),
                },
                DetectedStack {
                    stack: Box::new(Rust {}),
                    files: Files::empty(),
                },
                DetectedStack {
                    stack: Box::new(Toml {}),
                    files: Files::from(vec![PathBuf::from("Cargo.toml")]),
                },
            ]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn does_not_add_deleted_files_to_existing_stacks() {
            let stacks = from_files(&[File::from("kept.rs")], &Ignores::empty());
            let have = with_deleted(stacks, &[File::from("gone.rs")], &Ignores::empty());
            let want = DetectedStacks::new(vec![DetectedStack {
                stack: Box::new(Rust {}),
                files: Files::from(vec![PathBuf::from("kept.rs")]),
            }]);
            pretty::assert_eq!(have, want);
        }

        #[test]
        fn skips_ignored_deleted_files() {
            let stacks = DetectedStacks::new(vec![]);
            let ignores = Ignores::new(&["gone.rs".to_string()], Path::new("./")).unwrap();
            let have = with_deleted(stacks, &[File::from("gone.rs")], &ignores);
            pretty::assert_eq!(have, DetectedStacks::new(vec![]));
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
