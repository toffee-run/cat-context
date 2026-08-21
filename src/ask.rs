use std::fmt;
use std::fs;
use std::path::PathBuf;

use bollard::Docker;
use clap::ValueEnum;
use inquire::autocompletion::{Autocomplete, Replacement};
use inquire::validator::Validation;
use inquire::{Confirm, CustomUserError, InquireError, Select, Text};

use crate::cli::{is_markdown, markdown_path};
use crate::{Prompt, docker};

pub type Result<T> = std::result::Result<T, InquireError>;

pub struct Choice {
    pub name: String,
    pub label: String,
}

impl fmt::Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

pub fn action(choices: Vec<Choice>) -> Result<String> {
    let chosen = Select::new("Что сделать?", choices).prompt()?;
    Ok(chosen.name)
}

pub fn variant<T>(message: &str, given: Option<T>) -> Result<T>
where
    T: ValueEnum + Clone + fmt::Display + 'static,
{
    match given {
        Some(value) => Ok(value),
        None => Select::new(message, T::value_variants().to_vec()).prompt(),
    }
}

pub fn variant_or_keep<T>(message: &str, given: Option<T>) -> Result<Option<T>>
where
    T: ValueEnum + Clone + fmt::Display + 'static,
{
    match given {
        Some(value) => Ok(Some(value)),
        None => choose_or_keep(message, T::value_variants().to_vec()),
    }
}

pub async fn container(docker: &Docker, given: Option<String>, message: &str) -> Result<String> {
    if let Some(name) = given {
        return Ok(name);
    }

    let containers = docker::list(docker).await.unwrap_or_default();

    if containers.is_empty() {
        return Text::new(message).prompt();
    }

    let chosen = Select::new(message, containers).prompt()?;
    Ok(chosen.name)
}

pub fn prompt() -> Result<Prompt> {
    let source = Select::new("Промпт", PromptSource::all()).prompt()?;
    source.ask()
}

pub fn prompt_or_keep() -> Result<Option<Prompt>> {
    match choose_or_keep("Промпт", PromptSource::all())? {
        Some(source) => Ok(Some(source.ask()?)),
        None => Ok(None),
    }
}

pub fn save() -> Result<bool> {
    Confirm::new("Сохранить чаты (том контейнера)?")
        .with_default(true)
        .prompt()
}

#[derive(Clone, Copy)]
enum PromptSource {
    File,
    Text,
    Empty,
}

impl PromptSource {
    fn all() -> Vec<Self> {
        vec![Self::File, Self::Text, Self::Empty]
    }

    fn ask(self) -> Result<Prompt> {
        match self {
            Self::File => ask_file(),
            Self::Text => ask_text(),
            Self::Empty => Ok(Prompt::None),
        }
    }
}

impl fmt::Display for PromptSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::File => "файл .md",
            Self::Text => "текст",
            Self::Empty => "без промпта",
        })
    }
}

#[cfg(test)]
mod prompt_source_tests {
    use super::*;

    #[test]
    fn prompt_sources_are_named() {
        assert_eq!(PromptSource::File.to_string(), "файл .md");
        assert_eq!(PromptSource::Text.to_string(), "текст");
        assert_eq!(PromptSource::Empty.to_string(), "без промпта");
        assert_eq!(PromptSource::all().len(), 3);
    }

    #[test]
    fn empty_prompt_source_needs_no_input() {
        assert!(matches!(PromptSource::Empty.ask(), Ok(Prompt::None)));
    }
}

fn ask_file() -> Result<Prompt> {
    let path = Text::new("Файл с промптом")
        .with_autocomplete(MarkdownPaths)
        .with_validator(validate_markdown)
        .prompt()?;

    Ok(Prompt::File(PathBuf::from(path)))
}

fn ask_text() -> Result<Prompt> {
    let text = Text::new("Текст промпта").prompt()?;
    Ok(Prompt::Text(text))
}

fn validate_markdown(input: &str) -> std::result::Result<Validation, CustomUserError> {
    match markdown_path(input) {
        Ok(_) => Ok(Validation::Valid),
        Err(message) => Ok(Validation::Invalid(message.into())),
    }
}

#[cfg(test)]
mod validate_markdown_tests {
    use super::*;

    #[test]
    fn markdown_validation_reports_the_extension() {
        assert!(matches!(
            validate_markdown("plan.md"),
            Ok(Validation::Valid)
        ));
        assert!(matches!(
            validate_markdown("plan.txt"),
            Ok(Validation::Invalid(_))
        ));
    }
}

enum Change<T> {
    Keep,
    New(T),
}

impl<T: fmt::Display> fmt::Display for Change<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keep => f.write_str("оставить как есть"),
            Self::New(value) => value.fmt(f),
        }
    }
}

#[cfg(test)]
mod change_tests {
    use super::*;

    #[test]
    fn keeping_a_value_is_offered_first() {
        let kept: Change<PromptSource> = Change::Keep;

        assert_eq!(kept.to_string(), "оставить как есть");
        assert_eq!(
            Change::New(PromptSource::Text).to_string(),
            PromptSource::Text.to_string()
        );
    }
}

