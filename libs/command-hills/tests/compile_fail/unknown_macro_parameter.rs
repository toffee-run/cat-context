struct Target {
    base: Option<String>,
}

struct Context;

#[command_hills::fill(target = Target, context = Context, unknown = Bar)]
struct Command {
    base: Option<String>,
}

fn main() {}
