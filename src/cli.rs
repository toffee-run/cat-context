use std::path::{Path, PathBuf};

use bollard::Docker;
use clap::CommandFactory;
use clap_complete::CompleteEnv;
use clap_complete::engine::{ArgValueCompleter, PathCompleter};
use command_hills::{Resolve, Result};

use crate::Prompt;
use crate::ask;

#[derive(clap::Args, Debug, Default)]
#[group(multiple = false)]
pub struct PromptArgs {
    #[arg(
        long,
        value_name = "FILE",
        add = ArgValueCompleter::new(PathCompleter::any().filter(is_visitable)),
    )]
    pub file: Option<PathBuf>,

    #[arg(long, value_name = "TEXT")]
    pub text: Option<String>,

    #[arg(long)]
    pub no_prompt: bool,
}

impl Resolve<Prompt> for PromptArgs {
    fn resolve(self) -> Result<Prompt> {
        if let Some(file) = self.file {
            return Ok(Prompt::File(file));
        }
        if let Some(text) = self.text {
            return Ok(Prompt::Text(text));
        }
        if self.no_prompt {
            return Ok(Prompt::None);
        }
        ask::prompt()
    }
}

impl Resolve<Option<Prompt>> for PromptArgs {
    fn resolve(self) -> Result<Option<Prompt>> {
        if let Some(file) = self.file {
            return Ok(Some(Prompt::File(file)));
        }
        if let Some(text) = self.text {
            return Ok(Some(Prompt::Text(text)));
        }
        if self.no_prompt {
            return Ok(Some(Prompt::None));
        }
        ask::prompt_or_keep()
    }
}

#[derive(clap::Args, Debug, Default)]
pub struct SaveArgs {
    #[arg(long)]
    pub save: bool,

    #[arg(long, conflicts_with = "save")]
    pub no_save: bool,
}

impl Resolve<bool> for SaveArgs {
    fn resolve(self) -> Result<bool> {
        if self.save {
            return Ok(true);
        }
        if self.no_save {
            return Ok(false);
        }
        ask::save()
    }
}

pub async fn container(given: Option<String>, docker: &Docker, message: &str) -> Result<String> {
    ask::container(docker, given, message).await
}

#[cfg(test)]
mod argument_tests {
    use super::*;
    use crate::{ActionArgs, Agent, Base, Cli};
    use clap::Parser;

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

        assert_eq!(args.container.as_deref(), Some("cat-arch-codex"));
        assert_eq!(args.base, Some(Base::Debian));
        assert!(args.save.save);
        assert!(!args.save.no_save);
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
    fn file_argument_accepts_any_path() {
        assert!(!rejects(&["cat-context", "start", "--file", "notes.txt"]));
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

#[cfg(test)]
mod resolve_prompt_tests {
    use super::*;

    #[test]
    fn explicit_prompt_arguments_resolve_into_prompt() {
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

        assert!(matches!(
            Resolve::<Prompt>::resolve(file),
            Ok(Prompt::File(_))
        ));
        assert!(matches!(
            Resolve::<Prompt>::resolve(text),
            Ok(Prompt::Text(_))
        ));
        assert!(matches!(
            Resolve::<Prompt>::resolve(empty),
            Ok(Prompt::None)
        ));
    }

    #[test]
    fn explicit_prompt_arguments_resolve_into_optional_prompt() {
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

        assert!(matches!(
            Resolve::<Option<Prompt>>::resolve(file),
            Ok(Some(Prompt::File(_)))
        ));
        assert!(matches!(
            Resolve::<Option<Prompt>>::resolve(text),
            Ok(Some(Prompt::Text(_)))
        ));
        assert!(matches!(
            Resolve::<Option<Prompt>>::resolve(empty),
            Ok(Some(Prompt::None))
        ));
    }
}

#[cfg(test)]
mod resolve_save_tests {
    use super::*;

    #[test]
    fn explicit_save_arguments_resolve_into_bool() {
        let save = SaveArgs {
            save: true,
            no_save: false,
        };
        let no_save = SaveArgs {
            save: false,
            no_save: true,
        };

        assert!(Resolve::<bool>::resolve(save).unwrap());
        assert!(!Resolve::<bool>::resolve(no_save).unwrap());
    }
}

pub fn complete() {
    CompleteEnv::with_factory(crate::Cli::command).complete();
}

pub fn endpoint(given: Option<Docker>) -> Docker {
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

#[cfg(test)]
mod fill_tests {
    use super::fixtures::offline_docker;
    use super::*;
    use crate::{Action, Agent, Base, Cli, fill};
    use clap::Parser;

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

        assert!(matches!(prompt, Prompt::File(path) if path == Path::new("plan.md")));
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
