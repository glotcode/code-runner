mod code_runner;

use std::io;
use std::fs;
use std::fmt;
use std::env;
use std::time;
use std::path;
use std::process;
use std::path::Path;

use crate::code_runner::cmd;
use crate::code_runner::language;
use crate::code_runner::non_empty_vec;


fn main() {
    let _ = start().map_err(handle_error);
}

fn handle_error(error: Error) {
    match error {
        // Print RunResult if it's a compile error
        Error::Compile(err) => {
            let run_result = to_error_result(err);
            let _ = serde_json::to_writer(io::stdout(), &run_result)
                .map_err(Error::SerializeRunResult)
                .map_err(handle_error);
        }

        _ => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
}


fn start() -> Result<(), Error> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let args = env::args().collect();

    let run_request = parse_request(stdin)?;

    let work_path = match work_path_from_args(args) {
        Some(path) => {
            path
        }

        None => {
            default_work_path()?
        }
    };

    // Some languages has a bootstrap file
    let bootstrap_file = Path::new("/bootstrap.tar.gz");

    if bootstrap_file.exists() {
        unpack_bootstrap_file(&work_path, &bootstrap_file)?;
    }

    let files = run_request.files
        .into_iter()
        .map(|file| file_from_request_file(&work_path, file))
        .collect::<Result<_, _>>()?;

    for file in &files {
        write_file(file)?;
    }

    let run_result = match run_request.command {
        Some(command) if !command.is_empty() => {
            run(&work_path, &command, run_request.stdin)
        }

        Some(_) | None => {
            run_default(&work_path, run_request.language, files, run_request.stdin)?
        }
    };

    serde_json::to_writer(stdout, &run_result)
        .map_err(Error::SerializeRunResult)
}


#[derive(serde::Serialize, Debug)]
struct RunResult {
    stdout: String,
    stderr: String,
    error: String,
}

fn to_success_result(output: cmd::SuccessOutput) -> RunResult {
    RunResult{
        stdout: output.stdout,
        stderr: output.stderr,
        error: "".to_string(),
    }
}

fn to_error_result(error: cmd::Error) -> RunResult {
    match error {
        cmd::Error::Output(cmd::OutputError::ExitFailure(output)) => {
            RunResult{
                stdout: output.stdout,
                stderr: output.stderr,
                error: match output.exit_code {
                    Some(exit_code) => {
                        format!("Exit code: {}", exit_code)
                    }

                    None => {
                        "".to_string()
                    }
                }
            }
        }

        _ => {
            RunResult{
                stdout: "".to_string(),
                stderr: "".to_string(),
                error: format!("{}", error),
            }
        }
    }
}


