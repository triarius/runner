#[test]
fn child_exit() {
    use assert_cmd::Command;

    let mut cmd = Command::cargo_bin("runner").unwrap();
    let assert = cmd.args(["--", "echo", "hello"]).assert();
    assert.success().stdout("hello\n");
}
