use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use lgl::world::{LabWorld, WorldOptions};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

fn expectations(source: &str) -> Option<BTreeSet<String>> {
    let mut expected = BTreeSet::new();
    let mut declared = false;
    for line in source.lines() {
        let line = line.trim();
        if line == "// expect-none" {
            declared = true;
        } else if let Some(rule) = line.strip_prefix("// expect:") {
            declared = true;
            expected.insert(rule.trim().to_string());
        }
    }
    declared.then_some(expected)
}

#[test]
fn corpus() {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(corpus_dir())
        .expect("corpus directory exists")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "typ"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "corpus is empty");

    let options = WorldOptions {
        creation_timestamp: Some(0),
        ..WorldOptions::default()
    };
    let mut failures = Vec::new();

    for path in &paths {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let source = std::fs::read_to_string(path).expect("readable corpus file");
        let Some(expected) = expectations(&source) else {
            failures
                .push(format!("{name}: missing `// expect:` or `// expect-none` header"));
            continue;
        };

        let world = LabWorld::new(path, &options).expect("world");
        let first = lgl::check(&world, true);
        for d in &first.diagnostics {
            if d.message.contains('\n') || d.hints.iter().any(|h| h.contains('\n')) {
                failures.push(format!("{name}: {} has a line break in its text", d.rule));
            }
        }
        let actual: BTreeSet<String> =
            first.diagnostics.iter().map(|d| d.rule.clone()).collect();
        if actual != expected {
            let lines: Vec<String> = first
                .diagnostics
                .iter()
                .map(|d| serde_json::to_string(d).unwrap())
                .collect();
            failures.push(format!(
                "{name}: expected {expected:?}, got {actual:?}\n  {}",
                lines.join("\n  ")
            ));
            continue;
        }

        if first.passed() {
            let fresh = LabWorld::new(path, &options).expect("world");
            let second = lgl::check(&fresh, true);
            if first.pdf != second.pdf {
                failures.push(format!("{name}: PDF output is not reproducible"));
            }
        }
    }

    assert!(failures.is_empty(), "corpus failures:\n{}", failures.join("\n"));
}
