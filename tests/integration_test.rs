use assert_cmd::Command;

#[test]
fn child_exit() {
    let mut runner = Command::cargo_bin("runner").unwrap();
    let assert = runner.args(["--", "echo", "hello"]).assert();
    assert.success().stdout("hello\n");
}

#[test]
fn stdin_close() {
    let mut runner = Command::cargo_bin("runner").unwrap();
    let assert = runner.args(["--", "cat"]).write_stdin("hello").assert();
    assert.success().stdout("hello");
}
