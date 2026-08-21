use std::ffi::OsStr;
use std::path::Path;
use std::{env, fs};

use bollard::Docker;
use clap_complete::engine::CompletionCandidate;

use crate::docker::{self, Managed};

const SCHEMES: [&str; 6] = [
    "unix://", "npipe://", "tcp://", "http://", "https://", "ssh://",
];

pub fn containers(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return Vec::new();
    };

    let mut candidates = Vec::new();

    for container in list() {
        if !container.name.starts_with(current) {
            continue;
        }

        let hint = format!("{} · {}", container.base, container.agent);
        candidates.push(CompletionCandidate::new(&container.name).help(Some(hint.into())));
    }

    candidates
}

pub fn endpoints() -> Vec<CompletionCandidate> {
    let mut candidates: Vec<_> = SCHEMES.into_iter().map(CompletionCandidate::new).collect();

    for socket in local_sockets() {
        candidates.push(CompletionCandidate::new(format!("unix://{socket}")));
    }

    for host in ssh_hosts() {
        candidates.push(CompletionCandidate::new(format!("ssh://{host}")));
    }

    candidates
}

fn list() -> Vec<Managed> {
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return Vec::new();
    };

    let docker = match endpoint_from_argv() {
        Some(host) => Docker::connect_with_host(&host),
        None => Docker::connect_with_defaults(),
    };

    let Ok(docker) = docker else {
        return Vec::new();
    };

    runtime.block_on(docker::list(&docker)).unwrap_or_default()
}

fn endpoint_from_argv() -> Option<String> {
    endpoint_from(env::args())
}

#[cfg(test)]
mod endpoint_from_tests {
    use super::*;

    fn argv(args: &[&str]) -> impl Iterator<Item = String> {
        args.iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn endpoint_is_read_from_the_command_line() {
        assert_eq!(
            endpoint_from(argv(&["cat-context", "--connect", "ssh://box", "stop"])),
            Some("ssh://box".to_owned())
        );
        assert_eq!(
            endpoint_from(argv(&["cat-context", "--connect=ssh://box"])),
            Some("ssh://box".to_owned())
        );
        assert_eq!(endpoint_from(argv(&["cat-context", "stop"])), None);
        assert_eq!(endpoint_from(argv(&["cat-context", "--connect"])), None);
    }
}

#[cfg(test)]
mod endpoint_from_argv_tests {
    use super::*;

    #[test]
    fn the_test_binary_has_no_endpoint_flag() {
        assert_eq!(endpoint_from_argv(), None);
    }
}

fn endpoint_from(mut args: impl Iterator<Item = String>) -> Option<String> {
    while let Some(arg) = args.next() {
        if arg == "--connect" {
            return args.next();
        }

        if let Some(host) = arg.strip_prefix("--connect=") {
            return Some(host.to_owned());
        }
    }

    None
}

fn local_sockets() -> Vec<String> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_default();

    socket_candidates(&runtime_dir)
        .into_iter()
        .filter(|socket| Path::new(socket).exists())
        .collect()
}

fn socket_candidates(runtime_dir: &str) -> Vec<String> {
    vec![
        "/var/run/docker.sock".to_owned(),
        format!("{runtime_dir}/docker.sock"),
        format!("{runtime_dir}/podman/podman.sock"),
    ]
}

#[cfg(test)]
mod socket_candidates_tests {
    use super::*;

    #[test]
    fn socket_candidates_cover_docker_and_podman() {
        let candidates = socket_candidates("/run/user/1000");

        assert_eq!(
            candidates,
            vec![
                "/var/run/docker.sock".to_owned(),
                "/run/user/1000/docker.sock".to_owned(),
                "/run/user/1000/podman/podman.sock".to_owned(),
            ]
        );
    }
}

fn ssh_hosts() -> Vec<String> {
    let Ok(home) = env::var("HOME") else {
        return Vec::new();
    };

    let Ok(config) = fs::read_to_string(Path::new(&home).join(".ssh/config")) else {
        return Vec::new();
    };

    parse_hosts(&config)
}

fn parse_hosts(config: &str) -> Vec<String> {
    let mut hosts = Vec::new();

    for line in config.lines() {
        let Some(names) = line.trim().strip_prefix("Host ") else {
            continue;
        };

        for name in names.split_whitespace() {
            if !name.contains(['*', '?']) {
                hosts.push(name.to_owned());
            }
        }
    }

    hosts
}

#[cfg(test)]
mod parse_hosts_tests {
    use super::*;

    #[test]
    fn ssh_config_gives_plain_hosts_only() {
        let config = "\
    Host box other
        HostName 10.0.0.1
    Host *
        User root
    Host web?
    Host build
    ";

        assert_eq!(
            parse_hosts(config),
            vec!["box".to_owned(), "other".to_owned(), "build".to_owned()]
        );
    }

    #[test]
    fn ssh_config_without_hosts_is_empty() {
        assert!(parse_hosts("User root\n\n# Host commented\n").is_empty());
    }
}
