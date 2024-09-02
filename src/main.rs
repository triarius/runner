use clap::Parser;
use eyre::Result;
use std::path::PathBuf;

/// A command runner that optionally logs the I/O streams to files.
#[derive(Debug, Parser, PartialEq, Eq)]
#[command(version)]
struct Args {
    /// The file to log stdin to.
    #[arg(short, long, env)]
    in_file: Option<PathBuf>,

    /// The file to log stdout to.
    #[arg(short, long, env)]
    out_file: Option<PathBuf>,

    /// The file to log stderr to.
    #[arg(short, long, env)]
    err_file: Option<PathBuf>,

    /// The command to run and its arguments. A command must be specified, arguments are space delimited.
    #[arg(last = true, required = true, num_args = 1..)]
    exec: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let exec = args.exec.iter().map(String::as_str).collect::<Vec<&str>>();
    let code = runner::run(
        exec[0],
        &exec[1..],
        args.in_file.as_deref(),
        args.out_file.as_deref(),
        args.err_file.as_deref(),
    )?;

    std::process::exit(code);
}

#[cfg(test)]
mod test {
    #[test]
    fn arg_parse() {
        use super::Args;
        use clap::Parser;
        use pretty_assertions::assert_eq;

        [
            (
                vec!["runner", "--", "echo", "hello"],
                Args {
                    in_file: None,
                    out_file: None,
                    err_file: None,
                    exec: vec!["echo".to_string(), "hello".to_string()],
                },
            ),
            (
                vec!["runner", "--in-file", "in.txt", "--", "echo", "hello"],
                Args {
                    in_file: Some("in.txt".into()),
                    out_file: None,
                    err_file: None,
                    exec: vec!["echo".to_string(), "hello".to_string()],
                },
            ),
            (
                vec!["runner", "--out-file", "out.txt", "--", "echo", "hello"],
                Args {
                    in_file: None,
                    out_file: Some("out.txt".into()),
                    err_file: None,
                    exec: vec!["echo".to_string(), "hello".to_string()],
                },
            ),
            (
                vec!["runner", "--err-file", "err.txt", "--", "echo", "hello"],
                Args {
                    in_file: None,
                    out_file: None,
                    err_file: Some("err.txt".into()),
                    exec: vec!["echo".to_string(), "hello".to_string()],
                },
            ),
            (
                vec![
                    "runner",
                    "--in-file",
                    "in.txt",
                    "--out-file",
                    "out.txt",
                    "--",
                    "echo",
                    "hello",
                ],
                Args {
                    in_file: Some("in.txt".into()),
                    out_file: Some("out.txt".into()),
                    err_file: None,
                    exec: vec!["echo".to_string(), "hello".to_string()],
                },
            ),
            (
                vec![
                    "runner",
                    "--in-file",
                    "in.txt",
                    "--out-file",
                    "out.txt",
                    "--err-file",
                    "err.txt",
                    "--",
                    "echo",
                    "hello",
                ],
                Args {
                    in_file: Some("in.txt".into()),
                    out_file: Some("out.txt".into()),
                    err_file: Some("err.txt".into()),
                    exec: vec!["echo".to_string(), "hello".to_string()],
                },
            ),
        ]
        .into_iter()
        .for_each(|(input, expected)| {
            let actual = Args::try_parse_from(input).unwrap();
            assert_eq!(actual, expected);
        });
    }
}
