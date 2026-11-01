use std::path::{Path, PathBuf};

use lgl::world::{LabWorld, WorldOptions};

fn corpus(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus").join(name)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lgl-fixes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn rules_after_repair(name: &str) -> (Vec<String>, usize, String) {
    let text = std::fs::read_to_string(corpus(name)).unwrap();
    let out = scratch(name);
    let options = WorldOptions { creation_timestamp: Some(0), ..WorldOptions::default() };
    let (applied, _) = lgl::repair(&text, &out, &options, 5).unwrap();
    let world = LabWorld::new(&out, &options).unwrap();
    let rules = lgl::check(&world, false).diagnostics.into_iter().map(|d| d.rule).collect();
    (rules, applied, std::fs::read_to_string(&out).unwrap())
}

#[test]
fn fixed_height_block_gets_auto_height() {
    let (rules, applied, text) = rules_after_repair("fixed-height-block.typ");
    assert!(applied > 0, "no fix applied");
    assert!(rules.is_empty(), "remaining: {rules:?}\n{text}");
    assert!(text.contains("height: auto"), "{text}");
}

#[test]
fn wide_box_is_limited_to_available_width() {
    let (rules, applied, text) = rules_after_repair("overflow-wide-box.typ");
    assert!(applied > 0, "no fix applied");
    assert!(rules.is_empty(), "remaining: {rules:?}\n{text}");
    assert!(text.contains("width: 100%"), "{text}");
}

#[test]
fn margin_intrusion_is_limited_to_available_width() {
    let (rules, applied, text) = rules_after_repair("margin-intrusion.typ");
    assert!(applied > 0, "no fix applied");
    assert!(rules.is_empty(), "remaining: {rules:?}\n{text}");
}

#[test]
fn manual_spacing_outlier_is_removed() {
    let (rules, applied, text) = rules_after_repair("inconsistent-spacing.typ");
    assert!(applied > 0, "no fix applied");
    assert!(rules.is_empty(), "remaining: {rules:?}\n{text}");
    assert!(!text.contains("#v(24pt)"), "{text}");
}

#[test]
fn clean_documents_are_left_untouched() {
    for name in ["ok-basic.typ", "ok-features.typ"] {
        let original = std::fs::read_to_string(corpus(name)).unwrap();
        let (_, applied, text) = rules_after_repair(name);
        assert_eq!(applied, 0, "{name} was modified");
        assert_eq!(text, original);
    }
}
