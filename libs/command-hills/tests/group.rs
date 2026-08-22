use clap::Parser;
use command_hills::{Resolve, Result};

fn fallback() -> Result<Prompt> {
    Ok(Prompt::Text("asked".to_owned()))
}

#[command_hills::group(fallback = fallback)]
#[derive(Debug, PartialEq)]
enum Prompt {
    #[hill(arg(long, value_name = "FILE"))]
    File(String),
    #[hill(arg(long, value_name = "TEXT"))]
    Text(String),
    #[hill(arg(long = "no-prompt"))]
    None,
    DefaultName,
}

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    prompt: PromptArgs,
}

#[test]
fn each_group_flag_resolves_to_its_variant() {
    let file =
        Cli::try_parse_from(["test", "--file", "plan.md"]).expect("флаг file должен разбираться");
    let text =
        Cli::try_parse_from(["test", "--text", "hello"]).expect("флаг text должен разбираться");
    let none = Cli::try_parse_from(["test", "--no-prompt"])
        .expect("переименованный флаг должен разбираться");
    let default_name = Cli::try_parse_from(["test", "--default-name"])
        .expect("имя флага должно порождаться в kebab-case");

    assert_eq!(
        Resolve::<Prompt>::resolve(file.prompt).expect("file должен разрешаться"),
        Prompt::File("plan.md".to_owned())
    );
    assert_eq!(
        Resolve::<Prompt>::resolve(text.prompt).expect("text должен разрешаться"),
        Prompt::Text("hello".to_owned())
    );
    assert_eq!(
        Resolve::<Prompt>::resolve(none.prompt).expect("none должен разрешаться"),
        Prompt::None
    );
    assert_eq!(
        Resolve::<Prompt>::resolve(default_name.prompt).expect("default-name должен разрешаться"),
        Prompt::DefaultName
    );
}

#[test]
fn group_flags_are_mutually_exclusive() {
    let result = Cli::try_parse_from(["test", "--file", "plan.md", "--text", "hello"]);

    assert!(result.is_err());
}

#[test]
fn empty_required_group_uses_fallback() {
    let cli = Cli::try_parse_from(["test"]).expect("пустая группа должна разбираться");

    assert_eq!(
        Resolve::<Prompt>::resolve(cli.prompt).expect("должен вызываться fallback"),
        Prompt::Text("asked".to_owned())
    );
}

#[test]
fn empty_optional_group_returns_none() {
    let cli = Cli::try_parse_from(["test"]).expect("пустая группа должна разбираться");

    assert_eq!(
        Resolve::<Option<Prompt>>::resolve(cli.prompt).expect("optional-группа должна разрешаться"),
        None
    );
}
