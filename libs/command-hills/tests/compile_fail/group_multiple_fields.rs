fn fallback() -> command_hills::Result<Text> {
    Ok(Text::Value("one".to_owned(), "two".to_owned()))
}

#[command_hills::group(fallback = fallback)]
enum Text {
    Value(String, String),
}

fn main() {}
