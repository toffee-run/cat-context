struct Context;

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "list", unknown = "x")]
    List,
}

fn main() {}
