pub struct NoCtx;
use clap::{Parser, Subcommand, CommandFactory};
use command_hills::fill;

pub struct Ctx;

pub struct Start {
    pub name: String,
}

pub struct Restart {
    pub name: String,
}

#[fill(target = Start, context = NoCtx)]
pub struct StartArgs {
    #[arg(long)]
    pub name: String,
}

#[fill(target = Restart, context = Ctx)]
pub struct RestartArgs {
    #[arg(long)]
    pub name: String,
}

#[derive(Parser)]
#[command(name = "spike-t007", disable_help_flag = true, disable_help_subcommand = true)]
pub struct Cli {
    #[arg(long)]
    pub connect: Option<String>,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    #[command(about = "Start a container")]
    Start(StartArgs),

    #[command(about = "Restart a container")]
    Restart(RestartArgs),
}

fn main() {
    let app = Cli::command();
    for subcmd in app.get_subcommands() {
        println!("name: {}, about: {}", subcmd.get_name(), subcmd.get_about().unwrap_or_default());
    }

    let cli = Cli::parse();
    
    // Dispatch
    let ctx = Ctx;
    match cli.cmd {
        Cmd::Start(args) => {
            let action = args.resolve(&NoCtx);
            println!("Start: {}", action.name);
        }
        Cmd::Restart(args) => {
            let action = args.resolve(&ctx);
            println!("Restart: {}", action.name);
        }
    }
}
