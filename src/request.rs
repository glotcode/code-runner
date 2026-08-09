use serde::Deserialize;

/// A language-agnostic request to build and run a collection of files.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRequest {
    pub run_instructions: RunInstructions,
    pub files: Vec<RequestFile>,
    pub stdin: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInstructions {
    pub build_commands: Vec<String>,
    pub run_command: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestFile {
    pub name: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::RunRequest;

    #[test]
    fn parses_request() {
        let request: RunRequest = serde_json::from_str(
            r#"{
                "runInstructions": {
                    "buildCommands": ["rustc -o app main.rs"],
                    "runCommand": "./app"
                },
                "files": [{"name": "main.rs", "content": "fn main() {}"}],
                "stdin": null
            }"#,
        )
        .expect("request should parse");

        assert_eq!(request.run_instructions.build_commands.len(), 1);
        assert_eq!(request.run_instructions.run_command, "./app");
        assert_eq!(request.files.len(), 1);
        assert!(request.stdin.is_none());
    }
}
