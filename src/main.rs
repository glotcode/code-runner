mod cmd;
mod request;

use request::{RequestFile, RunInstructions, RunRequest};
use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::time;

fn main() {
    if let Err(error) = start() {
        handle_error(error);
    }
}

fn handle_error(error: Error) {
    match error {
        // Print RunResult if it's a compile error
        Error::Compile(err) => {
            let run_result = to_error_result(err);
            let stdout = io::stdout();
            let _ = serde_json::to_writer(stdout.lock(), &run_result)
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
    let run_request = serde_json::from_reader(stdin.lock()).map_err(Error::ParseRequest)?;
    let args = env::args().collect::<Vec<_>>();
    let work_path = work_path_from_args(&args)?.map_or_else(default_work_path, Ok)?;

    fs::create_dir_all(&work_path).map_err(|err| Error::CreateWorkDir(work_path.clone(), err))?;

    // Runtime images may provide files needed by their build and run commands.
    let bootstrap_file = Path::new("/bootstrap.tar.gz");

    if bootstrap_file.exists() {
        unpack_bootstrap_file(&work_path, bootstrap_file)?;
    }

    let run_result = run(&work_path, run_request)?;

    let stdout = io::stdout();
    serde_json::to_writer(stdout.lock(), &run_result).map_err(Error::SerializeRunResult)
}

fn run(work_path: &Path, run_request: RunRequest) -> Result<RunResult, Error> {
    let RunRequest {
        run_instructions,
        files,
        stdin,
    } = run_request;

    validate_run_instructions(&run_instructions)?;
    validate_files(&files)?;

    let mut created_parent_dirs = HashSet::new();
    for file in files {
        write_file(work_path, file, &mut created_parent_dirs)?;
    }

    run_by_instructions(work_path, &run_instructions, stdin)
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RunResult {
    stdout: String,
    stderr: String,
    error: String,
    duration: u64,
}

fn to_success_result(output: cmd::SuccessOutput) -> RunResult {
    RunResult {
        stdout: output.stdout,
        stderr: output.stderr,
        error: String::new(),
        duration: output.duration.as_nanos() as u64,
    }
}

fn to_error_result(error: cmd::Error) -> RunResult {
    match error {
        cmd::Error::Output(cmd::OutputError::ExitFailure(output), duration) => RunResult {
            stdout: output.stdout,
            stderr: output.stderr,
            error: match output.exit_code {
                Some(exit_code) => {
                    format!("Exit code: {}", exit_code)
                }

                None => String::new(),
            },
            duration: duration.as_nanos() as u64,
        },

        _ => RunResult {
            stdout: String::new(),
            stderr: String::new(),
            error: format!("{}", error),
            duration: error.duration().as_nanos() as u64,
        },
    }
}

fn validate_run_instructions(run_instructions: &RunInstructions) -> Result<(), Error> {
    if run_instructions.run_command.trim().is_empty() {
        return Err(Error::EmptyRunCommand);
    }

    if let Some(index) = run_instructions
        .build_commands
        .iter()
        .position(|command| command.trim().is_empty())
    {
        return Err(Error::EmptyBuildCommand(index));
    }

    Ok(())
}

fn validate_files(files: &[RequestFile]) -> Result<(), Error> {
    if files.is_empty() {
        return Err(Error::NoFiles);
    }

    let mut names: HashSet<Cow<'_, Path>> = HashSet::with_capacity(files.len());

    for file in files {
        let path = Path::new(&file.name);
        let mut needs_normalization = false;

        for component in path.components() {
            match component {
                std::path::Component::Normal(_) => {}
                std::path::Component::CurDir => needs_normalization = true,
                _ => return Err(Error::InvalidFileName(file.name.clone())),
            }
        }

        if path.as_os_str().is_empty() || file.name.contains('\0') {
            return Err(Error::InvalidFileName(file.name.clone()));
        }

        let normalized_name = if needs_normalization {
            Cow::Owned(
                path.components()
                    .filter_map(|component| match component {
                        std::path::Component::Normal(part) => Some(part),
                        _ => None,
                    })
                    .collect(),
            )
        } else {
            Cow::Borrowed(path)
        };

        if normalized_name.as_os_str().is_empty() {
            return Err(Error::InvalidFileName(file.name.clone()));
        }

        if !names.insert(normalized_name) {
            return Err(Error::DuplicateFileName(file.name.clone()));
        }
    }

    Ok(())
}

fn work_path_from_args(arguments: &[String]) -> Result<Option<PathBuf>, Error> {
    match arguments {
        [_] => Ok(None),
        [_, flag, path] if flag == "--path" && !path.is_empty() => Ok(Some(PathBuf::from(path))),
        _ => Err(Error::InvalidArguments),
    }
}

fn default_work_path() -> Result<PathBuf, Error> {
    let duration = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .map_err(Error::GetTimestamp)?;

    let name = format!("glot-{}-{}", process::id(), duration.as_nanos());

    Ok(env::temp_dir().join(name))
}

fn unpack_bootstrap_file(work_path: &Path, bootstrap_file: &Path) -> Result<(), Error> {
    let command = format!("tar -zxf {}", bootstrap_file.to_string_lossy());

    cmd::run(cmd::Options {
        work_path,
        command: &command,
        stdin: None,
    })
    .map_err(Error::Bootstrap)?;

    Ok(())
}

fn write_file(
    work_path: &Path,
    file: RequestFile,
    created_parent_dirs: &mut HashSet<PathBuf>,
) -> Result<(), Error> {
    let path = work_path.join(file.name);
    let parent_dir = path.parent().expect("validated file path has a parent");

    if parent_dir != work_path && created_parent_dirs.insert(parent_dir.to_path_buf()) {
        fs::create_dir_all(parent_dir)
            .map_err(|err| Error::CreateParentDir(parent_dir.to_path_buf(), err))?;
    }

    fs::write(&path, file.content).map_err(|err| Error::WriteFile(path, err))
}

fn run_by_instructions(
    work_path: &Path,
    run_instructions: &RunInstructions,
    stdin: Option<String>,
) -> Result<RunResult, Error> {
    for command in &run_instructions.build_commands {
        cmd::run(cmd::Options {
            work_path,
            command,
            stdin: None,
        })
        .map_err(Error::Compile)?;
    }

    Ok(run_command(work_path, &run_instructions.run_command, stdin))
}

fn run_command(work_path: &Path, command: &str, stdin: Option<String>) -> RunResult {
    let result = cmd::run(cmd::Options {
        work_path,
        command,
        stdin,
    });

    match result {
        Ok(output) => to_success_result(output),

        Err(err) => to_error_result(err),
    }
}

enum Error {
    ParseRequest(serde_json::Error),
    InvalidArguments,
    EmptyRunCommand,
    EmptyBuildCommand(usize),
    NoFiles,
    InvalidFileName(String),
    DuplicateFileName(String),
    GetTimestamp(time::SystemTimeError),
    CreateWorkDir(PathBuf, io::Error),
    CreateParentDir(PathBuf, io::Error),
    WriteFile(PathBuf, io::Error),
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

            Error::InvalidArguments => {
                write!(f, "Usage: code-runner [--path <work-directory>]")
            }

            Error::EmptyRunCommand => {
                write!(f, "Run command must not be empty")
            }

            Error::EmptyBuildCommand(index) => {
                write!(f, "Build command at index {index} must not be empty")
            }

            Error::NoFiles => {
                write!(f, "At least one file is required")
            }

            Error::InvalidFileName(name) => {
                write!(f, "File name must be a non-empty relative path: '{name}'")
            }

            Error::DuplicateFileName(name) => {
                write!(f, "Duplicate file name: '{name}'")
            }

            Error::GetTimestamp(err) => {
                write!(f, "Failed to get timestamp for work directory, {}", err)
            }

            Error::CreateWorkDir(path, err) => {
                write!(
                    f,
                    "Failed to create work directory '{}'. {}",
                    path.to_string_lossy(),
                    err
                )
            }

            Error::CreateParentDir(file_path, err) => {
                write!(
                    f,
                    "Failed to create parent dir for file '{}'. {}",
                    file_path.to_string_lossy(),
                    err
                )
            }

            Error::WriteFile(file_path, err) => {
                write!(
                    f,
                    "Failed to write file: '{}'. {}",
                    file_path.to_string_lossy(),
                    err
                )
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

#[cfg(test)]
mod tests {
    use super::{
        Error, RequestFile, RunInstructions, validate_files, validate_run_instructions,
        work_path_from_args,
    };

    fn file(name: &str) -> RequestFile {
        RequestFile {
            name: name.to_string(),
            content: String::new(),
        }
    }

    #[test]
    fn accepts_empty_file_content() {
        assert!(validate_files(&[file("empty.txt")]).is_ok());
    }

    #[test]
    fn rejects_missing_files() {
        assert!(matches!(validate_files(&[]), Err(Error::NoFiles)));
    }

    #[test]
    fn rejects_empty_commands() {
        let run_instructions = RunInstructions {
            build_commands: vec!["  ".to_string()],
            run_command: "run".to_string(),
        };
        assert!(matches!(
            validate_run_instructions(&run_instructions),
            Err(Error::EmptyBuildCommand(0))
        ));

        let run_instructions = RunInstructions {
            build_commands: vec![],
            run_command: String::new(),
        };
        assert!(matches!(
            validate_run_instructions(&run_instructions),
            Err(Error::EmptyRunCommand)
        ));
    }

    #[test]
    fn rejects_file_names_outside_work_directory() {
        for name in ["/tmp/file", "../file", "dir/../../file"] {
            assert!(matches!(
                validate_files(&[file(name)]),
                Err(Error::InvalidFileName(_))
            ));
        }
    }

    #[test]
    fn rejects_duplicate_normalized_file_names() {
        for duplicate in ["./dir/file", "dir//file", "dir/file/"] {
            assert!(matches!(
                validate_files(&[file("dir/file"), file(duplicate)]),
                Err(Error::DuplicateFileName(_))
            ));
        }
    }

    #[test]
    fn rejects_invalid_arguments() {
        let args = ["code-runner".to_string(), "--unknown".to_string()];
        assert!(matches!(
            work_path_from_args(&args),
            Err(Error::InvalidArguments)
        ));
    }
}
