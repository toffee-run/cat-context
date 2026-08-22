struct Context;

async fn resolve(
    value: Option<Option<String>>,
    context: &Context,
) -> command_hills::Result<Option<String>> {
    let _ = context;
    Ok(value.flatten())
}

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "restart")]
    Restart {
        #[hill(keep = "Value", with = resolve)]
        value: Option<String>,
    },
}

fn main() {}
