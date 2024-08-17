use crate::non_empty_vec;
use serde::Deserialize;
use std::path;

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Assembly,
    Ats,
    Bash,
    C,
    Clisp,
    Clojure,
    Cobol,
    CoffeeScript,
    Cpp,
    Crystal,
    Csharp,
    D,
    Dart,
    Elixir,
    Elm,
    Erlang,
    Fsharp,
    Go,
    Groovy,
    Guile,
    Hare,
    Haskell,
    Idris,
    Java,
    JavaScript,
    Julia,
    Kotlin,
    Lua,
    Mercury,
    Nim,
    Nix,
    Ocaml,
    Pascal,
    Perl,
    Php,
    Python,
    Raku,
    Ruby,
    Rust,
    SaC,
    Scala,
    Swift,
    TypeScript,
    Zig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInstructions {
    pub build_commands: Vec<String>,
    pub run_command: String,
}

pub fn run_instructions(
    language: &Language,
    files: non_empty_vec::NonEmptyVec<path::PathBuf>,
) -> RunInstructions {
    let (main_file, other_files) = files.parts();
    let main_file_str = main_file.to_string_lossy();

    match language {
        Language::Assembly => RunInstructions {
            build_commands: vec![
                format!("nasm -f elf64 -o a.o {}", main_file_str),
                "ld -o a.out a.o".to_string(),
            ],
            run_command: "./a.out".to_string(),
        },

        Language::Ats => RunInstructions {
            build_commands: vec![format!(
                "patscc -o a.out {} {}",
                main_file_str,
                source_files(other_files, "dats")
            )],
            run_command: "./a.out".to_string(),
        },

        Language::Bash => RunInstructions {
            build_commands: vec![],
            run_command: format!("bash {}", main_file_str),
        },

        Language::C => RunInstructions {
            build_commands: vec![format!(
                "clang -o a.out -lm {} {}",
                main_file_str,
                source_files(other_files, "c")
            )],
            run_command: "./a.out".to_string(),
        },

        Language::Clisp => RunInstructions {
            build_commands: vec![],
            run_command: format!("sbcl --noinform --non-interactive --load {}", main_file_str),
        },

        Language::Clojure => RunInstructions {
            build_commands: vec![],
            run_command: format!("clj -M {}", main_file_str),
        },

        Language::Cobol => RunInstructions {
            build_commands: vec![format!(
                "cobc -x -o a.out {} {}",
                main_file_str,
                source_files(other_files, "cob")
            )],
            run_command: "./a.out".to_string(),
        },

        Language::CoffeeScript => RunInstructions {
            build_commands: vec![],
            run_command: format!("coffee {}", main_file_str),
        },

        Language::Cpp => RunInstructions {
            build_commands: vec![format!(
                "clang++ -std=c++11 -o a.out {} {}",
                main_file_str,
                source_files(other_files, "c")
            )],
            run_command: "./a.out".to_string(),
        },

        Language::Crystal => RunInstructions {
            build_commands: vec![],
            run_command: format!("crystal run {}", main_file_str),
        },

        Language::Csharp => RunInstructions {
            build_commands: vec![format!(
                "mcs -out:a.exe {} {}",
                main_file_str,
                source_files(other_files, "cs")
            )],
            run_command: "mono a.exe".to_string(),
        },

        Language::D => RunInstructions {
            build_commands: vec![format!(
                "dmd -ofa.out {} {}",
                main_file_str,
                source_files(other_files, "d")
            )],
            run_command: "./a.out".to_string(),
        },

        Language::Dart => RunInstructions {
            build_commands: vec![],
            run_command: format!("dart {}", main_file_str),
        },

        Language::Elixir => RunInstructions {
            build_commands: vec![],
            run_command: format!(
                "elixirc {} {}",
                main_file_str,
                source_files(other_files, "ex")
            ),
        },

        Language::Elm => RunInstructions {
            build_commands: vec![format!("elm make --output a.js {}", main_file_str)],
            run_command: "elm-runner a.js".to_string(),
        },

        Language::Erlang => RunInstructions {
            build_commands: filter_by_extension(other_files, "erl")
                .iter()
                .map(|file| format!("erlc {}", file.to_string_lossy()))
                .collect(),
            run_command: format!("escript {}", main_file_str),
        },

        Language::Fsharp => {
            let mut source_files = filter_by_extension(other_files, "fs");
            source_files.reverse();

            RunInstructions {
                build_commands: vec![format!(
                    "fsharpc --out:a.exe {} {}",
                    space_separated_files(source_files),
                    main_file_str
                )],
                run_command: "mono a.exe".to_string(),
            }
        }

        Language::Go => RunInstructions {
            build_commands: vec![format!("go build -o a.out {}", main_file_str)],
            run_command: "./a.out".to_string(),
        },

        Language::Groovy => RunInstructions {
            build_commands: vec![],
            run_command: format!("groovy {}", main_file_str),
        },

        Language::Guile => RunInstructions {
            build_commands: vec![],
            run_command: format!(
                "guile --no-debug --fresh-auto-compile --no-auto-compile -s {}",
                main_file_str
            ),
        },

        Language::Hare => RunInstructions {
            build_commands: vec![format!("hare build -o a.out {}", main_file_str)],
            run_command: "./a.out".to_string(),
        },

        Language::Haskell => RunInstructions {
            build_commands: vec![],
            run_command: format!("runghc {}", main_file_str),
        },

        Language::Idris => RunInstructions {
            build_commands: vec![format!("idris2 -o a.out --output-dir . {}", main_file_str)],
            run_command: "./a.out".to_string(),
        },

        Language::Java => {
            let file_stem = main_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Main");

            RunInstructions {
                build_commands: vec![format!("javac {}", main_file_str)],
                run_command: format!("java {}", titlecase_ascii(file_stem)),
            }
        }

        Language::JavaScript => RunInstructions {
            build_commands: vec![],
            run_command: format!("node {}", main_file_str),
        },

        Language::Julia => RunInstructions {
            build_commands: vec![],
            run_command: format!("julia {}", main_file_str),
        },

        Language::Kotlin => {
            let file_stem = main_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Main");

            RunInstructions {
                build_commands: vec![format!("kotlinc {}", main_file_str)],
                run_command: format!("kotlin {}Kt", titlecase_ascii(file_stem)),
            }
        }

        Language::Lua => RunInstructions {
            build_commands: vec![],
            run_command: format!("lua {}", main_file_str),
        },

        Language::Mercury => RunInstructions {
            build_commands: vec![format!(
                "mmc -o a.out {} {}",
                main_file_str,
                source_files(other_files, "m")
            )],
            run_command: "./a.out".to_string(),
        },

        Language::Nim => RunInstructions {
            build_commands: vec![],
            run_command: format!(
                "nim --hints:off --verbosity:0 compile --run {}",
                main_file_str
            ),
        },

        Language::Nix => RunInstructions {
            build_commands: vec![],
            run_command: format!("nix-instantiate --eval {}", main_file_str),
        },

        Language::Ocaml => {
            let mut source_files = filter_by_extension(other_files, "ml");
            source_files.reverse();

            RunInstructions {
                build_commands: vec![format!(
                    "ocamlc -o a.out {} {}",
                    space_separated_files(source_files),
                    main_file_str
                )],
                run_command: "./a.out".to_string(),
            }
        }

        Language::Pascal => RunInstructions {
            build_commands: vec![format!("fpc -oa.out {}", main_file_str)],
            run_command: "./a.out".to_string(),
        },

        Language::Perl => RunInstructions {
            build_commands: vec![],
            run_command: format!("perl {}", main_file_str),
        },

        Language::Php => RunInstructions {
            build_commands: vec![],
            run_command: format!("php {}", main_file_str),
        },

        Language::Python => RunInstructions {
            build_commands: vec![],
            run_command: format!("python {}", main_file_str),
        },

        Language::Raku => RunInstructions {
            build_commands: vec![],
            run_command: format!("raku {}", main_file_str),
        },

        Language::Ruby => RunInstructions {
            build_commands: vec![],
            run_command: format!("ruby {}", main_file_str),
        },

        Language::Rust => RunInstructions {
            build_commands: vec![format!("rustc -o a.out {}", main_file_str)],
            run_command: "./a.out".to_string(),
        },

        Language::SaC => RunInstructions {
            build_commands: vec![format!(
                "sac2c -t seq -o a.out {} {}",
                main_file_str,
                source_files(other_files, "c")
            )],
            run_command: "./a.out".to_string(),
        },

        Language::Scala => RunInstructions {
            build_commands: vec![format!(
                "scalac {} {}",
                main_file_str,
                source_files(other_files, "scala")
            )],
            run_command: "scala Main".to_string(),
        },

        Language::Swift => RunInstructions {
            build_commands: vec![format!(
                "swiftc -o a.out {} {}",
                main_file_str,
                source_files(other_files, "swift")
            )],
            run_command: "./a.out".to_string(),
        },

        Language::TypeScript => RunInstructions {
            build_commands: vec![format!(
                "tsc -outFile a.js {} {}",
                main_file_str,
                source_files(other_files, "ts")
            )],
            run_command: "node a.js".to_string(),
        },

        Language::Zig => RunInstructions {
            build_commands: vec![],
            run_command: format!("zig run {}", main_file_str),
        },
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

fn titlecase_ascii(s: &str) -> String {
    if !s.is_ascii() || s.len() < 2 {
        s.to_string()
    } else {
        let (head, tail) = s.split_at(1);
        format!("{}{}", head.to_ascii_uppercase(), tail)
    }
}
