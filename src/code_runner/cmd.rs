use std::io;
use std::io::Write;
use std::process;
use std::string;


pub struct Options {
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


#[derive(Debug)]
pub enum ExecuteError {
    Execute(io::Error),
    CaptureStdin(),
    WriteStdin(io::Error),
    WaitForChild(io::Error),
}

pub fn execute(options: Options) -> Result<process::Output, ExecuteError> {
    let mut child = process::Command::new("sh")
        .arg("-c")
        .arg(options.command)
        .stdin(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .map_err(ExecuteError::Execute)?;

    if let Some(stdin) = options.stdin {
        child.stdin
            .as_mut()
            .ok_or(ExecuteError::CaptureStdin())?
            .write_all(stdin.as_bytes())
            .map_err(ExecuteError::WriteStdin)?;
    }

    child.wait_with_output()
        .map_err(ExecuteError::WaitForChild)
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

#[derive(Debug)]
pub enum OutputError {
    ExitFailure(ErrorOutput),
    ReadStdout(string::FromUtf8Error),
    ReadStderr(string::FromUtf8Error),
}


pub fn get_output(output: process::Output) -> Result<SuccessOutput, OutputError> {
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(OutputError::ReadStdout)?;

        let stderr = String::from_utf8(output.stderr)
            .map_err(OutputError::ReadStderr)?;

        Ok(SuccessOutput{stdout, stderr})
    } else {
        let stdout = String::from_utf8(output.stdout)
            .map_err(OutputError::ReadStdout)?;

        let stderr = String::from_utf8(output.stderr)
            .map_err(OutputError::ReadStderr)?;

        let exit_code = output.status.code();

        Err(OutputError::ExitFailure(ErrorOutput{stdout, stderr, exit_code}))
    }
}
