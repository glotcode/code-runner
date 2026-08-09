use std::fmt;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;
use std::thread;
use std::time::Duration;
use std::time::Instant;

pub struct Options<'a> {
    pub work_path: &'a Path,
    pub command: &'a str,
    pub stdin: Option<String>,
}

pub fn run(options: Options<'_>) -> Result<SuccessOutput, Error> {
    let now = Instant::now();
    let output = execute(options).map_err(|err| Error::Execute(err, now.elapsed()))?;
    let elapsed = now.elapsed();
    get_output(output, elapsed).map_err(|err| Error::Output(err, now.elapsed()))
}

#[derive(Debug)]
pub enum Error {
    Execute(ExecuteError, Duration),
    Output(OutputError, Duration),
}

impl Error {
    pub fn duration(&self) -> Duration {
        match self {
            Error::Execute(_, duration) => *duration,
            Error::Output(_, duration) => *duration,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Execute(err, _) => {
                write!(f, "Error while executing command. {}", err)
            }

            Error::Output(err, _) => {
                write!(f, "Error in output from command. {}", err)
            }
        }
    }
}

#[derive(Debug)]
pub enum ExecuteError {
    Execute(io::Error),
    CaptureStdin(),
    WriteStdin(io::Error),
    StdinWriterPanicked(),
    WaitForChild(io::Error),
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExecuteError::Execute(err) => {
                write!(f, "{}", err)
            }

            ExecuteError::CaptureStdin() => {
                write!(f, "Failed to capture stdin.")
            }

            ExecuteError::WriteStdin(err) => {
                write!(f, "Failed to write to stdin. {}", err)
            }

            ExecuteError::StdinWriterPanicked() => {
                write!(f, "Stdin writer thread panicked.")
            }

            ExecuteError::WaitForChild(err) => {
                write!(f, "Failed while waiting for child. {}", err)
            }
        }
    }
}

pub fn execute(options: Options<'_>) -> Result<process::Output, ExecuteError> {
    let mut child = process::Command::new("sh")
        .arg("-c")
        .arg(options.command)
        .current_dir(options.work_path)
        .stdin(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .map_err(ExecuteError::Execute)?;

    let stdin_writer = options.stdin.map(|input| {
        let mut stdin = child.stdin.take().ok_or(ExecuteError::CaptureStdin())?;

        Ok(thread::spawn(move || stdin.write_all(input.as_bytes())))
    });

    let output = child
        .wait_with_output()
        .map_err(ExecuteError::WaitForChild)?;

    if let Some(writer) = stdin_writer {
        let write_result = writer?
            .join()
            .map_err(|_| ExecuteError::StdinWriterPanicked())?;

        if let Err(err) = write_result
            && err.kind() != io::ErrorKind::BrokenPipe
        {
            return Err(ExecuteError::WriteStdin(err));
        }
    }

    Ok(output)
}

#[derive(Debug)]
pub struct SuccessOutput {
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct ErrorOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl fmt::Display for ErrorOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut messages = Vec::new();

        if let Some(code) = self.exit_code {
            messages.push(format!("code: {}", code));
        }

        if !self.stdout.is_empty() {
            messages.push(format!("stdout: {}", self.stdout))
        }

        if !self.stderr.is_empty() {
            messages.push(format!("stderr: {}", self.stderr))
        }

        write!(f, "{}", messages.join(", "))
    }
}

#[derive(Debug)]
pub enum OutputError {
    ExitFailure(ErrorOutput),
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputError::ExitFailure(err) => {
                write!(f, "Exited with non-zero exit code. {}", err)
            }
        }
    }
}

pub fn get_output(
    output: process::Output,
    duration: Duration,
) -> Result<SuccessOutput, OutputError> {
    let stdout = decode_output(output.stdout);
    let stderr = decode_output(output.stderr);

    if output.status.success() {
        Ok(SuccessOutput {
            stdout,
            stderr,
            duration,
        })
    } else {
        let exit_code = output.status.code();

        Err(OutputError::ExitFailure(ErrorOutput {
            stdout,
            stderr,
            exit_code,
        }))
    }
}

fn decode_output(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
}

#[cfg(test)]
mod tests {
    use super::{Options, execute, get_output};
    use std::process;
    use std::time::Duration;

    #[test]
    fn handles_output_produced_before_stdin_is_consumed() {
        let input = "x".repeat(256 * 1024);
        let work_path = std::env::temp_dir();
        let output = execute(Options {
            work_path: &work_path,
            command: "head -c 262144 /dev/zero; wc -c",
            stdin: Some(input),
        })
        .expect("command should not deadlock");

        assert!(output.status.success());
        assert!(output.stdout.ends_with(b"262144\n"));
    }

    #[test]
    fn preserves_non_utf8_output_lossily() {
        let output = process::Command::new("sh")
            .args(["-c", "printf '\\377'"])
            .output()
            .expect("shell should run");

        let output = get_output(output, Duration::ZERO).expect("command should succeed");
        assert_eq!(output.stdout, "\u{fffd}");
    }

    #[test]
    fn allows_commands_to_close_stdin_early() {
        let work_path = std::env::temp_dir();
        let output = execute(Options {
            work_path: &work_path,
            command: "exit 0",
            stdin: Some("x".repeat(256 * 1024)),
        })
        .expect("a closed stdin pipe should not fail the command");

        assert!(output.status.success());
    }
}
