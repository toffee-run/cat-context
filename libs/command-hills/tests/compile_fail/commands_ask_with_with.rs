struct Context;

async fn resolve(value: Option<String>, context: &Context) -> command_hills::Result<String> {
    let _ = context;
    Ok(value.unwrap_or_default())
}

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "start")]
    Start {
        #[hill(ask = "Value", with = resolve)]
        value: String,
    },
}

fn main() {}
