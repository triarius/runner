use clap::Parser;
use eyre::Result;
use std::{path::PathBuf, str::FromStr};

/// A command runner that optionally logs the I/O streams to files.
#[derive(Debug, Parser, PartialEq)]
#[command(version)]
struct Cli {
    /// The file to log stdin to.
    #[clap(short, long, env)]
    in_file: Option<String>,

    /// The file to log stdout to.
    #[clap(short, long, env)]
    out_file: Option<String>,

    /// The file to log stderr to.
    #[clap(short, long, env)]
    err_file: Option<String>,

    /// The command to run and its arguments. A command must be specified, arguments are space delimited.
    #[clap(last = true)]
    exec: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let exec = cli.exec.iter().map(String::as_str).collect::<Vec<&str>>();
    if exec.is_empty() {
        use clap::CommandFactory;
        eprintln!("EXEC must have at least one argument.");
        eprintln!("{}", Cli::command().render_long_help());
        std::process::exit(1);
    }

    let in_file = cli
        .in_file
        .as_ref()
        .map(|s| PathBuf::from_str(s))
        .transpose()?;
    let out_file = cli
        .out_file
        .as_ref()
        .map(|s| PathBuf::from_str(s))
        .transpose()?;
    let err_file = cli
        .err_file
        .as_ref()
        .map(|s| PathBuf::from_str(s))
        .transpose()?;

    let code = runner::run(
        &exec[0],
        &exec[1..],
        in_file.as_deref(),
        out_file.as_deref(),
        err_file.as_deref(),
    )?;

    std::process::exit(code);
}
