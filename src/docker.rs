use std::collections::HashMap;
use std::fmt;

use bollard::Docker;
use bollard::models::{ContainerSummary, ContainerSummaryStateEnum};
use bollard::query_parameters::{ListContainersOptions, ListContainersOptionsBuilder};
use clap::ValueEnum;

use crate::{Agent, Base};

pub const PREFIX: &str = "cat";

pub struct Managed {
    pub name: String,
    pub base: Base,
    pub agent: Agent,
    pub running: bool,
}

impl fmt::Display for Managed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = if self.running {
            "работает"
        } else {
            "остановлен"
        };
        write!(f, "{}  ({state})", self.name)
    }
}

pub fn container_name(base: Base, agent: Agent) -> String {
    format!("{PREFIX}-{base}-{agent}")
}

pub fn parse_name(name: &str) -> Option<(Base, Agent)> {
    let name = name.trim_start_matches('/');

    for base in Base::value_variants() {
        let Some(rest) = name.strip_prefix(&format!("{PREFIX}-{base}-")) else {
            continue;
        };

        for agent in Agent::value_variants() {
            let Some(tail) = rest.strip_prefix(&agent.to_string()) else {
                continue;
            };

            if tail.is_empty() || tail.starts_with('-') {
                return Some((*base, *agent));
            }
        }
    }

    None
}

pub async fn list(docker: &Docker) -> Result<Vec<Managed>, bollard::errors::Error> {
    let summaries = docker.list_containers(Some(list_options())).await?;
    Ok(managed(summaries))
}

fn list_options() -> ListContainersOptions {
    ListContainersOptionsBuilder::new()
        .all(true)
        .filters(&HashMap::from([("name", vec![name_pattern()])]))
        .build()
}

#[cfg(test)]
mod list_options_tests {
    use super::*;

    #[test]
    fn listing_asks_for_all_containers_by_name() {
        let options = list_options();
        let filters = options.filters.expect("фильтр задан");

        assert!(options.all);
        assert_eq!(filters["name"], vec![name_pattern()]);
    }
}

fn managed(summaries: Vec<ContainerSummary>) -> Vec<Managed> {
    let mut containers = Vec::new();

    for summary in summaries {
        let Some(names) = summary.names else {
            continue;
        };

        let Some(name) = names.first() else {
            continue;
        };

        let name = name.trim_start_matches('/').to_owned();

        let Some((base, agent)) = parse_name(&name) else {
            continue;
        };

        containers.push(Managed {
            name,
            base,
            agent,
            running: summary.state == Some(ContainerSummaryStateEnum::RUNNING),
        });
    }

    containers.sort_by(|left, right| left.name.cmp(&right.name));
    containers
}

#[cfg(test)]
mod managed_tests {
    use super::*;

    fn summary(name: &str, running: bool) -> ContainerSummary {
        ContainerSummary {
            names: Some(vec![format!("/{name}")]),
            state: running.then_some(ContainerSummaryStateEnum::RUNNING),
            ..Default::default()
        }
    }

    #[test]
    fn summaries_become_sorted_containers() {
        let listed = managed(vec![
            summary("cat-debian-codex", true),
            summary("cat-alpine-claude-code", false),
        ]);

        let names: Vec<_> = listed.iter().map(|item| item.name.as_str()).collect();

        assert_eq!(names, vec!["cat-alpine-claude-code", "cat-debian-codex"]);
        assert_eq!(listed[0].base, Base::Alpine);
        assert_eq!(listed[0].agent, Agent::ClaudeCode);
        assert!(!listed[0].running);
        assert!(listed[1].running);
    }

    #[test]
    fn foreign_and_nameless_summaries_are_skipped() {
        let nameless = ContainerSummary::default();
        let empty_names = ContainerSummary {
            names: Some(Vec::new()),
            ..Default::default()
        };

        let listed = managed(vec![
            summary("catalog", true),
            summary("cat-ubuntu-codex", true),
            nameless,
            empty_names,
            summary("cat-arch-opencode", true),
        ]);

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "cat-arch-opencode");
    }
}

fn name_pattern() -> String {
    let bases = joined_variants::<Base>();
    let agents = joined_variants::<Agent>();

    format!("^/?{PREFIX}-({bases})-({agents})(-|$)")
}

#[cfg(test)]
mod name_pattern_tests {
    use super::*;

    #[test]
    fn pattern_is_built_from_variants() {
        assert_eq!(
            name_pattern(),
            "^/?cat-(debian|alpine|arch)-(claude-code|codex|antigravity|opencode)(-|$)"
        );
    }
}

fn joined_variants<T: ValueEnum + fmt::Display>() -> String {
    T::value_variants()
        .iter()
        .map(T::to_string)
        .collect::<Vec<_>>()
        .join("|")
}
