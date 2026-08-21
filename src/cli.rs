use std::path::{Path, PathBuf};

use bollard::Docker;
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::CompleteEnv;
use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter, PathCompleter};

use crate::ask;
use crate::{Action, Agent, Base, Command, Prompt, complete};

#[derive(Parser, Debug)]
#[command(name = "cat-context", version)]
struct Cli {
    #[arg(
        long,
        global = true,
        env = "DOCKER_HOST",
        value_name = "URL",
        value_parser = Docker::connect_with_host,
        add = ArgValueCandidates::new(complete::endpoints),
    )]
    connect: Option<Docker>,

    #[command(subcommand)]
    action: Option<ActionArgs>,
}

#[derive(Subcommand, Debug)]
enum ActionArgs {
    #[command(about = "запустить новый контейнер")]
    Start(StartArgs),
    #[command(about = "пересоздать контейнер")]
    Restart(RestartArgs),
    #[command(about = "остановить контейнер")]
    Stop(TargetArgs),
    #[command(about = "удалить контейнер")]
    Delete(TargetArgs),
}

#[derive(Args, Debug, Default)]
struct StartArgs {
    #[arg(long, value_enum, value_name = "BASE")]
    base: Option<Base>,

    #[arg(long, value_enum, value_name = "AGENT")]
    agent: Option<Agent>,

    #[command(flatten)]
    prompt: PromptArgs,
}

#[derive(Args, Debug, Default)]
struct RestartArgs {
    #[command(flatten)]
    target: TargetArgs,

    #[arg(long, value_enum, value_name = "BASE")]
    base: Option<Base>,

    #[command(flatten)]
    prompt: PromptArgs,

    #[arg(long)]
    save: bool,

    #[arg(long, conflicts_with = "save")]
    no_save: bool,
}

#[derive(Args, Debug, Default)]
struct TargetArgs {
    #[arg(
        long,
        value_name = "NAME",
        add = ArgValueCompleter::new(complete::containers),
    )]
    container: Option<String>,
}

#[derive(Args, Debug, Default)]
#[group(multiple = false)]
struct PromptArgs {
    #[arg(
        long,
        value_name = "FILE",
        value_parser = markdown_path,
        add = ArgValueCompleter::new(PathCompleter::any().filter(is_visitable)),
    )]
    file: Option<PathBuf>,

    #[arg(long, value_name = "TEXT")]
    text: Option<String>,

    #[arg(long)]
    no_prompt: bool,
}

#[cfg(test)]
mod argument_tests {
    use super::*;

    fn action_of(args: &[&str]) -> ActionArgs {
        Cli::try_parse_from(args)
            .expect("аргументы разбираются")
            .action
            .expect("подкоманда задана")
    }

    fn rejects(args: &[&str]) -> bool {
        Cli::try_parse_from(args).is_err()
    }

    #[test]
    fn every_start_argument_is_optional() {
        let ActionArgs::Start(args) = action_of(&["cat-context", "start"]) else {
            panic!("ожидался start");
        };

        assert!(args.base.is_none());
        assert!(args.agent.is_none());
        assert!(args.prompt.file.is_none());
        assert!(args.prompt.text.is_none());
        assert!(!args.prompt.no_prompt);
    }

    #[test]
    fn start_reads_base_agent_and_file() {
        let ActionArgs::Start(args) = action_of(&[
            "cat-context",
            "start",
            "--base",
            "alpine",
            "--agent",
            "codex",
            "--file",
            "plan.md",
        ]) else {
            panic!("ожидался start");
        };

        assert_eq!(args.base, Some(Base::Alpine));
        assert_eq!(args.agent, Some(Agent::Codex));
        assert_eq!(args.prompt.file, Some(PathBuf::from("plan.md")));
    }

    #[test]
    fn restart_reads_container_base_and_save() {
        let ActionArgs::Restart(args) = action_of(&[
            "cat-context",
            "restart",
            "--container",
            "cat-arch-codex",
            "--base",
            "debian",
            "--save",
        ]) else {
            panic!("ожидался restart");
        };

        assert_eq!(args.target.container.as_deref(), Some("cat-arch-codex"));
        assert_eq!(args.base, Some(Base::Debian));
        assert!(args.save);
        assert!(!args.no_save);
    }

