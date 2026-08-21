use std::ffi::OsStr;

use cat_context::complete::{containers, endpoints};

#[test]
fn endpoints_offer_every_scheme() {
    let offered: Vec<String> = endpoints()
        .iter()
        .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
        .collect();

    for scheme in [
        "unix://", "npipe://", "tcp://", "http://", "https://", "ssh://",
    ] {
        assert!(offered.contains(&scheme.to_owned()), "{scheme}");
    }
}

#[test]
fn nothing_matches_an_unknown_prefix() {
    assert!(containers(OsStr::new("нет-такого-контейнера")).is_empty());
}

#[test]
fn a_broken_name_gives_no_candidates() {
    let broken = unsafe { OsStr::from_encoded_bytes_unchecked(&[0xff, 0xfe]) };

    assert!(containers(broken).is_empty());
}
