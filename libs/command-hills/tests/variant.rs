use clap::Parser;

#[derive(Debug, PartialEq)]
enum Action {
    Restart {
        container: String,
        base: Option<String>,
    },
}

#[command_hills::fill(variant = Action::Restart)]
struct Restart {
    #[arg(long)]
    container: String,
    #[arg(long)]
    base: Option<String>,
}

#[test]
fn parses_arguments_and_resolves_enum_variant() {
    let command = Restart::try_parse_from(["test", "--container", "backend", "--base", "alpine"])
        .expect("аргументы должны разбираться");

    assert_eq!(
        command.resolve().expect("вариант должен разрешаться"),
        Action::Restart {
            container: "backend".to_owned(),
            base: Some("alpine".to_owned()),
        }
    );
}
