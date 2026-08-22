use clap::Parser;
use command_hills::{Resolve, ResolveWithCtx, Result};

#[derive(Debug, PartialEq)]
struct Prompt(String);

#[derive(clap::Args)]
struct PromptArgs {
    #[arg(long)]
    text: Option<String>,
}

impl Resolve<Prompt> for PromptArgs {
    fn resolve(self) -> Result<Prompt> {
        Ok(Prompt(self.text.unwrap_or_else(|| "asked".to_owned())))
    }
}

impl Resolve<Option<Prompt>> for PromptArgs {
    fn resolve(self) -> Result<Option<Prompt>> {
        Ok(self.text.map(Prompt))
    }
}

struct Context {
    suffix: String,
}

impl ResolveWithCtx<Context, Prompt> for PromptArgs {
    fn resolve(self, context: &Context) -> impl Future<Output = Result<Prompt>> {
        let text = self.text.unwrap_or_else(|| "asked".to_owned());
        std::future::ready(Ok(Prompt(format!("{}{}", text, context.suffix))))
    }
}

impl ResolveWithCtx<Context, Option<Prompt>> for PromptArgs {
    fn resolve(self, context: &Context) -> impl Future<Output = Result<Option<Prompt>>> {
        let prompt = self
            .text
            .map(|text| Prompt(format!("{}{}", text, context.suffix)));
        std::future::ready(Ok(prompt))
    }
}

#[derive(Debug, PartialEq)]
struct RequiredTarget {
    prompt: Prompt,
}

#[command_hills::fill(target = RequiredTarget)]
struct RequiredCommand {
    #[command(flatten)]
    prompt: PromptArgs,
}

#[derive(Debug, PartialEq)]
struct OptionalTarget {
    prompt: Option<Prompt>,
}

#[command_hills::fill(target = OptionalTarget)]
struct OptionalCommand {
    #[command(flatten)]
    prompt: PromptArgs,
}

#[command_hills::fill(target = RequiredTarget, context = Context)]
struct RequiredContextCommand {
    #[command(flatten)]
    #[hill(ctx)]
    prompt: PromptArgs,
}

#[command_hills::fill(target = OptionalTarget, context = Context)]
struct OptionalContextCommand {
    #[command(flatten)]
    #[hill(ctx)]
    prompt: PromptArgs,
}

#[test]
fn one_type_resolves_two_targets() {
    let required =
        RequiredCommand::try_parse_from(["test"]).expect("обязательная команда должна разбираться");
    let optional = OptionalCommand::try_parse_from(["test"])
        .expect("необязательная команда должна разбираться");

    assert_eq!(
        required
            .resolve()
            .expect("обязательная цель должна разрешаться"),
        RequiredTarget {
            prompt: Prompt("asked".to_owned()),
        }
    );
    assert_eq!(
        optional
            .resolve()
            .expect("необязательная цель должна разрешаться"),
        OptionalTarget { prompt: None }
    );
}

#[tokio::test]
async fn one_type_resolves_two_targets_with_context() {
    let required = RequiredContextCommand::try_parse_from(["test", "--text", "hello"])
        .expect("обязательная команда должна разбираться");
    let optional = OptionalContextCommand::try_parse_from(["test"])
        .expect("необязательная команда должна разбираться");
    let context = Context {
        suffix: " world".to_owned(),
    };

    assert_eq!(
        required
            .resolve(&context)
            .await
            .expect("обязательная цель должна разрешаться"),
        RequiredTarget {
            prompt: Prompt("hello world".to_owned()),
        }
    );
    assert_eq!(
        optional
            .resolve(&context)
            .await
            .expect("необязательная цель должна разрешаться"),
        OptionalTarget { prompt: None }
    );
}
