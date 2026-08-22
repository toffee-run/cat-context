struct Target {
    base: String,
}

struct Context;

#[command_hills::fill(target = Target, context = Context)]
struct Command {
    #[hill(ask = "Образ", ctx)]
    #[command(flatten)]
    base: Option<String>,
}

fn main() {}
