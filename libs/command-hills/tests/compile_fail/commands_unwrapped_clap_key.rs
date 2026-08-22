struct Context;

async fn container(
    value: Option<String>,
    context: &Context,
    message: &str,
) -> command_hills::Result<String> {
    let _ = (context, message);
    Ok(value.unwrap_or_default())
}

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "stop")]
    Stop {
        #[hill(with = container, message = "Which?", long, value_name = "NAME")]
        container: String,
    },
}

fn main() {}
