struct Target {
    container: String,
}

fn resolve_container(given: u32) -> command_hills::Result<String> {
    Ok(given.to_string())
}

#[command_hills::fill(target = Target)]
struct Command {
    #[hill(with = resolve_container)]
    container: Option<String>,
}

fn main() {}
