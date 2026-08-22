use clap::CommandFactory;
use spike_lib::generate_cli;
use spike_lib::reexports::CompletionCandidate;
use std::ffi::OsStr;

fn my_candidates() -> Vec<CompletionCandidate> {
    vec![CompletionCandidate::new("alpha"), CompletionCandidate::new("beta")]
}

fn my_completer(_current: &OsStr) -> Vec<CompletionCandidate> {
    vec![CompletionCandidate::new("hello"), CompletionCandidate::new("world")]
}

#[generate_cli]
struct Cli {
    #[hill(candidates = || my_candidates())]
    opt_a: Option<String>,

    #[hill(completer = my_completer)]
    opt_b: Option<String>,

    #[hill(value_parser = ::spike_lib::reexports::PossibleValuesParser::new(["one", "two"]))]
    opt_c: Option<String>,
}

fn main() {
    use spike_lib::reexports::{CompleteEnv, Parser};
    CompleteEnv::with_factory(|| Cli::command()).complete();
    let cli = Cli::parse();
    println!("{:?}", cli);
}
