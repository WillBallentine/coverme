use clap::{builder::NonEmptyStringValueParser, Arg, ArgAction, ArgMatches, Command, ValueHint};
use std::ffi::OsString;

pub fn get_cli_args<'a, I, T>(args: I) -> ArgMatches
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone + 'a,
{
    let command = build_command();
    command.get_matches_from(args)
}

fn build_command() -> Command {
    Command::new("coverall")
        .next_line_help(true)
        .hide_possible_values(true)
        .about("A commandline code coverage analyzer.")
        .help_expected(true)
        .max_term_width(80)
        .arg(
            Arg::new("repo")
                .help("Path to the repo you are wanting to check the test coverage of.")
                .required(true)
                .long("repo")
                .value_name("PTH")
                .action(ArgAction::Set),
        )
}
