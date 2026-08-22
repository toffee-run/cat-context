use spike_macro::generate_resolve;

pub struct Target {
    pub container: String,
}

#[generate_resolve]
pub struct Args {
    pub container: String,
    pub extra: String,
}

fn main() {}
