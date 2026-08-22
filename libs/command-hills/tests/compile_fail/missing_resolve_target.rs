use command_hills::{Resolve, Result};

struct Prompt;

#[derive(clap::Args)]
struct PromptArgs {}

impl Resolve<Prompt> for PromptArgs {
    fn resolve(self) -> Result<Prompt> {
        Ok(Prompt)
    }
}

struct Target {
    prompt: Option<Prompt>,
}

#[command_hills::fill(target = Target)]
struct Command {
    #[command(flatten)]
    prompt: PromptArgs,
}

fn main() {}
