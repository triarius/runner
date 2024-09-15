mod flat_map_err;

use crate::flat_map_err::FlatMapErr;
use core::panic;
use crossbeam::channel::{bounded, select, Receiver, TrySendError};
use eyre::{eyre, Result};
use std::{
    fs::File,
    io::{stderr, stdin, stdout, Read, Write},
    path::Path,
    process::{Command, Stdio},
    thread::{self, Scope},
};

fn io_streams<W: Write + Send + 'static>(
    writer: W,
    log_path: Option<&Path>,
) -> Result<(Stdio, Vec<Box<dyn Write + Send>>)> {
    match log_path {
        Some(path) => {
            let file = File::create(path)?;
            Ok((Stdio::piped(), vec![Box::new(writer), Box::new(file)]))
        }
        None => Ok((Stdio::inherit(), vec![Box::new(writer)])),
    }
}

/// Run a process and tee its stdin, stdout, and stderr to the given files.
///
/// # Errors
/// This function returns an error if the child process cannot be spawned, or if any of the
/// file or thread operations fail.
///
/// # Panics
/// Errors reading from stdin other than `ErrorKind::Interrupted`.
pub fn run(
    cmd: &str,
    args: &[&str],
    stdin_log_path: Option<&Path>,
    stdout_log_path: Option<&Path>,
    stderr_log_path: Option<&Path>,
) -> Result<i32> {
    let in_io = stdout_log_path.map_or(Stdio::inherit(), |_| Stdio::piped());
    let (out_io, mut out_writers) = io_streams(stdout(), stdout_log_path)?;
    let (err_io, mut err_writers) = io_streams(stderr(), stderr_log_path)?;

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(in_io)
        .stdout(out_io)
        .stderr(err_io)
        .spawn()?;

    match child.stdin.take() {
        Some(child_in) => {
            let in_file = File::create(stdin_log_path.unwrap())?;
            let mut in_writers: Vec<Box<dyn Write + Send>> =
                vec![Box::new(child_in), Box::new(in_file)];

            // Create a channel to send data from stdin to the cancellable_tee thread.
            let (t_in, r_in) = bounded(1);

            // Read from stdin and send on a channel. We will call select! on this
            // channel in the cancellable_tee thread.
            // DO NOT join on this thread, it will cause reading stdin to block.
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    match stdin().read(&mut buffer) {
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                            t_in.send((buffer, n)).unwrap();
                        }
                        Err(e) => match e.kind() {
                            std::io::ErrorKind::Interrupted => continue,
                            _ => panic!("{e:?}"),
                        },
                    }
                }
            });

            thread::scope(|s| -> Result<i32> {
                let (t_cancel, r_cancel) = bounded(1);

                // If cancellable_tee were not used here, reading from stdin will block the thread even
                // after the child process has exited. This will prevent the process from exiting.
                // So we have to cancel the tee thread when the child process exits and before
                // joining on the tee thread.
                s.spawn(move || cancellable_tee(&r_cancel, &r_in, &mut in_writers[..]));

                let code = run_with_output(s, child, &mut out_writers, &mut err_writers)?;

                // Cancel the tee threads. This may fail if the stdin was closed before the child process.
                t_cancel.try_send(()).flat_map_err(|e| match e {
                    TrySendError::Full(()) => Err(e),
                    TrySendError::Disconnected(()) => Ok(()), // If the stdin was closed, this is expected
                })?;

                Ok(code)
            })
        }
        None => thread::scope(|s| run_with_output(s, child, &mut out_writers, &mut err_writers)),
    }
}

fn run_with_output<'a>(
    s: &'a Scope<'a, '_>,
    mut child: std::process::Child,
    out_writers: &'a mut [Box<dyn Write + Send>],
    err_writers: &'a mut [Box<dyn Write + Send>],
) -> Result<i32> {
    if let Some(child_out) = child.stdout.take() {
        s.spawn(move || tee(child_out, &mut out_writers[..]));
    }
    if let Some(child_err) = child.stderr.take() {
        s.spawn(move || tee(child_err, &mut err_writers[..]));
    }

    child
        .wait()?
        .code()
        .ok_or_else(|| eyre!("process terminated by signal"))
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
