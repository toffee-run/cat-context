use clap::{CommandFactory, Parser};

#[derive(Debug, PartialEq)]
struct Context {
    suffix: String,
}

async fn resolve_stop(given: Option<String>, context: &Context) -> command_hills::Result<String> {
    Ok(format!(
        "{}{}",
        given.unwrap_or_else(|| "stopped".to_owned()),
        context.suffix
    ))
}

async fn resolve_delete(given: Option<String>, context: &Context) -> command_hills::Result<String> {
    Ok(format!(
        "{}{}",
        given.unwrap_or_else(|| "deleted".to_owned()),
        context.suffix
    ))
}

#[command_hills::commands(context = Context)]
#[derive(Debug, PartialEq)]
enum Action {
    #[hill(about = "stop a container")]
    Stop {
        #[hill(with = resolve_stop, arg(long, value_name = "NAME"))]
        container: String,
    },
    #[hill(about = "delete a container")]
    Delete {
        #[hill(with = resolve_delete, arg(long, value_name = "NAME"))]
        container: String,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    action: ActionArgs,
}

#[tokio::test]
async fn subcommands_parse_without_arguments_and_fill_actions() {
    let context = Context {
        suffix: "-ctx".to_owned(),
    };
    let stop = Cli::try_parse_from(["test", "stop"]).expect("stop должна разбираться");
    let delete = Cli::try_parse_from(["test", "delete"]).expect("delete должна разбираться");

    assert_eq!(
        fill(stop.action, &context)
            .await
            .expect("stop должна заполняться"),
        Action::Stop {
            container: "stopped-ctx".to_owned()
        }
    );
    assert_eq!(
        fill(delete.action, &context)
            .await
            .expect("delete должна заполняться"),
        Action::Delete {
            container: "deleted-ctx".to_owned()
        }
    );
}

#[tokio::test]
async fn subcommands_accept_explicit_arguments() {
    let context = Context {
        suffix: "-ctx".to_owned(),
    };
    let cli = Cli::try_parse_from(["test", "stop", "--container", "web"])
        .expect("аргумент контейнера должен разбираться");

    assert_eq!(
        fill(cli.action, &context)
            .await
            .expect("stop должна заполняться"),
        Action::Stop {
            container: "web-ctx".to_owned()
        }
    );
}

#[test]
fn subcommand_about_is_available_from_command() {
    let command = Cli::command();
    let descriptions = command
        .get_subcommands()
        .map(|subcommand| {
            (
                subcommand.get_name().to_owned(),
                subcommand.get_about().map(ToString::to_string),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        descriptions,
        vec![
            ("stop".to_owned(), Some("stop a container".to_owned())),
            ("delete".to_owned(), Some("delete a container".to_owned()))
        ]
    );
}
