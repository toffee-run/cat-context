struct Context;

fn endpoint(given: Option<Context>) -> Context {
    given.unwrap_or(Context)
}

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "list")]
    List,
}

#[command_hills::root(action = Action, ask = "Choose")]
struct Command {
    #[hill(with = endpoint)]
    connect: Context,
}

fn main() {}
