use bollard::Docker;
use inquire::InquireError;

use cat_context::{Action, Command, exit_code};

fn command() -> Command {
    Command {
        connect: Docker::connect_with_host("tcp://127.0.0.1:1").expect("клиент собран"),
        action: Action::Stop {
            container: "cat-arch-codex".to_owned(),
        },
    }
}

#[test]
fn a_filled_command_exits_with_success() {
    assert_eq!(exit_code(Ok(command())), 0);
}

#[test]
fn a_cancelled_dialog_exits_quietly() {
    assert_eq!(exit_code(Err(InquireError::OperationCanceled)), 130);
    assert_eq!(exit_code(Err(InquireError::OperationInterrupted)), 130);
}

#[test]
fn any_other_failure_is_reported() {
    assert_eq!(exit_code(Err(InquireError::NotTTY)), 1);
}
