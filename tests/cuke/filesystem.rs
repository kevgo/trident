use crate::world::{ExistingFile, TridentWorld};
use cucumber::gherkin::Step;
use test_helpers::{docstring_body, platform_newlines, standardize_newlines};
use tokio::fs;

pub enum Newlines {
    Unix,
    Platform,
}

pub async fn write_file(
    world: &mut TridentWorld,
    step: &Step,
    filename: String,
    newlines: Newlines,
) {
    let body = docstring_body(step.docstring.as_ref().unwrap());
    let content = match newlines {
        Newlines::Unix => standardize_newlines(&body),
        Newlines::Platform => platform_newlines(&body),
    };
    let filepath = world.dir.join(&filename);
    let parent = filepath.parent().unwrap();
    if parent != world.dir {
        fs::create_dir_all(parent)
            .await
            .unwrap_or_else(|_| panic!("cannot create parent '{}'", parent.display()));
    }
    fs::write(&filepath, content.as_bytes())
        .await
        .unwrap_or_else(|_| panic!("cannot write to file '{}'", filepath.display()));
    world.original_files.push(ExistingFile {
        name: filename,
        content,
    });
}
