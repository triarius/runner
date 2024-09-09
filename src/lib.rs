mod flat_map_err;

use crate::flat_map_err::FlatMapErr;
use crossbeam::channel::{bounded, select, Receiver, TrySendError};
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
///
/// # Panics
/// Errors reading from stdin moving its data.
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

    // Create a channel to send data from stdin to the cancellable_tee thread.
    let (t_in, r_in) = bounded(1);

    // Read from stdin and send on a channel. We will call select! on this
    // channel in the cancellable_tee thread.
    // DO NOT join on this thread, it will cause reading stdin to block.
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            let n = stdin().read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            t_in.send((buffer, n)).unwrap();
        }
    });

    thread::scope(|s| -> Result<i32> {
        let (t_cancel, r_cancel) = bounded(1);

        // If cancellable_tee were not used here, reading from stdin will block the thread even
        // after the child process has exited. This will prevent the process from exiting.
        // So we have to cancel the tee thread when the child process exits and before
        // joining on the tee thread.
        s.spawn(move || cancellable_tee(&r_cancel, &r_in, &mut stdin_outputs[..]));
        s.spawn(|| tee(child_out, &mut stdout_outputs[..]));
        s.spawn(|| tee(child_err, &mut stderr_outputs[..]));

        let code = child
            .wait()?
            .code()
            .ok_or_else(|| eyre!("process terminated by signal"))?;

        // Cancel the tee threads. This may fail if the stdin was closed before the child process.
        t_cancel.try_send(()).flat_map_err(|e| match e {
            TrySendError::Full(()) => Err(e),
            TrySendError::Disconnected(()) => Ok(()), // If the stdin was closed, this is expected
        })?;

        Ok(code)
    })
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

/// Read from a channel and write to multiple outputs until the channel is closed or the cancel
/// channel is received.
///
/// # Errors
/// If writing to any of the outputs fails, this function returns an error.
fn cancellable_tee(
    cancel: &Receiver<()>,
    data: &Receiver<([u8; 1024], usize)>,
    outputs: &mut [Box<dyn Write + Send>],
) -> Result<()> {
    loop {
        select! {
            recv(cancel) -> _ => break,
            recv(data) -> recv => {
                if let Ok((buffer, n)) = recv {
                    if n == 0 {
                        continue;
                    }
                    for o in outputs.iter_mut() {
                        o.write_all(&buffer[..n])?;
                    }
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}
