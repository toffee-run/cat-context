struct Target {
    base: Option<String>,
}

struct Context;

#[command_hills::fill(target = Target, context = Context)]
struct Command {
    base: Option<u32>,
}

fn main() {}
