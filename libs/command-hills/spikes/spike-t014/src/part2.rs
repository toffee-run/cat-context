use clap::{Parser, CommandFactory};
use inquire::Select;
use std::fmt;

use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Docker;

impl FromStr for Docker {
    type Err = String;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Docker)
    }
}

pub enum Action {
    Start,
}
#[derive(clap::Args, Clone, Debug)]
pub struct StartArgs {
    #[arg(long)]
    pub base: String,
}
#[derive(clap::Subcommand, Clone, Debug)]
pub enum ActionArgs {
    Start(StartArgs),
}

async fn fill(args: ActionArgs, _ctx: &Docker) -> command_hills::Result<Action> {
    Ok(Action::Start) // mock
}

fn endpoint(_given: Option<Docker>) -> Docker {
    Docker // mock
}

// ---------------- Generated Code ----------------

#[derive(Parser, Debug)]
#[command(name = "cat-context")] // Needs to know the app name, or uses default
pub struct Cli {
    #[arg(long, global = true, env = "DOCKER_HOST", value_name = "URL")]
    pub connect: Option<Docker>,
    #[command(subcommand)]
    pub action: Option<ActionArgs>,
}

pub struct Command {
    pub connect: Docker,
    pub action: Action,
}

#[derive(Clone)]
struct ActionChoice {
    name: String,
    label: String,
}
impl fmt::Display for ActionChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

impl Command {
    pub async fn parse_and_run() -> command_hills::Result<Self> {
        let cli = Cli::parse();
        
        let connect = endpoint(cli.connect); // called because `with = endpoint`
        
        let args = match cli.action {
            Some(args) => args,
            None => {
                let choices: Vec<_> = Cli::command()
                    .get_subcommands()
                    .map(|cmd| ActionChoice {
                        name: cmd.get_name().to_owned(),
                        label: cmd.get_about().map(|a| a.to_string()).unwrap_or_else(|| cmd.get_name().to_owned()),
                    })
                    .collect();
                
                // Problem: macro doesn't know the prompt text "Что делать?"
                let chosen = Select::new("Action:", choices).prompt()?;
                
                // Mechanical trick to parse just the subcommand
                // Problem: need the binary name "cat-context" or something generic like "app"
                let parsed = Cli::try_parse_from(["app", &chosen.name])
                    .expect("subcommand should parse without args");
                parsed.action.expect("subcommand must be present")
            }
        };

        // Problem: macro doesn't know `fill` takes `&connect` specifically, unless it assumes all other fields are context?
        let action = fill(args, &connect).await?;

        Ok(Self { connect, action })
    }
}
