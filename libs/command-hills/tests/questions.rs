use clap::Parser;
use command_hills::{ResolveWithCtx, Result};

#[derive(Clone, Debug, PartialEq, clap::ValueEnum)]
enum Base {
    Alpine,
    Debian,
}

#[derive(Debug, PartialEq)]
struct AskedTarget {
    base: Base,
}

#[command_hills::fill(target = AskedTarget)]
struct AskedCommand {
    #[arg(long, value_enum)]
    #[hill(ask = "Base")]
    base: Option<Base>,
}

#[test]
fn given_value_does_not_open_a_terminal() {
    let command = AskedCommand::try_parse_from(["test", "--base", "alpine"])
        .expect("аргументы должны разбираться");

    assert_eq!(
        command.resolve().expect("цель должна разрешаться"),
        AskedTarget { base: Base::Alpine }
    );
}

#[derive(Debug, PartialEq)]
struct ContextTarget {
    prompt: String,
}

#[derive(clap::Args)]
struct ContextArgs {
    #[arg(long)]
    text: String,
}

struct Context {
    suffix: String,
}

impl ResolveWithCtx<Context, String> for ContextArgs {
    fn resolve(self, context: &Context) -> impl Future<Output = Result<String>> {
        std::future::ready(Ok(format!("{}{}", self.text, context.suffix)))
    }
}

#[command_hills::fill(target = ContextTarget, context = Context)]
struct ContextCommand {
    #[command(flatten)]
    #[hill(ctx)]
    prompt: ContextArgs,
}

#[tokio::test]
async fn contextual_field_is_awaited() {
    let command = ContextCommand::try_parse_from(["test", "--text", "hello"])
        .expect("аргументы должны разбираться");
    let context = Context {
        suffix: " world".to_owned(),
    };

    assert_eq!(
        command
            .resolve(&context)
            .await
            .expect("цель должна разрешаться"),
        ContextTarget {
            prompt: "hello world".to_owned(),
        }
    );
}
