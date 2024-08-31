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

    let mut stdin_outputs = outputs(child_in, stdin_log_path)?;
    let mut stdout_outputs = outputs(stdout(), stdout_log_path)?;
    let mut stderr_outputs = outputs(stderr(), stderr_log_path)?;

    let _ = thread::scope(|s| -> Result<i32> {
        let stdin_thread = s.spawn(|| tee(stdin(), &mut stdin_outputs[..]));
        let stdout_thread = s.spawn(|| tee(child_out, &mut stdout_outputs[..]));
        let stderr_thread = s.spawn(|| tee(child_err, &mut stderr_outputs[..]));

        [stdout_thread, stderr_thread, stdin_thread]
            .into_iter()
            .flat_map(|t| t.join().map_err(|_| eyre!("thread panic")))
            .collect::<Result<()>>()?;

        Ok(1)
    });

    let code = child
        .wait()?
        .code()
        .ok_or_else(|| eyre!("process terminated by signal"))?;
    Ok(code)
}

fn outputs<W: Write + Send + 'static>(
    writer: W,
    filename: Option<&Path>,
) -> Result<Vec<Box<(dyn Write + Send)>>> {
    match filename {
        Some(filename) => {
            let file = File::create(filename)?;
            Ok(vec![Box::new(writer), Box::new(file)])
        }
        None => Ok(vec![Box::new(writer)]),
    }
}

fn tee(mut stream: impl Read, outputs: &mut [Box<dyn Write + Send>]) -> Result<()> {
    let mut buffer = [0u8; 1024];
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        for o in outputs.iter_mut() {
            o.write_all(&buffer[..n])?;
        }
    }
    Ok(())
}