    #[test]
    fn stop_and_delete_share_the_target() {
        let ActionArgs::Stop(stop) = action_of(&["cat-context", "stop", "--container", "one"])
        else {
            panic!("ожидался stop");
        };
        let ActionArgs::Delete(delete) =
            action_of(&["cat-context", "delete", "--container", "two"])
        else {
            panic!("ожидался delete");
        };

        assert_eq!(stop.container.as_deref(), Some("one"));
        assert_eq!(delete.container.as_deref(), Some("two"));
    }

    #[test]
    fn prompt_sources_exclude_each_other() {
        assert!(rejects(&[
            "cat-context",
            "start",
            "--file",
            "plan.md",
            "--text",
            "привет"
        ]));
        assert!(rejects(&[
            "cat-context",
            "start",
            "--text",
            "привет",
            "--no-prompt"
        ]));
    }

    #[test]
    fn save_excludes_no_save() {
        assert!(rejects(&[
            "cat-context",
            "restart",
            "--container",
            "one",
            "--save",
            "--no-save"
        ]));
    }

    #[test]
    fn file_argument_takes_markdown_only() {
        assert!(rejects(&["cat-context", "start", "--file", "notes.txt"]));
        assert!(!rejects(&["cat-context", "start", "--file", "notes.md"]));
    }

    #[test]
    fn short_flags_and_help_are_gone() {
        assert!(rejects(&["cat-context", "start", "-b", "alpine"]));
        assert!(rejects(&["cat-context", "--help"]));
        assert!(rejects(&["cat-context", "start", "-h"]));
        assert!(rejects(&["cat-context", "help"]));
    }

    #[test]
    fn unknown_scheme_is_rejected() {
        assert!(rejects(&["cat-context", "--connect", "ftp://box", "stop"]));
        assert!(!rejects(&["cat-context", "--connect", "ssh://box", "stop"]));
    }
}

impl PromptArgs {
    fn into_prompt(self) -> Option<Prompt> {
        if let Some(path) = self.file {
            return Some(Prompt::File(path));
        }
        if let Some(text) = self.text {
            return Some(Prompt::Text(text));
        }
        if self.no_prompt {
            return Some(Prompt::None);
        }
        None
    }
}

#[cfg(test)]
mod into_prompt_tests {
    use super::*;

    #[test]
    fn prompt_arguments_turn_into_a_prompt() {
        let file = PromptArgs {
            file: Some(PathBuf::from("plan.md")),
            ..PromptArgs::default()
        };
        let text = PromptArgs {
            text: Some("привет".to_owned()),
            ..PromptArgs::default()
        };
        let empty = PromptArgs {
            no_prompt: true,
            ..PromptArgs::default()
        };

        assert!(matches!(file.into_prompt(), Some(Prompt::File(_))));
        assert!(matches!(text.into_prompt(), Some(Prompt::Text(_))));
        assert!(matches!(empty.into_prompt(), Some(Prompt::None)));
        assert!(PromptArgs::default().into_prompt().is_none());
    }
}

pub fn complete() {
    CompleteEnv::with_factory(Cli::command).complete();
}

pub async fn command() -> ask::Result<Command> {
    let cli = Cli::parse();

    let connect = endpoint(cli.connect);

    let args = match cli.action {
        Some(args) => args,
        None => ask_action()?,
    };

    let action = fill(args, &connect).await?;
    Ok(Command { connect, action })
}

