use std::env;
use std::result::Result::Ok;

use clap::ArgMatches;
use cli::get_cli_args;

use anyhow::Result;
use codeanalysis::start_analysis;
use colored::*;

pub mod cli;
pub mod codeanalysis;
pub mod coverage;
pub mod csharp;
pub mod js;
pub mod utils;

fn run() -> Result<()> {
    let cli_args = get_cli_args(env::args_os());

    let command = unwrap_command(cli_args);

    start_analysis(command);

    Ok(())
}

fn unwrap_command(cli_args: ArgMatches) -> utils::Command {
    let cmd_lang = cli_args.get_one::<String>("language").unwrap().clone();
    utils::Command {
        repo: cli_args.get_one::<String>("repo").unwrap().clone(),
        lang: if cmd_lang == "csharp" {
            utils::Lang::Csharp
        } else if cmd_lang == "python" {
            utils::Lang::Python
        } else if cmd_lang == "js" {
            utils::Lang::JS
        } else if cmd_lang == "rust" {
            utils::Lang::Rust
        } else {
            utils::Lang::Undefined
        },
    }
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {:#}", "Error".red(), e);
            std::process::exit(1);
        }
    }
}

#[test]
fn test_unwrap_command() {
    use clap::Arg;
    use clap::Command;

    let matches = Command::new("test")
        .arg(
            Arg::new("language")
                .long("language")
                .required(true)
                .num_args(1),
        )
        .arg(Arg::new("repo").long("repo").required(true).num_args(1))
        .get_matches_from(vec![
            "test",
            "--language",
            "rust",
            "--repo",
            "/path/to/repo",
        ]);

    let command = unwrap_command(matches);

    assert_eq!(command.repo, "/path/to/repo");
    assert_eq!(command.lang, utils::Lang::Rust);
}
