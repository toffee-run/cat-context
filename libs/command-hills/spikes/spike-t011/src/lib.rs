use command_hills::commands;
use clap::ValueEnum;

pub struct Docker;
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, ValueEnum)]
pub enum Base {
    #[default]
    Debian,
    Alpine,
    Arch,
}

pub mod case1 {
    use super::*;
    type MaybeBase = Option<Base>;
    #[commands(context = Docker)]
    pub enum Action {
        #[hill(about = "start")]
        Start {
            #[hill(keep = "Базовый образ")]
            base: MaybeBase,
        },
    }
}

pub mod case2 {
    use super::*;
    #[commands(context = Docker)]
    pub enum Action {
        #[hill(about = "start")]
        Start {
            #[hill(keep = "Базовый образ")]
            base: std::option::Option<Base>,
        },
    }
}

pub mod case3 {
    use super::*;
    #[commands(context = Docker)]
    pub enum Action {
        #[hill(about = "start")]
        Start {
            #[hill(keep = "Базовый образ")]
            base: Base,
        },
    }
}

pub mod case4 {
    use super::*;
    #[commands(context = Docker)]
    pub enum Action {
        #[hill(about = "start")]
        Start {
            #[hill(ask = "Базовый образ")]
            base: Option<Base>,
        },
    }
}
