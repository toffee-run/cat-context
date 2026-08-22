struct Context;

#[command_hills::commands(context = Context)]
struct Action {
    value: String,
}

fn main() {}
