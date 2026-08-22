use clap::Parser;

#[derive(Debug, PartialEq)]
struct Target {
    container: String,
}

fn resolve_container(given: Option<String>) -> command_hills::Result<String> {
    Ok(given.unwrap_or_else(|| "chosen".to_owned()))
}

#[command_hills::fill(target = Target)]
struct Command {
    #[arg(long)]
    #[hill(with = resolve_container)]
    container: Option<String>,
}

struct Context {
    suffix: String,
}

async fn resolve_container_with_ctx(
    given: Option<String>,
    context: &Context,
) -> command_hills::Result<String> {
    Ok(format!(
        "{}{}",
        given.unwrap_or_else(|| "chosen".to_owned()),
        context.suffix
    ))
}

#[command_hills::fill(target = Target, context = Context)]
struct ContextCommand {
    #[arg(long)]
    #[hill(with = resolve_container_with_ctx)]
    container: Option<String>,
}

#[test]
fn field_resolves_with_function() {
    let command = Command::try_parse_from(["test"]).expect("команда должна разбираться");

    assert_eq!(
        command.resolve().expect("цель должна разрешаться"),
        Target {
            container: "chosen".to_owned(),
        }
    );
}

#[tokio::test]
async fn field_resolves_with_context_function() {
    let command = ContextCommand::try_parse_from(["test", "--container", "web"])
        .expect("команда должна разбираться");
    let context = Context {
        suffix: "-1".to_owned(),
    };

    assert_eq!(
        command
            .resolve(&context)
            .await
            .expect("цель должна разрешаться"),
        Target {
            container: "web-1".to_owned(),
        }
    );
}
