use std::path;
use crate::code_runner::non_empty_vec;


#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Assembly,
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
    let (main_file, _other_files) = files.parts();

    match language {
        Language::Assembly => {
            RunInstructions{
                build_commands: vec![
                    format!("nasm -f elf64 -o a.o {}", main_file.to_string_lossy()),
                    "ld -o a.out a.o".to_string(),
                ],
                run_command: "a.out".to_string(),
            }
        }

        Language::Bash => {
            RunInstructions{
                build_commands: vec![],
                run_command: format!("bash {}", main_file.to_string_lossy()),
            }
        }

        Language::Haskell => {
            RunInstructions{
                build_commands: vec![],
                run_command: format!("runghc {}", main_file.to_string_lossy()),
            }
        }

        Language::Python => {
            RunInstructions{
                build_commands: vec![],
                run_command: format!("python {}", main_file.to_string_lossy()),
            }
        }
    }
}
