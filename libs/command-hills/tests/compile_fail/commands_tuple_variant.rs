struct Context;

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "stop")]
    Stop(String),
}

fn main() {}
