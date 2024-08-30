use clap::Parser;
use eyre::Result;
use std::path::PathBuf;

/// A command runner that optionally logs the I/O streams to files.
#[derive(Debug, Parser, PartialEq)]
#[command(version)]
struct Cli {
    /// The file to log stdin to.
    #[clap(short, long, env)]
    in_file: Option<PathBuf>,

    /// The file to log stdout to.
    #[clap(short, long, env)]
    out_file: Option<PathBuf>,

    /// The file to log stderr to.
    #[clap(short, long, env)]
    err_file: Option<PathBuf>,

    /// The command to run and its arguments. A command must be specified, arguments are space delimited.
    #[clap(last = true, required = true, num_args = 1..)]
    exec: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let exec = cli.exec.iter().map(String::as_str).collect::<Vec<&str>>();
    let code = runner::run(
        exec[0],
        &exec[1..],
        cli.in_file.as_deref(),
        cli.out_file.as_deref(),
        cli.err_file.as_deref(),
    )?;

    std::process::exit(code);
}
