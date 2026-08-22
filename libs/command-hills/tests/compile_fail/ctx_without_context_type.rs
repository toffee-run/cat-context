struct Target {
    base: String,
}

#[derive(clap::Args)]
struct BaseArgs;

impl command_hills::ResolveWithCtx<Context, String> for BaseArgs {
    async fn resolve(self, _ctx: &Context) -> command_hills::Result<String> {
        Ok("base".to_owned())
    }
}

struct Context;

#[command_hills::fill(target = Target)]
struct Command {
    #[command(flatten)]
    #[hill(ctx)]
    base: BaseArgs,
}

fn main() {}
