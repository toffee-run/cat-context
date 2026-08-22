use std::path::Path;

use cat_context::cli::is_markdown;

#[test]
fn markdown_is_recognised_by_extension() {
    assert!(is_markdown(Path::new("plan.md")));
    assert!(is_markdown(Path::new("docs/PLAN.MD")));
    assert!(!is_markdown(Path::new("plan.txt")));
    assert!(!is_markdown(Path::new("plan")));
}
