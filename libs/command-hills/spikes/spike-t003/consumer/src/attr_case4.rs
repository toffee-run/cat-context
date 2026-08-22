use spike_macro::generate_resolve;

pub struct Target {
    pub container: String,
}

#[generate_resolve]
pub struct Args {
    #[unknown]
    pub container: String,
}

fn main() {}
