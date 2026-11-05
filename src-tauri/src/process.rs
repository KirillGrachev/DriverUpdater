//! Process execution with a wall-clock timeout and deadlock-safe output capture.
//!
//! <p>Both stdout and stderr are drained on dedicated threads. A child that
//! writes more than the pipe buffer therefore cannot block forever while the
//! timeout is being waited on.</p>

#[cfg(target_os = "windows")]
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const MAX_CAPTURED_OUTPUT: usize = 4 * 1024 * 1024;

/// Captured result of a finished child process.
pub struct ProcessOutput {
    pub success: bool,
    #[cfg(target_os = "windows")]
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
    run_inner(&mut command, timeout, None)
}

/// Runs a command and checks a cancellation flag while it is running.
pub fn run_cancellable(
    mut command: Command,
    timeout: Duration,
    cancel: &AtomicBool,
) -> Result<ProcessOutput, String> {
    run_inner(&mut command, timeout, Some(cancel))
}

fn run_inner(
    command: &mut Command,
    timeout: Duration,
    cancel: Option<&AtomicBool>,
) -> Result<ProcessOutput, String> {
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
    let started = Instant::now();
    let status = loop {
        if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            stop_child(&mut child, pid);
            return Err("process cancelled".to_string());
        }
        match child.try_wait().map_err(|error| error.to_string())? {
            Some(status) => break status,
            None if started.elapsed() >= timeout => {
                stop_child(&mut child, pid);
                return Err("process timed out".to_string());
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    };

    let stdout = stdout_reader.join().map_err(|_| "stdout reader failed")?;
    let stderr = stderr_reader.join().map_err(|_| "stderr reader failed")?;
    Ok(ProcessOutput {
        success: status.success(),
        #[cfg(target_os = "windows")]
        code: status.code(),
        stdout,
        stderr,
    })
}

fn stop_child(child: &mut std::process::Child, pid: u32) {
    kill_process_tree(pid);
    let _ = child.kill();
    let _ = child.wait();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::Arc;

    #[test]
    fn cancellation_stops_a_running_child() {
        #[cfg(target_os = "windows")]
        let command = {
            let mut command = Command::new(system_executable("cmd.exe"));
            command.args(["/C", "ping -n 20 127.0.0.1 >nul"]);
            command
        };
        #[cfg(not(target_os = "windows"))]
        let command = {
            let mut command = Command::new("sh");
            command.args(["-c", "sleep 20"]);
            command
        };

        let cancel = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancel);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            signal.store(true, Ordering::SeqCst);
        });

        match run_cancellable(command, Duration::from_secs(10), &cancel) {
            Err(error) => assert!(error.contains("cancelled")),
            Ok(_) => panic!("the child should have been cancelled"),
        }
    }
}
