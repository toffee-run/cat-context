pub mod ask;
pub mod cli;
pub mod complete;
pub mod docker;

use std::fmt;
use std::path::PathBuf;

use bollard::Docker;
use clap::ValueEnum;
use clap_complete::engine::ArgValueCompleter;
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

#[command_hills::commands(context = Docker)]
#[derive(Debug)]
pub enum Action {
    #[hill(about = "запустить новый контейнер")]
    Start {
        #[hill(ask = "Базовый образ", arg(long, value_enum, value_name = "BASE"))]
        base: Base,
        #[hill(ask = "Агент", arg(long, value_enum, value_name = "AGENT"))]
        agent: Agent,
        #[hill(args = cli::PromptArgs)]
        prompt: Prompt,
    },
    #[hill(about = "пересоздать контейнер")]
    Restart {
        #[hill(
            with = cli::container,
            message = "Какой контейнер пересоздать?",
            arg(
                long,
                value_name = "NAME",
                add = ArgValueCompleter::new(complete::containers)
            )
        )]
        container: String,
        #[hill(keep = "Базовый образ", arg(long, value_enum, value_name = "BASE"))]
        base: Option<Base>,
        #[hill(args = cli::PromptArgs)]
        prompt: Option<Prompt>,
        #[hill(args = cli::SaveArgs)]
        save: bool,
    },
    #[hill(about = "остановить контейнер")]
    Stop {
        #[hill(
            with = cli::container,
            message = "Какой контейнер остановить?",
            arg(
                long,
                value_name = "NAME",
                add = ArgValueCompleter::new(complete::containers)
            )
        )]
        container: String,
    },
    #[hill(about = "удалить контейнер")]
    Delete {
        #[hill(
            with = cli::container,
            message = "Какой контейнер удалить?",
            arg(
                long,
                value_name = "NAME",
                add = ArgValueCompleter::new(complete::containers)
            )
        )]
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
