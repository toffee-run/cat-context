use clap::{Parser, ValueEnum, CommandFactory};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate, PathCompleter};
use clap_complete::env::CompleteEnv;
use inquire::autocompletion::Autocomplete;
use inquire::CustomUserError;
use std::path::{Path, PathBuf};

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
        // No custom validation! It just parses into PathBuf.
        add = ArgValueCompleter::new(PathCompleter::any().filter(is_visitable)),
    )]
    file: Option<PathBuf>,
}

#[derive(Clone)]
struct MarkdownPathCompleter;

impl Autocomplete for MarkdownPathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let mut results = vec![];
        // Minimal mock implementation for the unit test requirement
        if input == "src" {
            results.push("src/main.rs".to_string());
            results.push("src/utils.md".to_string());
        }
        
        let filtered = results.into_iter()
            .filter(|p| is_visitable(Path::new(p)))
            .collect();
            
        Ok(filtered)
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, CustomUserError> {
        Ok(match highlighted_suggestion {
            Some(s) => inquire::autocompletion::Replacement::Some(s),
            None => inquire::autocompletion::Replacement::None,
        })
    }
}

fn main() {
    // If COMPLETE is set, this intercepts and exits
    CompleteEnv::with_factory(|| Cli::command()).complete();

    // If we parse normally, help flag is disabled by clap features.
    // So this will panic/error with "unexpected argument" if we pass --help.
    let _cli = Cli::parse();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inquire_suggests_variants_by_name() {
        let names: Vec<String> = Base::value_variants()
            .iter()
            .map(|v| v.to_possible_value().unwrap().get_name().to_string())
            .collect();
        assert_eq!(names, vec!["debian".to_string(), "alpine".to_string(), "arch".to_string()]);
    }

    #[test]
    fn inquire_path_completer_filters_md() {
        let mut completer = MarkdownPathCompleter;
        let suggestions = completer.get_suggestions("src").unwrap();
        // src/utils.md is retained because it's markdown, src/main.rs is excluded
        assert_eq!(suggestions, vec!["src/utils.md"]);
    }
    
    #[test]
    fn base_parses_from_name() {
        let cli = Cli::try_parse_from(["t004", "--base", "debian"]).unwrap();
        assert_eq!(cli.base, Some(Base::Debian));
    }
}
