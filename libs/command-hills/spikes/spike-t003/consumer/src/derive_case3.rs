use spike_macro::ResolveDerive;

pub struct Target {
    pub container: String,
}

#[derive(ResolveDerive)]
pub struct Args {
    pub container: Option<String>,
}

fn main() {}
