use assert_cmd::Command;
use eyre::{eyre, Result};
use pretty_assertions::assert_eq;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use tempfile::tempdir;

#[test]
fn child_exit() -> Result<()> {
    Command::cargo_bin("runner")?
        .args(["--", "echo", "hello"])
        .assert()
        .success()
        .stdout("hello\n");
    Ok(())
}

#[test]
fn stdin_close() -> Result<()> {
    Command::cargo_bin("runner")?
        .args(["--", "cat"])
        .write_stdin("hello")
        .assert()
        .success()
        .stdout("hello");
    Ok(())
}

const TEMP_HEADER: &str = "Temporary File Header";

#[test]
fn child_exit_with_files() -> Result<()> {
    let mut runner = Command::cargo_bin("runner").unwrap();
    let dir = tempdir()?;

    let in_file_path = dir.path().join("in.log");
    let out_file_path = dir.path().join("out.log");
    let err_file_path = dir.path().join("err.log");

    [&in_file_path, &out_file_path, &err_file_path]
        .into_iter()
        .try_for_each(|p| create_temp_data_in_file(p))
        .unwrap();

    runner
        .args([
            "--in-file",
            in_file_path
                .to_str()
                .ok_or_else(|| eyre!("invalid in-file path"))?,
            "--out-file",
            out_file_path
                .to_str()
                .ok_or_else(|| eyre!("invalid out-file path"))?,
            "--err-file",
            err_file_path
                .to_str()
                .ok_or_else(|| eyre!("invalid err-file path"))?,
            "--",
            "cat",
        ])
        .write_stdin("hello world\n")
        .assert()
        .success()
        .stdout("hello world\n");

    let in_contents = read_file(&in_file_path)?;
    let out_contents = read_file(&out_file_path)?;
    let err_contents = read_file(&err_file_path)?;

    assert_eq!(in_contents, format!("{TEMP_HEADER}\nhello world\n"));
    assert_eq!(out_contents, format!("{TEMP_HEADER}\nhello world\n"));
    assert_eq!(err_contents, format!("{TEMP_HEADER}\n"));
    Ok(())
}

fn read_file(path: &Path) -> Result<String> {
    let mut out = String::new();
    File::open(path)?.read_to_string(&mut out)?;
    Ok(out)
}

fn create_temp_data_in_file(path: &Path) -> Result<()> {
    let mut f = File::create_new(path)?;
    writeln!(f, "{TEMP_HEADER}")?;
    Ok(())
}
