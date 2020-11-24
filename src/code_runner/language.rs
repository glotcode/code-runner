use std::path;
use crate::code_runner::non_empty_vec;


#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Bash,
    Python,
}


#[derive(Debug)]
pub struct RunInstructions {
    pub build_commands: Vec<String>,
    pub run_commands: String,
}


// TODO: implement all languages
pub fn run_instructions(language: &Language, files: non_empty_vec::NonEmptyVec<path::PathBuf>) -> RunInstructions {
    let (main_file, _other_files) = files.parts();

    match language {
        Language::Python => {
            RunInstructions{
                build_commands: vec![],
                run_commands: format!("python {}", main_file.to_string_lossy()),
            }
        }

        Language::Bash => {
            RunInstructions{
                build_commands: vec![],
                run_commands: format!("bash {}", main_file.to_string_lossy()),
            }
        }
    }
}
