struct Context;

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "stop")]
    Stop {
        #[hill(message = "Container")]
        container: String,
    },
}

fn main() {}