fn endpoint(given: Option<Docker>) -> Docker {
    match given {
        Some(docker) => docker,
        None => Docker::connect_with_defaults().expect("подключение по умолчанию"),
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::fixtures::offline_docker;
    use super::*;

    #[test]
    fn a_given_endpoint_is_taken_as_is() {
        let given = offline_docker();

        assert_eq!(
            format!("{:?}", endpoint(Some(given))),
            format!("{:?}", offline_docker())
        );
    }
}

async fn fill(args: ActionArgs, docker: &Docker) -> ask::Result<Action> {
    match args {
        ActionArgs::Start(args) => start(args),
        ActionArgs::Restart(args) => restart(args, docker).await,
        ActionArgs::Stop(args) => stop(args, docker).await,
        ActionArgs::Delete(args) => delete(args, docker).await,
    }
}

#[cfg(test)]
mod fill_tests {
    use super::fixtures::offline_docker;
    use super::*;

    async fn action_from(args: &[&str]) -> Action {
        let action = Cli::try_parse_from(args)
            .expect("аргументы разбираются")
            .action
            .expect("подкоманда задана");

        fill(action, &offline_docker())
            .await
            .expect("вопросов не осталось")
    }

    #[tokio::test]
    async fn full_start_arguments_need_no_questions() {
        let action = action_from(&[
            "cat-context",
            "start",
            "--base",
            "alpine",
            "--agent",
            "codex",
            "--text",
            "привет",
        ])
        .await;

        let Action::Start {
            base,
            agent,
            prompt,
        } = action
        else {
            panic!("ожидался Start");
        };

        assert_eq!(base, Base::Alpine);
        assert_eq!(agent, Agent::Codex);
        assert!(matches!(prompt, Prompt::Text(text) if text == "привет"));
    }

    #[tokio::test]
    async fn start_accepts_a_markdown_prompt() {
        let action = action_from(&[
            "cat-context",
            "start",
            "--base",
            "arch",
            "--agent",
            "opencode",
            "--file",
            "plan.md",
        ])
        .await;

        let Action::Start { prompt, .. } = action else {
            panic!("ожидался Start");
        };

        assert!(matches!(prompt, Prompt::File(path) if path == PathBuf::from("plan.md")));
    }

    #[tokio::test]
    async fn full_restart_arguments_need_no_questions() {
        let action = action_from(&[
            "cat-context",
            "restart",
            "--container",
            "cat-arch-codex",
            "--base",
            "debian",
            "--no-prompt",
            "--no-save",
        ])
        .await;

        let Action::Restart {
            container,
            base,
            prompt,
            save,
        } = action
        else {
            panic!("ожидался Restart");
        };

        assert_eq!(container, "cat-arch-codex");
        assert_eq!(base, Some(Base::Debian));
        assert!(matches!(prompt, Some(Prompt::None)));
        assert!(!save);
    }

    #[tokio::test]
    async fn restart_takes_a_new_prompt_and_keeps_chats() {
        let action = action_from(&[
            "cat-context",
            "restart",
            "--container",
            "cat-arch-codex",
            "--base",
            "alpine",
            "--text",
            "новый",
            "--save",
        ])
        .await;

        let Action::Restart {
            base, prompt, save, ..
        } = action
        else {
            panic!("ожидался Restart");
        };

        assert_eq!(base, Some(Base::Alpine));
        assert!(matches!(prompt, Some(Prompt::Text(text)) if text == "новый"));
        assert!(save);
    }

    #[tokio::test]
    async fn stop_and_delete_carry_the_container() {
        let stopped = action_from(&["cat-context", "stop", "--container", "cat-arch-codex"]).await;
        let deleted =
            action_from(&["cat-context", "delete", "--container", "cat-debian-codex"]).await;

        assert!(matches!(stopped, Action::Stop { container } if container == "cat-arch-codex"));
        assert!(matches!(deleted, Action::Delete { container } if container == "cat-debian-codex"));
    }
}

async fn stop(args: TargetArgs, docker: &Docker) -> ask::Result<Action> {
    let container = ask::container(docker, args.container, "Какой контейнер остановить?").await?;
    Ok(Action::Stop { container })
}

async fn delete(args: TargetArgs, docker: &Docker) -> ask::Result<Action> {
    let container = ask::container(docker, args.container, "Какой контейнер удалить?").await?;
    Ok(Action::Delete { container })
}

fn ask_action() -> ask::Result<ActionArgs> {
    let chosen = ask::action(action_choices())?;

    let args = Cli::try_parse_from(["cat-context", &chosen])
        .expect("подкоманда без аргументов разбирается")
        .action
        .expect("подкоманда названа");

    Ok(args)
}

fn action_choices() -> Vec<ask::Choice> {
    Cli::command()
        .get_subcommands()
        .map(|command| ask::Choice {
            name: command.get_name().to_owned(),
            label: match command.get_about() {
                Some(about) => about.to_string(),
                None => command.get_name().to_owned(),
            },
        })
        .collect()
}

#[cfg(test)]
mod action_choices_tests {
    use super::*;

    #[test]
    fn every_subcommand_becomes_a_menu_item() {
        let choices = action_choices();
        let names: Vec<&str> = choices.iter().map(|choice| choice.name.as_str()).collect();
        let labels: Vec<&str> = choices.iter().map(|choice| choice.label.as_str()).collect();

        assert_eq!(names, vec!["start", "restart", "stop", "delete"]);
        assert_eq!(
            labels,
            vec![
                "запустить новый контейнер",
                "пересоздать контейнер",
                "остановить контейнер",
                "удалить контейнер"
            ]
        );
    }

    #[test]
    fn a_chosen_name_parses_into_empty_arguments() {
        for choice in action_choices() {
            let parsed = Cli::try_parse_from(["cat-context", &choice.name]);

            assert!(parsed.is_ok(), "{}", choice.name);
        }
    }
}

fn start(args: StartArgs) -> ask::Result<Action> {
    let base = ask::variant("Базовый образ", args.base)?;
    let agent = ask::variant("Агент", args.agent)?;

    let prompt = match args.prompt.into_prompt() {
        Some(prompt) => prompt,
        None => ask::prompt()?,
    };

    Ok(Action::Start {
        base,
        agent,
        prompt,
    })
}

async fn restart(args: RestartArgs, docker: &Docker) -> ask::Result<Action> {
    let RestartArgs {
        target,
        base,
        prompt,
        save,
        no_save,
    } = args;

    let container =
        ask::container(docker, target.container, "Какой контейнер пересоздать?").await?;
    let base = ask::variant_or_keep("Базовый образ", base)?;

    let prompt = match prompt.into_prompt() {
        Some(prompt) => Some(prompt),
        None => ask::prompt_or_keep()?,
    };

    let save = match save_flag(save, no_save) {
        Some(save) => save,
        None => ask::save()?,
    };

    Ok(Action::Restart {
        container,
        base,
        prompt,
        save,
    })
}

fn save_flag(save: bool, no_save: bool) -> Option<bool> {
    if save {
        return Some(true);
    }

    if no_save {
        return Some(false);
    }

    None
}

#[cfg(test)]
mod save_flag_tests {
    use super::*;

    #[test]
    fn save_flag_needs_an_explicit_choice() {
        assert_eq!(save_flag(true, false), Some(true));
        assert_eq!(save_flag(false, true), Some(false));
        assert_eq!(save_flag(false, false), None);
    }
}

pub fn markdown_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);

    if is_markdown(&path) {
        Ok(path)
    } else {
        Err(format!("нужен .md файл: {value}"))
    }
}

pub fn is_markdown(path: &Path) -> bool {
    match path.extension() {
        Some(extension) => extension.eq_ignore_ascii_case("md"),
        None => false,
    }
}

fn is_visitable(path: &Path) -> bool {
    path.is_dir() || is_markdown(path)
}

#[cfg(test)]
mod is_visitable_tests {
    use super::*;

    #[test]
    fn completion_visits_directories_and_markdown() {
        assert!(is_visitable(Path::new("src")));
        assert!(is_visitable(Path::new("plan.md")));
        assert!(!is_visitable(Path::new("Cargo.toml")));
    }
}

#[cfg(test)]
mod fixtures {
    use super::*;

    pub fn offline_docker() -> Docker {
        Docker::connect_with_host("tcp://127.0.0.1:1")
            .expect("клиент собран без обращения к демону")
    }
}
