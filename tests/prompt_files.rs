use std::path::{Path, PathBuf};

use cat_context::cli::{is_markdown, markdown_path};

#[test]
fn markdown_path_checks_the_extension() {
    assert_eq!(markdown_path("plan.md"), Ok(PathBuf::from("plan.md")));
    assert_eq!(markdown_path("plan.MD"), Ok(PathBuf::from("plan.MD")));
    assert!(markdown_path("plan.txt").is_err());
    assert!(markdown_path("plan").is_err());
}

#[test]
fn markdown_is_recognised_by_extension() {
    assert!(is_markdown(Path::new("plan.md")));
    assert!(is_markdown(Path::new("docs/PLAN.MD")));
    assert!(!is_markdown(Path::new("plan.txt")));
    assert!(!is_markdown(Path::new("plan")));
}
