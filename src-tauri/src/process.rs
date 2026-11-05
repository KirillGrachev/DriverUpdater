//! Process execution with a wall-clock timeout and deadlock-safe output capture.
//!
//! <p>Both stdout and stderr are drained on dedicated threads. A child that
//! writes more than the pipe buffer therefore cannot block forever while the
//! timeout is being waited on.</p>

#[cfg(target_os = "windows")]
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

const MAX_CAPTURED_OUTPUT: usize = 4 * 1024 * 1024;

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

    let pid = child.id();
    let status = match child
        .wait_timeout(timeout)
        .map_err(|error| error.to_string())?
    {
        Some(status) => status,
        None => {
            kill_process_tree(pid);
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
        let mut chunk = [0_u8; 8192];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    if buffer.len() < MAX_CAPTURED_OUTPUT {
                        let remaining = MAX_CAPTURED_OUTPUT - buffer.len();
                        buffer.extend_from_slice(&chunk[..read.min(remaining)]);
                    }
                }
            }
        }
    }
    String::from_utf8_lossy(&buffer).into_owned()
}

/// Returns an absolute executable path under `%SystemRoot%`.
#[cfg(target_os = "windows")]
pub fn system_executable(name: &str) -> PathBuf {
    windows_directory().join("System32").join(name)
}

/// Returns an executable stored directly under `%SystemRoot%`, such as explorer.exe.
#[cfg(target_os = "windows")]
pub fn windows_executable(name: &str) -> PathBuf {
    windows_directory().join(name)
}

#[cfg(target_os = "windows")]
fn windows_directory() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
}

/// Runs PowerShell with an absolute executable path and no profile.
#[cfg(target_os = "windows")]
pub fn run_powershell(script: &str, timeout: Duration) -> Result<ProcessOutput, String> {
    let mut command = Command::new(system_executable(r"WindowsPowerShell\v1.0\powershell.exe"));
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    run(command, timeout)
}

#[cfg(target_os = "windows")]
fn kill_process_tree(pid: u32) {
    let _ = Command::new(system_executable("taskkill.exe"))
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(target_os = "windows"))]
fn kill_process_tree(_pid: u32) {}
