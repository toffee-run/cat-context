pub use command_hills_macros::fill;

pub trait Resolve<Target> {
    fn resolve(self) -> Target;
}

#[doc(hidden)]
pub mod __private {
    pub use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter, CompletionCandidate};
}
