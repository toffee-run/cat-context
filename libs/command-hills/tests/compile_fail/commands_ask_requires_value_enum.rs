struct Context;

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "start")]
    Start {
        #[hill(ask = "Value")]
        value: String,
    },
}

fn main() {}
