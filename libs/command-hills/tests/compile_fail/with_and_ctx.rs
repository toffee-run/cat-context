struct Target {
    container: String,
}

struct Context;

#[command_hills::fill(target = Target, context = Context)]
struct Command {
    #[hill(with = resolve_container, ctx)]
    container: Option<String>,
}

fn main() {}
