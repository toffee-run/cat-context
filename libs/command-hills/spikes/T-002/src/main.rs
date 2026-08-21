use clap::{Args, Parser};
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub enum Base {
    Debian,
    Alpine,
    Arch,
}

#[derive(Clone, Debug, Default)]
pub enum Prompt {
    File(PathBuf),
    Text(String),
    #[default]
    None,
}

#[derive(Debug)]
pub enum Action {
    Restart {
        container: String,
        base: Option<Base>,
        prompt: Option<Prompt>,
        save: bool,
    },
}

pub struct Docker;

pub trait ResolveWithCtx<Ctx, Target> {
    fn resolve(self, ctx: &Ctx) -> Target;
}

pub trait Resolve<Target> {
    fn resolve(self) -> Target;
}

pub struct Only<T>(PhantomData<T>);

// -----------------------------------------------------------------------------

#[derive(Args, Debug)]
pub struct TargetArgs {
    #[arg(long)]
    pub container: Option<String>,
}

impl ResolveWithCtx<Docker, String> for TargetArgs {
    fn resolve(self, _ctx: &Docker) -> String {
        self.container.unwrap_or_else(|| "default".to_string())
    }
}

#[derive(Args, Debug, Default)]
pub struct PromptArgs {
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long)]
    pub no_prompt: bool,
}

impl Resolve<Option<Prompt>> for PromptArgs {
    fn resolve(self) -> Option<Prompt> {
        if let Some(f) = self.file {
            Some(Prompt::File(f))
        } else if let Some(t) = self.text {
            Some(Prompt::Text(t))
        } else if self.no_prompt {
            Some(Prompt::None)
        } else {
            None
        }
    }
}

// -----------------------------------------------------------------------------

/*
#[command_hill(target = Action::Restart, context = Docker)]
struct RestartCommand {
    #[command(flatten)]
    container: TargetArgs,

    base: Option<Base>,

    #[command(flatten)]
    prompt: PromptArgs,

    save: bool,

    no_save: Only<bool>,
}
*/

#[derive(Parser, Debug)]
pub struct RestartCommandArgs {
    #[command(flatten)]
    pub container: TargetArgs,

    #[arg(long)]
    pub base: Option<Base>,

    #[command(flatten)]
    pub prompt: PromptArgs,

    #[arg(long)]
    pub save: bool,

    #[arg(long)]
    pub no_save: bool,
}

impl RestartCommandArgs {
    pub fn resolve(self, ctx: &Docker) -> Action {
        Action::Restart {
            container: self.container.resolve(ctx),
            base: self.base,
            prompt: self.prompt.resolve(),
            save: self.save,
        }
    }
}

fn main() {
    let args = RestartCommandArgs {
        container: TargetArgs {
            container: Some("web".to_string()),
        },
        base: Some(Base::Debian),
        prompt: PromptArgs {
            text: Some("hello".to_string()),
            ..Default::default()
        },
        save: true,
        no_save: false,
    };

    let docker = Docker;
    let action = args.resolve(&docker);
    println!("{:?}", action);
}
