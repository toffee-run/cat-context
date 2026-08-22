use spike_macro::generate_resolve;

pub struct Target {
    pub container: String,
    pub base: Option<String>,
}

#[generate_resolve]
pub struct Args {
    pub container: String,
    pub base: Option<String>,
}

fn main() {}
