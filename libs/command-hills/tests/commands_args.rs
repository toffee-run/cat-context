use clap::Parser;
use command_hills::{Resolve, ResolveWithCtx, Result};

#[derive(Debug, PartialEq)]
struct Prompt(String);

#[derive(clap::Args)]
#[group(multiple = false)]
struct PromptArgs {
    #[arg(long)]
    file: Option<String>,
    #[arg(long)]
    text: Option<String>,
}

impl Resolve<Prompt> for PromptArgs {
    fn resolve(self) -> Result<Prompt> {
        Ok(Prompt(
            self.file
                .or(self.text)
                .unwrap_or_else(|| "asked".to_owned()),
        ))
    }
}

impl Resolve<Option<Prompt>> for PromptArgs {
    fn resolve(self) -> Result<Option<Prompt>> {
        Ok(self.file.or(self.text).map(Prompt))
    }
}

struct Context {
    suffix: String,
}

impl ResolveWithCtx<Context, Prompt> for PromptArgs {
    fn resolve(self, context: &Context) -> impl Future<Output = Result<Prompt>> {
        let value = self
            .file
            .or(self.text)
            .unwrap_or_else(|| "asked".to_owned());
        std::future::ready(Ok(Prompt(format!("{}{}", value, context.suffix))))
    }
}

#[command_hills::commands(context = Context)]
#[derive(Debug, PartialEq)]
enum Action {
    #[hill(about = "start")]
    Start {
        #[hill(args = PromptArgs)]
        prompt: Prompt,
    },
    #[hill(about = "update")]
    Update {
        #[hill(args = PromptArgs)]
        prompt: Option<Prompt>,
    },
    #[hill(about = "restart")]
    Restart {
        #[hill(args = PromptArgs, ctx)]
        prompt: Prompt,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    action: ActionArgs,
}

#[tokio::test]
async fn grouped_arguments_resolve_with_and_without_context() {
    let start = Cli::try_parse_from(["test", "start", "--text", "hello"])
        .expect("синхронная группа должна разбираться");
    let restart = Cli::try_parse_from(["test", "restart", "--text", "hello"])
        .expect("контекстная группа должна разбираться");
    let context = Context {
        suffix: " world".to_owned(),
    };

    assert_eq!(
        fill(start.action, &context)
            .await
            .expect("синхронная группа должна разрешаться"),
        Action::Start {
            prompt: Prompt("hello".to_owned())
        }
    );
    assert_eq!(
        fill(restart.action, &context)
            .await
            .expect("контекстная группа должна разрешаться"),
        Action::Restart {
            prompt: Prompt("hello world".to_owned())
        }
    );
}

#[tokio::test]
async fn one_argument_type_resolves_two_target_types() {
    let update = Cli::try_parse_from(["test", "update", "--file", "plan.md"])
        .expect("группа для опциональной цели должна разбираться");
    let context = Context {
        suffix: String::new(),
    };

    assert_eq!(
        fill(update.action, &context)
            .await
            .expect("опциональная цель должна разрешаться"),
        Action::Update {
            prompt: Some(Prompt("plan.md".to_owned()))
        }
    );
}

#[test]
fn grouped_arguments_remain_mutually_exclusive() {
    let result = Cli::try_parse_from(["test", "start", "--file", "plan.md", "--text", "hello"]);

    assert!(result.is_err());
}

#[test]
fn grouped_argument_subcommand_parses_without_values() {
    Cli::try_parse_from(["test", "start"]).expect("группа должна быть необязательной");
}
