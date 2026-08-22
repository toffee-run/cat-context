enum Action {
    Restart { base: Option<String> },
}

#[command_hills::fill(variant = Action::Restart)]
struct Restart {
    base: Option<String>,
    memory: Option<String>,
}

fn main() {}
