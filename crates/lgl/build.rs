//! Embeds every font file under the repository's `fonts` directory into the
//! binary, so that documents can use the bundled families wherever lgl runs.

use std::fmt::Write;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("ttf") || e.eq_ignore_ascii_case("otf"))
        {
            out.push(path);
        }
    }
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let fonts = manifest.join("../../fonts");
    println!("cargo:rerun-if-changed={}", fonts.display());

    let mut files = Vec::new();
    collect(&fonts, &mut files);
    files.sort();

    let mut code = String::from("pub static BUNDLED_FONTS: &[&[u8]] = &[\n");
    for file in &files {
        let path = file.canonicalize().unwrap();
        writeln!(code, "    include_bytes!({:?}),", path.display().to_string()).unwrap();
    }
    code.push_str("];\n");

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bundled_fonts.rs");
    std::fs::write(out, code).unwrap();
}
