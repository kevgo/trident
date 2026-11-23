use cucumber::World;
use std::io;
use std::path::PathBuf;
use std::process::Output;
use tokio::fs;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TridentWorld {
    /// need to hold on to this to keep the tempdir alive
    _tempdir: tempfile::TempDir,

    /// the directory containing the test files for the current scenario
    pub dir: PathBuf,

    /// the current working directory
    pub cwd: PathBuf,

    pub original_files: Vec<ExistingFile>,

    /// the result of running Trident
    pub output: Option<Output>,

    /// path to the .feature file of the currently running scenario
    pub feature_path: Option<PathBuf>,

    /// the file into which the Git double logs the Git commands that Trident runs
    pub git_log: PathBuf,

    /// the Git commands that the last Trident run executed
    pub git_commands: Option<Vec<String>>,

    /// whether a step verified what Trident printed to STDOUT
    pub stdout_verified: bool,

    /// whether a step verified what Trident printed to STDERR
    pub stderr_verified: bool,

    /// whether a step verified the Git commands that Trident executed
    pub git_commands_verified: bool,
}

impl TridentWorld {
    pub fn new() -> Self {
        let tempdir = tempfile::tempdir().unwrap();
        let random = rand::random_range(0..u64::MAX).to_string();
        let dir = tempdir.path().join(random);
        std::fs::create_dir(&dir).unwrap();
        let cwd = std::env::current_dir().unwrap();
        // outside of the workspace so that it doesn't show up as an untracked file
        let git_log = tempdir.path().join("git-commands.log");
        Self {
            _tempdir: tempdir,
            cwd,
            dir,
            original_files: Vec::new(),
            output: None,
            feature_path: None,
            git_log,
            git_commands: None,
            stdout_verified: false,
            stderr_verified: false,
            git_commands_verified: false,
        }
    }

    /// provides the exit code of the Atlanta run
    pub fn exit_code(&self) -> i32 {
        match &self.output {
            Some(result) => result.status.code().unwrap(),
            None => panic!(),
        }
    }

    /// provides the Git commands that the last Trident run executed
    pub fn git_commands(&self) -> &[String] {
        self.git_commands.as_deref().expect("Trident hasn't run")
    }

    /// describes the aspects of the Trident run that the scenario did not verify
    pub fn unverified_aspects(&self) -> Vec<&'static str> {
        let mut result = Vec::new();
        if !self.stdout_verified {
            result.push("STDOUT");
        }
        if !self.stderr_verified {
            result.push("STDERR");
        }
        if !self.git_commands_verified {
            result.push("the executed Git commands");
        }
        result
    }

    /// provides the content that the file with the given filename had before the tested logic ran
    pub fn original_file_content(&self, filename: &str) -> Option<&str> {
        self.original_files
            .iter()
            .find(|file| file.name == filename)
            .map(|file| file.content.as_str())
    }

    /// provides the content that the file with the given filename has after the tested logic ran
    pub async fn current_file_content(&self, filename: &str) -> io::Result<String> {
        let filepath = self.dir.join(filename);
        fs::read_to_string(filepath).await
    }
}

#[derive(Debug)]
pub struct ExistingFile {
    pub name: String,
    pub content: String,
}
