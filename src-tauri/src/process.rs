//! Process execution with a wall-clock timeout and deadlock-safe output capture.
//!
//! <p>Both stdout and stderr are drained on dedicated threads. A child that
//! writes more than the pipe buffer therefore cannot block forever while the
//! timeout is being waited on.</p>

use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

/// Captured result of a finished child process.
pub struct ProcessOutput {
    pub success: bool,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Runs a command with a timeout, returning its combined text output.
///
/// @param command command to spawn; stdio is replaced by the helper
/// @param timeout maximum wall-clock time for the child process
/// @returns captured output, or an error for spawn/timeout/non-UTF8 output
pub fn run(mut command: Command, timeout: Duration) -> Result<ProcessOutput, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start process: {error}"))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_reader = std::thread::spawn(move || read_pipe(stdout));
    let stderr_reader = std::thread::spawn(move || read_pipe(stderr));

    let status = match child
        .wait_timeout(timeout)
        .map_err(|error| error.to_string())?
    {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err("process timed out".to_string());
        }
    };

    let stdout = stdout_reader.join().map_err(|_| "stdout reader failed")?;
    let stderr = stderr_reader.join().map_err(|_| "stderr reader failed")?;
    Ok(ProcessOutput {
        success: status.success(),
        code: status.code(),
        stdout,
        stderr,
    })
}

fn read_pipe(pipe: Option<impl std::io::Read>) -> String {
    let mut buffer = Vec::new();
    if let Some(mut pipe) = pipe {
        let _ = pipe.read_to_end(&mut buffer);
    }
    String::from_utf8_lossy(&buffer).into_owned()
}
