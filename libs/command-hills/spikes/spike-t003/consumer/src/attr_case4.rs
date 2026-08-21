use spike_macro::generate_resolve;

pub struct Target {
    pub container: String,
}

#[generate_resolve]
pub struct Args {
    #[unknown]
    pub container: String, // Unknown attribute triggers custom macro error
}

fn main() {}
