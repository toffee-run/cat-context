pub mod ask;
pub mod cli;
pub mod complete;
pub mod docker;

use std::fmt;
use std::path::PathBuf;

use bollard::Docker;
use clap::ValueEnum;
use inquire::InquireError;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, ValueEnum)]
pub enum Base {
    #[default]
    Debian,
    Alpine,
    Arch,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, ValueEnum)]
pub enum Agent {
    ClaudeCode,
    Codex,
    Antigravity,
    Opencode,
}

#[derive(Clone, Debug, Default)]
pub enum Prompt {
    File(PathBuf),
    Text(String),
    #[default]
    None,
}

#[derive(Debug)]
pub enum Action {
    Start {
        base: Base,
        agent: Agent,
        prompt: Prompt,
    },
    Restart {
        container: String,
        base: Option<Base>,
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

pub struct Command {
    pub connect: Docker,
    pub action: Action,
}

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = self.to_possible_value().expect("вариант не скрыт");
        f.write_str(variant.get_name())
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = self.to_possible_value().expect("вариант не скрыт");
        f.write_str(variant.get_name())
    }
}

pub async fn run() -> u8 {
    exit_code(cli::command().await)
}

pub fn exit_code(result: ask::Result<Command>) -> u8 {
    match result {
        Ok(command) => {
            let _ = command;
            0
        }
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => 130,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}
