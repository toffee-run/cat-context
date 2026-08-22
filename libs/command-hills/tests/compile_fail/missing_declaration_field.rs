struct Target {
    base: Option<String>,
    memory: Option<String>,
}

struct Context;

#[command_hills::fill(target = Target, context = Context)]
struct Command {
    base: Option<String>,
}

fn main() {}
