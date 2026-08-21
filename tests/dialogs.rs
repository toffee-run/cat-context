use bollard::Docker;

use cat_context::ask::{Choice, container, variant, variant_or_keep};
use cat_context::{Agent, Base};

#[test]
fn a_given_variant_is_taken_as_is() {
    assert_eq!(
        variant("Базовый образ", Some(Base::Alpine)).expect("вопроса не было"),
        Base::Alpine
    );
    assert_eq!(
        variant_or_keep("Агент", Some(Agent::Codex)).expect("вопроса не было"),
        Some(Agent::Codex)
    );
}

#[tokio::test]
async fn a_given_container_is_taken_as_is() {
    let docker = Docker::connect_with_host("tcp://127.0.0.1:1")
        .expect("клиент собран без обращения к демону");

    let name = container(
        &docker,
        Some("cat-arch-codex".to_owned()),
        "Какой контейнер?",
    )
    .await
    .expect("вопроса не было");

    assert_eq!(name, "cat-arch-codex");
}

#[test]
fn a_choice_shows_its_label() {
    let choice = Choice {
        name: "start".to_owned(),
        label: "запустить новый контейнер".to_owned(),
    };

    assert_eq!(choice.to_string(), "запустить новый контейнер");
}
