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
    let options = WorldOptions {
        creation_timestamp: Some(0),
        ..WorldOptions::default()
    };
    let (applied, _) = lgl::repair(&text, &out, &options, 5).unwrap();
    let world = LabWorld::new(&out, &options).unwrap();
    let rules = lgl::check(&world, false)
        .diagnostics
        .into_iter()
        .map(|d| d.rule)
        .collect();
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

#[test]
fn unbalanced_columns_get_balanced() {
    let (rules, applied, text) = rules_after_repair("column-imbalance.typ");
    assert!(applied > 0, "no fix applied");
    assert!(!rules.iter().any(|r| r.starts_with("L019")), "remaining: {rules:?}\n{text}");
    assert!(text.contains("balanced: true"), "{text}");
}

#[test]
fn tiny_text_is_raised_to_readable_minimum() {
    let (rules, applied, text) = rules_after_repair("text-too-small.typ");
    assert!(applied > 0, "no fix applied");
    assert!(rules.is_empty(), "remaining: {rules:?}\n{text}");
    assert!(text.contains("size: 6pt"), "{text}");
}

#[test]
fn distorted_image_keeps_aspect_ratio() {
    let text = std::fs::read_to_string(corpus("distorted-image.typ")).unwrap();
    let dir = scratch("assets");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(corpus("assets/tiny.png"), dir.join("tiny.png")).unwrap();
    let out = scratch("distorted-image.typ");
    let options = WorldOptions { creation_timestamp: Some(0), ..WorldOptions::default() };
    let (applied, _) = lgl::repair(&text, &out, &options, 5).unwrap();
    let world = LabWorld::new(&out, &options).unwrap();
    let rules: Vec<String> =
        lgl::check(&world, false).diagnostics.into_iter().map(|d| d.rule).collect();
    let fixed = std::fs::read_to_string(&out).unwrap();
    assert!(applied > 0, "no fix applied");
    assert!(!rules.iter().any(|r| r.starts_with("L023")), "remaining: {rules:?}\n{fixed}");
    assert!(fixed.contains("fit: \"contain\""), "{fixed}");
}
