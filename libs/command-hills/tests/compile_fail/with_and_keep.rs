struct Target {
    container: Option<String>,
}

#[command_hills::fill(target = Target)]
struct Command {
    #[hill(with = resolve_container, keep = "Container")]
    container: Option<String>,
}

fn main() {}
