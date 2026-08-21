use clap::{CommandFactory, Parser};
use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter, CompletionCandidate};
use clap_complete::env::CompleteEnv;
use std::ffi::OsStr;

fn my_completer(_current: &OsStr) -> Vec<CompletionCandidate> {
    vec![
        CompletionCandidate::new("hello"),
        CompletionCandidate::new("world"),
    ]
}

#[derive(Parser, Debug)]
#[command(name = "t001")]
struct Cli {
    #[arg(long, add = ArgValueCandidates::new(|| vec![CompletionCandidate::new("alpha"), CompletionCandidate::new("beta")]))]
    opt_a: Option<String>,

    #[arg(long, add = ArgValueCompleter::new(my_completer))]
    opt_b: Option<String>,

    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(["one", "two"]))]
    opt_c: Option<String>,
}

fn main() {
    CompleteEnv::with_factory(|| Cli::command()).complete();

    let cli = Cli::parse();
    println!("{:?}", cli);
}
