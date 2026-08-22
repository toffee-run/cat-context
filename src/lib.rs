pub mod ask;
pub mod cli;
pub mod complete;
pub mod docker;

use std::fmt;
use std::path::PathBuf;

use bollard::Docker;
use clap::ValueEnum;
use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter, PathCompleter};
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

#[command_hills::group(fallback = cli::ask_prompt)]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Prompt {
    #[hill(arg(
        long,
        value_name = "FILE",
        add = ArgValueCompleter::new(PathCompleter::any().filter(cli::is_visitable))
    ))]
    File(PathBuf),
    #[hill(arg(long, value_name = "TEXT"))]
    Text(String),
    #[hill(arg(long = "no-prompt"))]
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
        #[hill(args = PromptArgs)]
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
        #[hill(args = PromptArgs)]
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

#[command_hills::root(action = Action, ask = "Что сделать?")]
#[command(name = "cat-context", version)]
#[derive(Debug)]
pub struct Command {
    #[hill(
        context,
        with = cli::endpoint,
        arg(
            long,
            global = true,
            env = "DOCKER_HOST",
            value_name = "URL",
            value_parser = Docker::connect_with_host,
            add = ArgValueCandidates::new(complete::endpoints)
        )
    )]
    pub connect: Docker,
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
    exit_code(Command::parse().await)
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
