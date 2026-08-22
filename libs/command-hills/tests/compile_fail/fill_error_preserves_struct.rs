struct Target {
    value: String,
}

#[command_hills::fill(target = Target)]
struct Arguments {
    #[hill(unknown = "x")]
    value: Option<String>,
}

fn accepts(_: Arguments) {}

fn main() {}
