pub struct Target {
    pub container: String,
}

pub struct Args {
    pub container: Option<String>,
}

impl Args {
    pub fn resolve(self) -> Target {
        Target {
            container: self.container,
        }
    }
}

fn main() {}
