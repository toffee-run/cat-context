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
    // We can use CompleteEnv from reexports to test it works exactly as in T-001
    // But since the task only asks to compile and print candidates, we'll do it like T-001:
    use spike_lib::reexports::{CompleteEnv, Parser};
    CompleteEnv::with_factory(|| Cli::command()).complete();
    let cli = Cli::parse();
    println!("{:?}", cli);
}
