pub fn is_markdown(path: &std::path::Path) -> bool { match path.extension() { Some(ext) => ext.eq_ignore_ascii_case("md"), None => false, } }

pub fn is_visitable(path: &std::path::Path) -> bool { path.is_dir() || is_markdown(path) }
use clap::{Parser, ValueEnum, CommandFactory};
use command_hills::fill;
use command_hills::__private::{ArgValueCompleter, CompletionCandidate};
use clap_complete::engine::PathCompleter;
use std::path::PathBuf;
use std::ffi::OsStr;
use clap_complete::env::CompleteEnv;

#[derive(Clone, Copy, PartialEq, Eq, Debug, ValueEnum)]
pub enum Base {
    Debian,
    Alpine,
    Arch,
}

pub struct Ctx;

#[derive(Debug)]
pub enum Prompt {
    File(PathBuf),
    Text(String),
    None,
}

#[derive(Debug)]
pub struct Restart {
    pub container: String,
    pub base: Option<Base>,
    pub prompt: Option<Prompt>,
    pub save: bool,
}

pub enum Action {
    Restart(Restart),
}

fn my_completer(_current: &OsStr) -> Vec<CompletionCandidate> {
    vec![CompletionCandidate::new("c1"), CompletionCandidate::new("c2")]
}

#[fill(target = Restart, context = Ctx)]
pub struct RestartArgs {
    #[arg(long, short = 'c', add = ArgValueCompleter::new(my_completer))]
    pub container: String,

    #[arg(long, short = 'b')]
    pub base: Option<Base>,

    #[command(flatten)]
    pub prompt: PromptGroup,

    #[arg(long, short = 's', conflicts_with = "no_save")]
    pub save: bool,
    #[arg(long)]
    pub no_save: command_hills::Only<bool>,
}

#[derive(clap::Args)]
#[group(multiple = false)]
pub struct PromptGroup {
    #[arg(long, short = 'f', add = ArgValueCompleter::new(PathCompleter::any().filter(is_visitable)))]
    pub file: Option<PathBuf>,
    #[arg(long, short = 't')]
    pub text: Option<String>,
    #[arg(long)]
    pub no_prompt: bool,
}

impl command_hills::Resolve<Option<Prompt>> for PromptGroup {
    fn resolve(self) -> Option<Prompt> {
        if self.no_prompt {
            return Some(Prompt::None);
        }
        if let Some(t) = self.text {
            return Some(Prompt::Text(t));
        }
        if let Some(f) = self.file {
            return Some(Prompt::File(f));
        }
        None
    }
}

#[derive(Parser)]
#[command(name = "cat-context", disable_help_flag = true, disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    Restart(RestartArgs),
}

fn main() {
    CompleteEnv::with_factory(|| Cli::command()).complete();
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    };
    match cli.cmd {
        Cmd::Restart(args) => {
            let restart = args.resolve(&Ctx);
            println!("{:?}", restart);
        }
    }
}
