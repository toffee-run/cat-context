use std::convert::Infallible;
use std::str::FromStr;

use clap::{CommandFactory, Parser};

#[derive(Clone, Debug, PartialEq)]
struct Context(String);

impl FromStr for Context {
    type Err = Infallible;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(value.to_owned()))
    }
}

fn endpoint(given: Option<Context>) -> Context {
    given.unwrap_or_else(|| Context("default".to_owned()))
}

async fn value(given: Option<String>, context: &Context) -> command_hills::Result<String> {
    Ok(given.unwrap_or_else(|| context.0.clone()))
}

#[command_hills::commands(context = Context)]
#[derive(Debug, PartialEq)]
enum Action {
    #[hill(about = "first action")]
    First {
        #[hill(with = value, arg(long))]
        value: String,
    },
    #[hill(about = "second action")]
    Second {
        #[hill(with = value, arg(long))]
        value: String,
    },
}

#[command_hills::root(action = Action, ask = "Choose action")]
#[derive(Debug, PartialEq)]
struct Command {
    #[hill(context, with = endpoint, arg(long))]
    context: Context,
}

#[tokio::test]
async fn root_parses_subcommand_and_passes_context_to_fill() {
    let cli = Cli::try_parse_from(["test", "--context", "root", "first"])
        .expect("корневая команда должна разбираться");

    assert_eq!(
        cli.resolve()
            .await
            .expect("корневая команда должна заполняться"),
        Command {
            context: Context("root".to_owned()),
            action: Action::First {
                value: "root".to_owned()
            }
        }
    );
}

#[test]
fn root_subcommands_parse_without_arguments() {
    Cli::try_parse_from(["test", "first"]).expect("первая подкоманда должна разбираться");
    Cli::try_parse_from(["test", "second"]).expect("вторая подкоманда должна разбираться");
    assert_eq!(Cli::command().get_subcommands().count(), 2);
}
