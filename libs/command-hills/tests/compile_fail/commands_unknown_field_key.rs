struct Context;

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "stop")]
    Stop {
        #[hill(magic = "x")]
        container: String,
    },
}

fn main() {}
