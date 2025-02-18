use std::env;
use std::result::Result::Ok;

use clap::ArgMatches;
use cli::get_cli_args;

use anyhow::Result;
use codeanalysis::start_analysis;
use colored::*;


pub mod cli;
pub mod utils;
pub mod codeanalysis;
pub mod coverage;


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
        } else if cmd_lang =="python" {
            utils::Lang::Python
        } else if cmd_lang == "js" {
            utils::Lang::JS
        } else if cmd_lang == "rust" {
            utils::Lang::Rust
        } else {
            utils::Lang::Undefined
        }
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
