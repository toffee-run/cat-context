pub use spike_macro::generate_cli;

pub mod reexports {
    pub use clap; pub use clap::Parser;
    pub use clap_complete::engine::ArgValueCandidates;
    pub use clap_complete::engine::ArgValueCompleter;
    pub use clap_complete::engine::CompletionCandidate;
    pub use clap_complete::env::CompleteEnv;
    pub use clap::builder::PossibleValuesParser;
}
