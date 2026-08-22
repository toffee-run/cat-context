use std::fmt;

use clap::ValueEnum;
use inquire::Select;

use crate::Result;

pub fn ask_variant<T>(message: &str, given: Option<T>) -> Result<T>
where
    T: ValueEnum + Clone,
{
    match given {
        Some(value) => Ok(value),
        None => Select::new(message, variant_choices::<T>())
            .prompt()
            .map(|choice| choice.value),
    }
}

pub fn ask_variant_or_keep<T>(message: &str, given: Option<T>) -> Result<Option<T>>
where
    T: ValueEnum + Clone,
{
    match given {
        Some(value) => Ok(Some(value)),
        None => {
            let mut choices = vec![KeepChoice::Keep];
            choices.extend(variant_choices::<T>().into_iter().map(KeepChoice::Variant));
            Select::new(message, choices)
                .prompt()
                .map(KeepChoice::into_option)
        }
    }
}

struct Choice<T> {
    name: String,
    value: T,
}

enum KeepChoice<T> {
    Keep,
    Variant(Choice<T>),
}

impl<T> KeepChoice<T> {
    fn into_option(self) -> Option<T> {
        match self {
            Self::Keep => None,
            Self::Variant(choice) => Some(choice.value),
        }
    }
}

impl<T> fmt::Display for Choice<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)
    }
}

impl<T> fmt::Display for KeepChoice<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keep => formatter.write_str("оставить как есть"),
            Self::Variant(choice) => choice.fmt(formatter),
        }
    }
}

fn variant_choices<T>() -> Vec<Choice<T>>
where
    T: ValueEnum + Clone,
{
    T::value_variants()
        .iter()
        .filter_map(|value| {
            value.to_possible_value().map(|possible| Choice {
                name: possible.get_name().to_owned(),
                value: value.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, clap::ValueEnum)]
    enum Base {
        Alpine,
        DebianSlim,
    }

    impl fmt::Display for Base {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("wrong-display")
        }
    }

    #[test]
    fn choices_use_value_enum_names() {
        let labels = variant_choices::<Base>()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        assert_eq!(labels, ["alpine", "debian-slim"]);
    }

    #[test]
    fn keep_is_the_first_choice() {
        let mut choices = vec![KeepChoice::Keep];
        choices.extend(
            variant_choices::<Base>()
                .into_iter()
                .map(KeepChoice::Variant),
        );
        let labels = choices.iter().map(ToString::to_string).collect::<Vec<_>>();

        assert_eq!(labels, ["оставить как есть", "alpine", "debian-slim"]);
    }
}