#[derive(serde::Deserialize, Debug)]
struct RunRequest {
    language: language::Language,
    files: Vec<RequestFile>,
    stdin: Option<String>,
    command: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct RequestFile {
    name: String,
    content: String,
}


#[derive(Debug)]
struct File {
    path: path::PathBuf,
    content: String,
}

fn file_from_request_file(base_path: &path::Path, file: RequestFile) -> Result<File, Error> {
    err_if_false(!file.name.is_empty(), Error::EmptyFileName())?;
    err_if_false(!file.content.is_empty(), Error::EmptyFileContent())?;

    Ok(File{
        path: base_path.join(file.name),
        content: file.content,
    })
}

fn parse_request<R: io::Read>(reader: R) -> Result<RunRequest, Error> {
    serde_json::from_reader(reader)
        .map_err(Error::ParseRequest)
}

fn work_path_from_args(arguments: Vec<String>) -> Option<path::PathBuf> {
    let args = arguments.iter()
        .map(|s| s.as_ref())
        .collect::<Vec<&str>>();

    match &args[1..] {
        ["--path", path] => {
            Some(path::PathBuf::from(path))
        }

        _ => {
            None
        }
    }
}

fn default_work_path() -> Result<path::PathBuf, Error> {
    let duration = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map_err(Error::GetTimestamp)?;

    let name = format!("glot-{}", duration.as_secs());

    Ok(env::temp_dir().join(name))
}

fn unpack_bootstrap_file(work_path: &path::Path, bootstrap_file: &path::Path) -> Result<(), Error> {
    cmd::run(cmd::Options{
        work_path: work_path.to_path_buf(),
        command: format!("tar -zxf {}", bootstrap_file.to_string_lossy()),
        stdin: None,
    })
    .map_err(Error::Bootstrap)?;

    Ok(())
}

fn write_file(file: &File) -> Result<(), Error> {
    let parent_dir = file.path.parent()
        .ok_or_else(|| Error::GetParentDir(file.path.to_path_buf()))?;

    // Create parent directories
    fs::create_dir_all(&parent_dir)
        .map_err(|err| Error::CreateParentDir(parent_dir.to_path_buf(), err))?;

    fs::write(&file.path, &file.content)
        .map_err(|err| Error::WriteFile(file.path.to_path_buf(), err))
}

fn compile(work_path: &path::Path, command: &str) -> Result<cmd::SuccessOutput, Error> {
    cmd::run(cmd::Options{
        work_path: work_path.to_path_buf(),
        command: command.to_string(),
        stdin: None,
    })
    .map_err(Error::Compile)
}


fn run_default(work_path: &path::Path, language: language::Language, files: Vec<File>, stdin: Option<String>) -> Result<RunResult, Error> {
    let file_paths = get_relative_file_paths(work_path, files)?;
    let run_instructions = language::run_instructions(&language, file_paths);

    for command in &run_instructions.build_commands {
        compile(work_path, command)?;
    }

    let run_result = run(work_path, &run_instructions.run_command, stdin);
    Ok(run_result)
}

fn run(work_path: &path::Path, command: &str, stdin: Option<String>) -> RunResult {
    let result = cmd::run(cmd::Options{
        work_path: work_path.to_path_buf(),
        command: command.to_string(),
        stdin
    });

    match result {
        Ok(output) => {
            to_success_result(output)
        }

        Err(err) => {
            to_error_result(err)
        }
    }
}

fn get_relative_file_paths(work_path: &path::Path, files: Vec<File>) -> Result<non_empty_vec::NonEmptyVec<path::PathBuf>, Error> {
    let names = files.into_iter()
        .map(|file| {
            let path = file.path
                .strip_prefix(work_path)
                .map_err(Error::StripWorkPath)?;

            Ok(path.to_path_buf())
        })
        .collect::<Result<Vec<path::PathBuf>, Error>>()?;

    non_empty_vec::from_vec(names)
        .ok_or(Error::NoFiles())
}


enum Error {
    ParseRequest(serde_json::Error),
    NoFiles(),
    StripWorkPath(path::StripPrefixError),
    EmptyFileName(),
    EmptyFileContent(),
    GetTimestamp(time::SystemTimeError),
    GetParentDir(path::PathBuf),
    CreateParentDir(path::PathBuf, io::Error),
    WriteFile(path::PathBuf, io::Error),
    Bootstrap(cmd::Error),
    Compile(cmd::Error),
    SerializeRunResult(serde_json::Error),
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseRequest(err) => {
                write!(f, "Failed to parse request json, {}", err)
            }

            Error::NoFiles() => {
                write!(f, "Error, no files were given")
            }

            Error::StripWorkPath(err) => {
                write!(f, "Failed to strip work path of file. {}", err)
            }

            Error::EmptyFileName() => {
                write!(f, "Error, file with empty name")
            }

            Error::EmptyFileContent() => {
                write!(f, "Error, file with empty content")
            }

            Error::GetTimestamp(err) => {
                write!(f, "Failed to get timestamp for work directory, {}", err)
            }

            Error::GetParentDir(file_path) => {
                write!(f, "Failed to get parent dir for file: '{}'", file_path.to_string_lossy())
            }

            Error::CreateParentDir(file_path, err) => {
                write!(f, "Failed to create parent dir for file '{}'. {}", file_path.to_string_lossy(), err)
            }

            Error::WriteFile(file_path, err) => {
                write!(f, "Failed to write file: '{}'. {}", file_path.to_string_lossy(), err)
            }

            Error::Bootstrap(err) => {
                write!(f, "Failed to unpack bootstrap file: {}", err)
            }

            Error::Compile(err) => {
                write!(f, "Failed to compile: {}", err)
            }

            Error::SerializeRunResult(err) => {
                write!(f, "Failed to serialize run result: {}", err)
            }
        }
    }
}


fn err_if_false<E>(value: bool, err: E) -> Result<(), E> {
    if value {
        Ok(())
    } else {
        Err(err)
    }
}
