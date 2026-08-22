use clap::{CommandFactory, Parser};

#[derive(Debug, PartialEq)]
struct Context {
    suffix: String,
}

async fn resolve_stop(
    given: Option<String>,
    context: &Context,
    message: &str,
) -> command_hills::Result<String> {
    Ok(format!(
        "{}{}:{message}",
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
        #[hill(
            with = resolve_stop,
            message = "Container",
            arg(long, value_name = "NAME")
        )]
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
            container: "stopped-ctx:Container".to_owned()
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
            container: "web-ctx:Container".to_owned()
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

mod optional_field {
    use clap::Parser;

    struct Context;

    #[command_hills::commands(context = Context)]
    #[derive(Debug, PartialEq)]
    enum Choice {
        #[hill(about = "change a value")]
        Change { value: Option<String> },
    }

    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        choice: ChoiceArgs,
    }

    #[tokio::test]
    async fn unmarked_optional_field_is_preserved() {
        let cli = Cli::try_parse_from(["test", "change"])
            .expect("опциональное поле должно разбираться без значения");

        assert_eq!(
            fill(cli.choice, &Context)
                .await
                .expect("опциональное поле должно переноситься"),
            Choice::Change { value: None }
        );
    }
}

mod question_fields {
    use clap::{Parser, ValueEnum};

    #[derive(Clone, Debug, PartialEq, clap::ValueEnum)]
    enum Base {
        Alpine,
        DebianSlim,
        #[value(skip)]
        Hidden,
    }

    struct Context;

    #[command_hills::commands(context = Context)]
    #[derive(Debug, PartialEq)]
    enum Choice {
        #[hill(about = "start")]
        Start {
            #[hill(ask = "Base", arg(long, value_enum))]
            base: Base,
        },
        #[hill(about = "restart")]
        Restart {
            #[hill(keep = "Base", arg(long, value_enum))]
            base: std::option::Option<Base>,
        },
    }

    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        choice: ChoiceArgs,
    }

    #[tokio::test]
    async fn given_question_values_fill_without_terminal() {
        let start = Cli::try_parse_from(["test", "start", "--base", "alpine"])
            .expect("значение ask должно разбираться");
        let restart = Cli::try_parse_from(["test", "restart", "--base", "debian-slim"])
            .expect("значение keep должно разбираться");

        assert_eq!(
            fill(start.choice, &Context)
                .await
                .expect("ask должно заполняться заданным значением"),
            Choice::Start { base: Base::Alpine }
        );
        assert_eq!(
            fill(restart.choice, &Context)
                .await
                .expect("keep должно заполняться заданным значением"),
            Choice::Restart {
                base: Some(Base::DebianSlim)
            }
        );
    }

    #[test]
    fn question_subcommands_parse_without_arguments() {
        Cli::try_parse_from(["test", "start"]).expect("ask должен быть необязательным");
        Cli::try_parse_from(["test", "restart"]).expect("keep должен быть необязательным");
    }

    #[test]
    fn value_enum_names_exclude_hidden_variant() {
        let _hidden = Base::Hidden;
        let names = Base::value_variants()
            .iter()
            .filter_map(clap::ValueEnum::to_possible_value)
            .map(|value| value.get_name().to_owned())
            .collect::<Vec<_>>();

        assert_eq!(names, ["alpine", "debian-slim"]);
    }
}
