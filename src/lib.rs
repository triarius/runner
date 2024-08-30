use eyre::{eyre, Result};
use std::{
    fs::File,
    io::{stderr, stdin, stdout, Read, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

/// Run a process and tee its stdin, stdout, and stderr to the given files.
///
/// # Errors
/// This function returns an error if the child process cannot be spawned, or if any of the
/// file or thread operations fail.
pub fn run(
    cmd: &str,
    args: &[&str],
    stdin_log_path: Option<&Path>,
    stdout_log_path: Option<&Path>,
    stderr_log_path: Option<&Path>,
) -> Result<i32> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_out = child
        .stdout
        .take()
        .ok_or_else(|| eyre!("child stdout is not piped"))?;

    let child_err = child
        .stderr
        .take()
        .ok_or_else(|| eyre!("child stderr is not piped"))?;

    let child_in = child
        .stdin
        .take()
        .ok_or_else(|| eyre!("child stdin is not piped"))?;

    thread::scope(|s| -> Result<()> {
        s.spawn(|| tee(child_out, stdout(), stdout_log_path));
        s.spawn(|| tee(child_err, stderr(), stderr_log_path));
        s.spawn(|| tee(stdin(), child_in, stdin_log_path));

        Ok(())
    })?;

    let code = child
        .wait()?
        .code()
        .ok_or_else(|| eyre!("process terminated by signal"))?;

    Ok(code)
}

fn outputs<W: Write + 'static>(writer: W, filename: Option<&Path>) -> Result<Vec<Box<dyn Write>>> {
    match filename {
        Some(filename) => {
            let file = File::create(filename)?;
            Ok(vec![Box::new(writer), Box::new(file)])
        }
        None => Ok(vec![Box::new(writer)]),
    }
}

fn tee<W: Write + 'static>(
    mut stream: impl Read,
    output: W,
    filename: Option<&Path>,
) -> Result<()> {
    let mut buffer = [0u8; 1024];
    let mut outputs = outputs(output, filename)?;
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        for output in &mut outputs {
            output.write_all(&buffer[..n])?;
        }
    }
    Ok(())
}
