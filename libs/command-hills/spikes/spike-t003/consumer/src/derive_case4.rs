use spike_macro::ResolveDerive;

pub struct Target {
    pub container: String,
}

#[derive(ResolveDerive)]
pub struct Args {
    #[unknown]
    pub container: String,
}

fn main() {}
