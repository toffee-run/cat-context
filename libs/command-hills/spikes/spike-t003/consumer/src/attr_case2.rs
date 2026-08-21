use spike_macro::generate_resolve;

pub struct Target {
    pub container: String,
}

#[generate_resolve]
pub struct Args {
    pub container: String,
    pub extra: String, // The macro will map this, triggering extra field error in Target
}

fn main() {}
