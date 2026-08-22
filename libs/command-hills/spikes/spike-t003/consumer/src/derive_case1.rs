use spike_macro::ResolveDerive;

pub struct Target {
    pub container: String,
    pub base: Option<String>,
}

#[derive(ResolveDerive)]
pub struct Args {
    pub container: String,
    pub base: Option<String>,
}

fn main() {}
