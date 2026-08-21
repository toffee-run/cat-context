pub struct Target {
    pub container: String,
}

pub struct Args {
    pub container: String,
    pub extra: String,
}

impl Args {
    pub fn resolve(self) -> Target {
        Target {
            container: self.container,
            extra: self.extra,
        }
    }
}

fn main() {}
