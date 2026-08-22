struct Target {
    base: String,
}

struct Context;

#[command_hills::fill(target = Target, context = Context)]
struct Command {
    #[hill(ctx)]
    base: String,
}

fn main() {}
