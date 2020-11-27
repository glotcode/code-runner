use std::path;
use crate::code_runner::non_empty_vec;


#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Assembly,
    Ats,
    Bash,
    Haskell,
    Python,
}


#[derive(Debug)]
pub struct RunInstructions {
    pub build_commands: Vec<String>,
    pub run_command: String,
}


// TODO: implement all languages
pub fn run_instructions(language: &Language, files: non_empty_vec::NonEmptyVec<path::PathBuf>) -> RunInstructions {
    let (main_file, other_files) = files.parts();
    let main_file_str = main_file.to_string_lossy();

    match language {
        Language::Assembly => {
            RunInstructions{
                build_commands: vec![
                    format!("nasm -f elf64 -o a.o {}", main_file_str),
                    "ld -o a.out a.o".to_string(),
                ],
                run_command: "./a.out".to_string(),
            }
        }

        Language::Ats => {
            RunInstructions{
                build_commands: vec![
                    format!("patscc -o a.out {} {}", main_file_str, source_files(other_files, "dats")),
                ],
                run_command: "./a.out".to_string(),
            }
        }

        Language::Bash => {
            RunInstructions{
                build_commands: vec![],
                run_command: format!("bash {}", main_file_str),
            }
        }

        Language::Haskell => {
            RunInstructions{
                build_commands: vec![],
                run_command: format!("runghc {}", main_file_str),
            }
        }

        Language::Python => {
            RunInstructions{
                build_commands: vec![],
                run_command: format!("python {}", main_file_str),
            }
        }
    }
}

fn source_files(files: Vec<path::PathBuf>, extension: &str) -> String {
    space_separated_files(filter_by_extension(files, extension))
}

fn filter_by_extension(files: Vec<path::PathBuf>, extension: &str) -> Vec<path::PathBuf> {
    files
        .into_iter()
        .filter(|file| file.extension().and_then(|s| s.to_str()) == Some(extension))
        .collect()
}

fn space_separated_files(files: Vec<path::PathBuf>) -> String {
    files
        .iter()
        .map(|file| file.to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join(" ")
}
