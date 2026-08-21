use cat_context::docker::{Managed, container_name, parse_name};
use cat_context::{Agent, Base};

#[test]
fn parses_own_names() {
    assert_eq!(
        parse_name("cat-alpine-codex"),
        Some((Base::Alpine, Agent::Codex))
    );
    assert_eq!(
        parse_name("/cat-debian-claude-code"),
        Some((Base::Debian, Agent::ClaudeCode))
    );
    assert_eq!(
        parse_name("cat-arch-opencode-2"),
        Some((Base::Arch, Agent::Opencode))
    );
}

#[test]
fn rejects_foreign_names() {
    let foreign = [
        "catalog",
        "cat-ubuntu-codex",
        "cat-alpine-cursor",
        "cat-alpine-codexx",
        "my-cat-alpine-codex",
    ];

    for name in foreign {
        assert_eq!(parse_name(name), None, "{name}");
    }
}

#[test]
fn container_name_matches_pattern() {
    assert_eq!(
        container_name(Base::Debian, Agent::ClaudeCode),
        "cat-debian-claude-code"
    );
    assert_eq!(
        parse_name(&container_name(Base::Arch, Agent::Opencode)),
        Some((Base::Arch, Agent::Opencode))
    );
}

#[test]
fn managed_shows_its_state() {
    let named = |running| Managed {
        name: "cat-arch-codex".to_owned(),
        base: Base::Arch,
        agent: Agent::Codex,
        running,
    };

    assert_eq!(named(true).to_string(), "cat-arch-codex  (работает)");
    assert_eq!(named(false).to_string(), "cat-arch-codex  (остановлен)");
}
