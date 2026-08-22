struct Target {
    container: String,
}

#[command_hills::fill(target = Target)]
struct Command {
    #[hill(with = resolve_container, ask = "Container")]
    container: Option<String>,
}

fn main() {}
