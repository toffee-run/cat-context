use clap::{Parser, ValueEnum, CommandFactory};
use clap_complete::engine::{ArgValueCompleter, PathCompleter};
use clap_complete::env::CompleteEnv;
use inquire::autocompletion::{Autocomplete, Replacement};
use inquire::CustomUserError;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Clone, Copy, PartialEq, Eq, Debug, ValueEnum)]
pub enum Base {
    Debian,
    Alpine,
    Arch,
}

pub fn is_markdown(path: &Path) -> bool {
    match path.extension() {
        Some(extension) => extension.eq_ignore_ascii_case("md"),
        None => false,
    }
}

fn is_visitable(path: &Path) -> bool {
    path.is_dir() || is_markdown(path)
}

#[derive(Parser, Debug)]
#[command(name = "t004")]
struct Cli {
    #[arg(long, value_enum)]
    base: Option<Base>,

    #[arg(
        long,
        value_name = "FILE",
        add = ArgValueCompleter::new(PathCompleter::any().filter(is_visitable)),
    )]
    file: Option<PathBuf>,
}

#[derive(Clone)]
struct MarkdownPathCompleter;

impl Autocomplete for MarkdownPathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        Ok(suggestions(input))
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        match highlighted_suggestion {
            Some(s) => Ok(Replacement::Some(s)),
            None => Ok(common_prefix(suggestions(input)).map(Replacement::Some).unwrap_or(Replacement::None)),
        }
    }
}

fn suggestions(input: &str) -> Vec<String> {
    let (dir, prefix) = split_path(input);

    let Ok(entries) = fs::read_dir(if dir.is_empty() { "." } else { &dir }) else {
        return Vec::new();
    };

    let mut found = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();

        if !name.starts_with(&prefix) {
            continue;
        }

        let path = entry.path();

        if path.is_dir() {
            found.push(format!("{dir}{name}/"));
        } else if is_markdown(&path) {
            found.push(format!("{dir}{name}"));
        }
    }

    found.sort();
    found
}

fn split_path(input: &str) -> (String, String) {
    match input.rsplit_once('/') {
        Some((dir, prefix)) => (format!("{dir}/"), prefix.to_owned()),
        None => (String::new(), input.to_owned()),
    }
}

fn common_prefix(values: Vec<String>) -> Option<String> {
    let mut values = values.into_iter();
    let mut common = values.next()?;

    for value in values {
        let shared = common
            .chars()
            .zip(value.chars())
            .take_while(|(left, right)| left == right)
            .count();

        common = common.chars().take(shared).collect();
    }

    Some(common)
}

fn main() {
    CompleteEnv::with_factory(|| Cli::command()).complete();
    let _cli = Cli::parse();
    let _completer = MarkdownPathCompleter;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("spike-t004-{name}-{}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn join(&self, name: &str) -> String {
            self.0.join(name).to_string_lossy().into_owned()
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn clap_variants_can_be_extracted_for_inquire() {
        let names: Vec<String> = Base::value_variants()
            .iter()
            .map(|v| v.to_possible_value().unwrap().get_name().to_string())
            .collect();
        assert_eq!(names, vec!["debian".to_string(), "alpine".to_string(), "arch".to_string()]);
    }

    #[test]
    fn inquire_path_completer_filters_md() {
        let dir = TempDir::new("suggestions");
        fs::write(dir.join("plan.md"), "").unwrap();
        fs::write(dir.join("notes.txt"), "").unwrap();
        fs::create_dir_all(dir.join("nested")).unwrap();

        let mut completer = MarkdownPathCompleter;
        let prefix = format!("{}/", dir.0.to_string_lossy());
        let found = completer.get_suggestions(&prefix).unwrap();

        assert!(found.contains(&format!("{}/", dir.join("nested"))));
        assert!(found.contains(&dir.join("plan.md")));
        assert!(!found.contains(&dir.join("notes.txt")));
    }
}
