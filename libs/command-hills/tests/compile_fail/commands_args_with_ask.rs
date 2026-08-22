struct Context;
struct Group;

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "start")]
    Start {
        #[hill(args = Group, ask = "Value")]
        value: String,
    },
}

fn main() {}
