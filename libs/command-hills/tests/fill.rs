use clap::Parser;
use command_hills::Resolve;

#[derive(Debug, PartialEq)]
struct Target {
    base: Option<String>,
    prompt: Prompt,
}

#[derive(Debug, PartialEq)]
struct Prompt(String);

#[derive(clap::Args)]
struct PromptArgs {
    #[arg(long)]
    text: String,
}

impl Resolve<Prompt> for PromptArgs {
    fn resolve(self) -> Prompt {
        Prompt(self.text)
    }
}

#[command_hills::fill(target = Target, context = Context)]
struct Command {
    #[arg(long)]
    base: Option<String>,
    #[command(flatten)]
    prompt: PromptArgs,
}

struct Context;

#[test]
fn parses_arguments_and_resolves_target() {
    let command = Command::try_parse_from(["test", "--base", "alpine", "--text", "hello"])
        .expect("аргументы должны разбираться");

    assert_eq!(
        command.resolve(&Context),
        Target {
            base: Some("alpine".to_owned()),
            prompt: Prompt("hello".to_owned()),
        }
    );
}
