pub mod actionlint;
pub mod biome;
pub mod checkstyle;
pub mod delete_empty_folders;
pub mod dprint;
pub mod gherkin_lint;
pub mod ghokin;
pub mod git_diff_check;
pub mod gofumpt;
pub mod golangci_lint;
pub mod hadolint;
pub mod keep_sorted;
pub mod prettier;
pub mod pyright;
pub mod ripgrep;
pub mod ruff;
pub mod rumdl;
pub mod sqlfmt;
pub mod taplo;
pub mod text_runner;
pub mod tikibase;

use crate::domain::{Result, UserError};
use rta::applications::AppDefinition;

/// Provides the RTA command to run the given RTA App.
/// Installs the app if needed.
pub(crate) fn get_rta_command(args: &GetRTACmdArgs<'_>) -> Result<Option<conc::Executable>> {
    // Apps like Prettier need to install multiple apps to run (first Node, then Prettier).
    // So we keep trying in a loop until either the command is available,
    // or we get stuck needing the same app installed again after having already installed it.
    let apps = rta::applications::all();
    let mut added = Vec::new();
    loop {
        let cmd_result = rta::get_cmd(rta::GetCmdArgs {
            app: args.app,
            app_args: args.args.clone(),
            version: args.version.clone(),
            apps: &apps,
            from_source: false,
            include_apps: vec![],
            optional: true,
            verbose: false,
        });
        match cmd_result {
            Ok(cmd) => {
                return Ok(cmd.map(|command| conc::Executable {
                    name: args.name.clone(),
                    command: (&command).into(),
                }));
            }
            Err(err) => match &err {
                rta::error::UserError::RunRequestMissingVersion { app }
                | rta::error::UserError::MissingRuntime {
                    runtime: app,
                    needed_by: _,
                    script: _,
                    searched_dirs: _,
                }
                | rta::error::UserError::NoVersionsFound { app } => {
                    if added.contains(app) {
                        // We have tried to install this missing app before,
                        // and it didn't work.
                        // Now we know it cannot be installed on this platform.
                        return Err(UserError::Rta { err });
                    }
                    let add_args = rta::commands::AddArgs {
                        app_name: app.to_owned(),
                        verbose: true,
                    };
                    if let Err(err) = rta::commands::add(add_args, &apps) {
                        return Err(UserError::Rta { err });
                    }
                    added.push(app.to_owned());
                }
                _ => return Err(UserError::Rta { err }),
            },
        }
    }
}

pub struct GetRTACmdArgs<'a> {
    name: String,
    app: &'a dyn AppDefinition,
    args: Vec<String>,
    version: Option<rta::Version>,
}
