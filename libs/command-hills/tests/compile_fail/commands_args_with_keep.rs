struct Context;
struct Group;

#[command_hills::commands(context = Context)]
enum Action {
    #[hill(about = "restart")]
    Restart {
        #[hill(args = Group, keep = "Value")]
        value: Option<String>,
    },
}

fn main() {}
