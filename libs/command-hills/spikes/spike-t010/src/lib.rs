use std::fmt;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use inquire::InquireError;

pub type Result<T> = std::result::Result<T, InquireError>;

pub trait Resolve<Target> {
    fn resolve(self) -> Result<Target>;
}

pub struct Docker;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, ValueEnum)]
pub enum Base {
    #[default]
    Debian,
    Alpine,
    Arch,
}

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = self.to_possible_value().expect("variant not hidden");
        f.write_str(variant.get_name())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, ValueEnum)]
pub enum Agent {
    ClaudeCode,
    Codex,
    Antigravity,
    Opencode,
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = self.to_possible_value().expect("variant not hidden");
        f.write_str(variant.get_name())
    }
}

#[derive(Clone, Debug, Default)]
pub enum Prompt {
    File(PathBuf),
    Text(String),
    #[default]
    None,
}

#[derive(clap::Args)]
#[group(multiple = false)]
pub struct PromptArgs {
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long)]
    pub no_prompt: bool,
}

impl Resolve<Prompt> for PromptArgs {
    fn resolve(self) -> Result<Prompt> {
        if let Some(path) = self.file {
            Ok(Prompt::File(path))
        } else if let Some(t) = self.text {
            Ok(Prompt::Text(t))
        } else if self.no_prompt {
            Ok(Prompt::None)
        } else {
            Ok(Prompt::None)
        }
    }
}

#[derive(clap::Args)]
pub struct SaveArgs {
    #[arg(long, conflicts_with = "no_save")]
    pub save: bool,
    #[arg(long)]
    pub no_save: bool,
}

impl Resolve<bool> for SaveArgs {
    fn resolve(self) -> Result<bool> {
        Ok(self.save)
    }
}

#[derive(Debug)]
pub enum Action {
    Start {
        base: Base,
        agent: Agent,
        prompt: Prompt,
    },
    Restart {
        base: Option<Base>,
        save: bool,
    },
}

#[derive(Subcommand)]
pub enum ActionArgs {
    #[command(about = "запустить новый контейнер")]
    Start(StartArgs),
    #[command(about = "пересоздать контейнер")]
    Restart(RestartArgs),
}

#[derive(clap::Args)]
pub struct StartArgs {
    #[arg(long)]
    pub base: Option<Base>,
    #[arg(long)]
    pub agent: Option<Agent>,
    #[command(flatten)]
    pub prompt: PromptArgs,
}

#[derive(clap::Args)]
pub struct RestartArgs {
    #[arg(long)]
    pub base: Option<Base>,
    #[command(flatten)]
    pub save: SaveArgs,
}

pub async fn fill(args: ActionArgs, _ctx: &Docker) -> Result<Action> {
    match args {
        ActionArgs::Start(args) => {
            let base = match args.base {
                Some(val) => val,
                None => {
                    let variants = Base::value_variants().to_vec();
                    inquire::Select::new("Базовый образ", variants).prompt()?
                }
            };
            let agent = match args.agent {
                Some(val) => val,
                None => {
                    let variants = Agent::value_variants().to_vec();
                    inquire::Select::new("Агент", variants).prompt()?
                }
            };
            let prompt = args.prompt.resolve()?;
            Ok(Action::Start {
                base,
                agent,
                prompt,
            })
        }
        ActionArgs::Restart(args) => {
            let base = match args.base {
                Some(val) => Some(val),
                None => {
                    struct KeepOpt(Option<Base>);
                    impl std::fmt::Display for KeepOpt {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            match &self.0 {
                                Some(v) => std::fmt::Display::fmt(v, f),
                                None => f.write_str("Базовый образ (оставить как есть)"),
                            }
                        }
                    }
                    let mut variants = vec![KeepOpt(None)];
                    variants.extend(Base::value_variants().iter().map(|v| KeepOpt(Some(*v))));
                    inquire::Select::new("Базовый образ", variants).prompt()?.0
                }
            };
            let save = args.save.resolve()?;
            Ok(Action::Restart { base, save })
        }
    }
}

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: ActionArgs,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_without_args() {
        let cli = Cli::try_parse_from(["bin", "start"]).unwrap();
        assert!(matches!(cli.cmd, ActionArgs::Start(_)));

        let cli = Cli::try_parse_from(["bin", "restart"]).unwrap();
        assert!(matches!(cli.cmd, ActionArgs::Restart(_)));
    }

    #[test]
    fn mutually_exclusive_flags() {
        let res = Cli::try_parse_from(["bin", "start", "--file", "f", "--text", "t"]);
        assert!(res.is_err());

        let res = Cli::try_parse_from(["bin", "restart", "--save", "--no-save"]);
        assert!(res.is_err());
    }
}
