enum Action {
    Restart {
        base: Option<String>,
        memory: Option<String>,
    },
}

#[command_hills::fill(variant = Action::Restart)]
struct Restart {
    base: Option<String>,
}

fn main() {}