fn choose_or_keep<T: fmt::Display>(message: &str, options: Vec<T>) -> Result<Option<T>> {
    let mut choices = vec![Change::Keep];
    choices.extend(options.into_iter().map(Change::New));

    match Select::new(message, choices).prompt()? {
        Change::Keep => Ok(None),
        Change::New(value) => Ok(Some(value)),
    }
}

#[derive(Clone)]
struct MarkdownPaths;

impl Autocomplete for MarkdownPaths {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> std::result::Result<Vec<String>, CustomUserError> {
        Ok(suggestions(input))
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted: Option<String>,
    ) -> std::result::Result<Replacement, CustomUserError> {
        match highlighted {
            Some(suggestion) => Ok(Some(suggestion)),
            None => Ok(common_prefix(suggestions(input))),
        }
    }
}

#[cfg(test)]
mod autocomplete_tests {
    use super::fixtures::TempDir;
    use super::*;

    #[test]
    fn autocomplete_reuses_the_suggestions() {
        let dir = TempDir::new("autocomplete");
        fs::write(dir.join("plan.md"), "").expect("файл создан");

        let input = format!("{}/pl", dir.0.to_string_lossy());
        let mut paths = MarkdownPaths;

        assert_eq!(
            paths.get_suggestions(&input).expect("подсказки собраны"),
            vec![dir.join("plan.md")]
        );
        assert_eq!(
            paths
                .get_completion(&input, Some("выбранное".to_owned()))
                .expect("дополнение получено"),
            Some("выбранное".to_owned())
        );
        assert_eq!(
            paths
                .get_completion(&input, None)
                .expect("дополнение получено"),
            Some(dir.join("plan.md"))
        );
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

#[cfg(test)]
mod suggestions_tests {
    use super::fixtures::TempDir;
    use super::*;

    #[test]
    fn suggestions_list_markdown_and_directories() {
        let dir = TempDir::new("suggestions");
        fs::write(dir.join("plan.md"), "").expect("файл создан");
        fs::write(dir.join("notes.txt"), "").expect("файл создан");
        fs::create_dir_all(dir.join("nested")).expect("каталог создан");

        let found = suggestions(&format!("{}/", dir.0.to_string_lossy()));

        assert!(found.contains(&format!("{}/", dir.join("nested"))));
        assert!(found.contains(&dir.join("plan.md")));
        assert!(!found.contains(&dir.join("notes.txt")));
    }

    #[test]
    fn suggestions_respect_the_typed_prefix() {
        let dir = TempDir::new("prefix");
        fs::write(dir.join("plan.md"), "").expect("файл создан");
        fs::write(dir.join("other.md"), "").expect("файл создан");

        let found = suggestions(&format!("{}/pl", dir.0.to_string_lossy()));

        assert_eq!(found, vec![dir.join("plan.md")]);
    }

    #[test]
    fn suggestions_of_a_missing_directory_are_empty() {
        assert!(suggestions("/no/such/directory/x").is_empty());
    }
}

fn split_path(input: &str) -> (String, String) {
    match input.rsplit_once('/') {
        Some((dir, prefix)) => (format!("{dir}/"), prefix.to_owned()),
        None => (String::new(), input.to_owned()),
    }
}

#[cfg(test)]
mod split_path_tests {
    use super::*;

    #[test]
    fn path_splits_on_the_last_slash() {
        assert_eq!(
            split_path("docs/plan"),
            ("docs/".to_owned(), "plan".to_owned())
        );
        assert_eq!(split_path("plan"), (String::new(), "plan".to_owned()));
        assert_eq!(split_path("docs/"), ("docs/".to_owned(), String::new()));
        assert_eq!(split_path("/etc/ho"), ("/etc/".to_owned(), "ho".to_owned()));
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

#[cfg(test)]
mod common_prefix_tests {
    use super::*;

    #[test]
    fn common_prefix_shrinks_to_the_shared_part() {
        assert_eq!(common_prefix(Vec::new()), None);
        assert_eq!(
            common_prefix(vec!["plan.md".to_owned()]),
            Some("plan.md".to_owned())
        );
        assert_eq!(
            common_prefix(vec!["plan-a.md".to_owned(), "plan-b.md".to_owned()]),
            Some("plan-".to_owned())
        );
        assert_eq!(
            common_prefix(vec!["plan.md".to_owned(), "notes.md".to_owned()]),
            Some(String::new())
        );
    }
}

#[cfg(test)]
mod fixtures {
    use super::*;

    pub struct TempDir(pub PathBuf);

    impl TempDir {
        pub fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("cat-context-{name}-{}", std::process::id()));
            fs::create_dir_all(&path).expect("временный каталог создан");
            Self(path)
        }

        pub fn join(&self, name: &str) -> String {
            self.0.join(name).to_string_lossy().into_owned()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
