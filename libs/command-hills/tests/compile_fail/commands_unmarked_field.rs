struct Context;

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "stop")]
    Stop { container: String },
}

fn main() {}
