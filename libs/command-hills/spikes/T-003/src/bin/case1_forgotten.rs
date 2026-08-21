pub struct Target {
    pub container: String,
    pub base: Option<String>,
}

pub struct Args {
    pub container: String,
}

impl Args {
    pub fn resolve(self) -> Target {
        Target {
            container: self.container,
        }
    }
}

fn main() {}
