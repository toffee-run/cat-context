use clap::{Parser, Subcommand};
use inquire::InquireError;

pub struct Docker;
#[derive(Clone)]
pub struct Base;
#[derive(Clone)]
pub struct Agent;

pub type Result<T> = std::result::Result<T, InquireError>;

pub enum Action {
    Start { base: Base, agent: Agent },
    Stop { container: String },
}

#[derive(Subcommand)]
pub enum ActionArgs {
    #[command(about = "запустить новый контейнер")]
    Start(StartArgs),
    #[command(about = "остановить контейнер")]
    Stop(StopArgs),
}

#[derive(clap::Args)]
pub struct StartArgs {
    #[arg(long)]
    pub base: Option<String>,

    #[arg(long)]
    pub agent: Option<String>,
}

#[derive(clap::Args)]
pub struct StopArgs {
    #[arg(long)]
    pub container: Option<String>,
}

pub async fn fill(args: ActionArgs, _ctx: &Docker) -> Result<Action> {
    match args {
        ActionArgs::Start(_args) => Ok(Action::Start {
            base: Base,
            agent: Agent,
        }),
        ActionArgs::Stop(args) => Ok(Action::Stop {
            container: args.container.unwrap_or_default(),
        }),
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
    use clap::{CommandFactory, Parser};

    #[test]
    fn parse_without_args() {
        let cli = Cli::try_parse_from(["bin", "start"]).unwrap();
        assert!(matches!(cli.cmd, ActionArgs::Start(_)));

        let cli = Cli::try_parse_from(["bin", "stop"]).unwrap();
        assert!(matches!(cli.cmd, ActionArgs::Stop(_)));
    }

    #[test]
    fn subcommands_menu() {
        let cmd = Cli::command();
        let subcommands: Vec<_> = cmd.get_subcommands().collect();
        for subcmd in subcommands {
            println!(
                "name: {}, about: {:?}",
                subcmd.get_name(),
                subcmd.get_about()
            );
        }

        let start_cmd = cmd
            .get_subcommands()
            .find(|c| c.get_name() == "start")
            .unwrap();
        assert_eq!(
            start_cmd.get_about().unwrap().to_string(),
            "запустить новый контейнер"
        );

        let stop_cmd = cmd
            .get_subcommands()
            .find(|c| c.get_name() == "stop")
            .unwrap();
        assert_eq!(
            stop_cmd.get_about().unwrap().to_string(),
            "остановить контейнер"
        );
    }
}
