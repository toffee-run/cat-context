use std::path::PathBuf;

use bollard::Docker;

#[derive(Default)]
enum Base {
    #[default]
    Debian,
    Alpine,
    Arch,
}

enum Agent {
    ClaudeCode,
    Codex,
    Antigravity,
    Opencode,
}

#[derive(Default)]
enum Prompt {
    File(PathBuf),
    Text(String),
    #[default]
    None,
}

enum Action {
    Start {
        base: Base,
        agent: Agent,
        prompt: Prompt,
    },
    Restart {
        container: String,
        agent: Option<Agent>,
        prompt: Option<Prompt>,
        save: bool,
    },
    Stop {
        container: String,
    },
    Delete {
        container: String,
    },
}

struct Command {
    connect: Docker,
    action: Action,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {}
