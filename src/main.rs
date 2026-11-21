use std::process::ExitCode;
use trident::cli::input::Command;

fn main() -> ExitCode {
    match inner() {
        Ok(exitcode) => exitcode,
        Err(err) => {
            err.print();
            ExitCode::FAILURE
        }
    }
}

fn inner() -> trident::domain::Result<ExitCode> {
    let Some(command) = trident::cli::input::parse()? else {
        return Ok(ExitCode::SUCCESS);
    };
    match command {
        Command::Ci(args) => trident::commands::ci(args),
        Command::Fix(args) => trident::commands::fix(args),
        Command::FixUnsafe(args) => trident::commands::fix_unsafe(args),
        Command::Full(args) => trident::commands::full(args),
        Command::InitClaude(args) => trident::embed::agents::claude(&args),
        Command::InitConfig(args) => trident::commands::init_config(&args),
        Command::InitGithook(args) => trident::embed::git::pre_commit(&args),
        Command::Lint(args) => trident::commands::lint(args),
        Command::Pitstop(args) => trident::commands::pitstop(args),
        Command::Postedit(args) => trident::commands::post_edit(args),
        Command::Precommit(args) => trident::commands::precommit(args),
        Command::Test(args) => trident::commands::test(args),
        Command::UpdateTools => trident::commands::update_tools(),
    }
}
