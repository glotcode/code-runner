use std::fmt;
use std::io;
use std::io::Write;
use std::path;
use std::process;
use std::string;

pub struct Options {
    pub work_path: path::PathBuf,
    pub command: String,
    pub stdin: Option<String>,
}

pub fn run(options: Options) -> Result<SuccessOutput, Error> {
    let output = execute(options).map_err(Error::Execute)?;
    get_output(output).map_err(Error::Output)
}

#[derive(Debug)]
pub enum Error {
    Execute(ExecuteError),
    Output(OutputError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Execute(err) => {
                write!(f, "Error while executing command. {}", err)
            }

            Error::Output(err) => {
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

            ExecuteError::WaitForChild(err) => {
                write!(f, "Failed while waiting for child. {}", err)
            }
        }
    }
}

pub fn execute(options: Options) -> Result<process::Output, ExecuteError> {
    let mut child = process::Command::new("sh")
        .arg("-c")
        .arg(options.command)
        .current_dir(&options.work_path)
        .stdin(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .map_err(ExecuteError::Execute)?;

    if let Some(stdin) = options.stdin {
        child
            .stdin
            .as_mut()
            .ok_or(ExecuteError::CaptureStdin())?
            .write_all(stdin.as_bytes())
            .map_err(ExecuteError::WriteStdin)?;
    }

    child.wait_with_output().map_err(ExecuteError::WaitForChild)
}

#[derive(Debug)]
pub struct SuccessOutput {
    pub stdout: String,
    pub stderr: String,
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
    ReadStdout(string::FromUtf8Error),
    ReadStderr(string::FromUtf8Error),
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputError::ExitFailure(err) => {
                write!(f, "Exited with non-zero exit code. {}", err)
            }

            OutputError::ReadStdout(err) => {
                write!(f, "Failed to read stdout. {}", err)
            }

            OutputError::ReadStderr(err) => {
                write!(f, "Failed to read stderr. {}", err)
            }
        }
    }
}

pub fn get_output(output: process::Output) -> Result<SuccessOutput, OutputError> {
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).map_err(OutputError::ReadStdout)?;

        let stderr = String::from_utf8(output.stderr).map_err(OutputError::ReadStderr)?;

        Ok(SuccessOutput { stdout, stderr })
    } else {
        let stdout = String::from_utf8(output.stdout).map_err(OutputError::ReadStdout)?;

        let stderr = String::from_utf8(output.stderr).map_err(OutputError::ReadStderr)?;

        let exit_code = output.status.code();

        Err(OutputError::ExitFailure(ErrorOutput {
            stdout,
            stderr,
            exit_code,
        }))
    }
}
