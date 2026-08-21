use spike_macro::generate_resolve;

pub struct Target {
    pub container: String,
}

#[generate_resolve]
pub struct Args {
    pub container: Option<String>, // Type mismatch
}

fn main() {}
