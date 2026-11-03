//! Embeds every font file under the repository's `fonts` directory into the
//! binary, compressed with DEFLATE, so that documents can use the bundled
//! families wherever lgl runs. Fonts are decompressed only when a document
//! uses them.

use std::fmt::Write;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
            e.eq_ignore_ascii_case("ttf") || e.eq_ignore_ascii_case("otf")
        }) {
            out.push(path);
        }
    }
}

/// FNV-1a, so the font index can recognise a file by its content.
fn fnv1a(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let fonts = manifest.join("../../fonts");
    println!("cargo:rerun-if-changed={}", fonts.display());

    let mut files = Vec::new();
    collect(&fonts, &mut files);
    files.sort();

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let packed = out.join("fonts");
    std::fs::create_dir_all(&packed).unwrap();

    let mut code = String::from("pub static BUNDLED_FONTS: &[BundledFont] = &[\n");
    for (i, file) in files.iter().enumerate() {
        let data = std::fs::read(file).unwrap();
        let hash = fnv1a(&data);
        let target = packed.join(format!("{i}.deflate"));
        let fresh = std::fs::read(packed.join(format!("{i}.hash")))
            .is_ok_and(|old| old == hash.to_le_bytes());
        if !fresh || !target.exists() {
            let compressed = miniz_oxide::deflate::compress_to_vec(&data, 9);
            std::fs::write(&target, compressed).unwrap();
            std::fs::write(packed.join(format!("{i}.hash")), hash.to_le_bytes()).unwrap();
        }
        let name = file.strip_prefix(&fonts).unwrap().to_string_lossy().replace('\\', "/");
        writeln!(
            code,
            "    BundledFont {{ name: {name:?}, hash: {hash:#018x}, size: {}, data: include_bytes!({:?}) }},",
            data.len(),
            target.display().to_string(),
        )
        .unwrap();
    }
    code.push_str("];\n");

    std::fs::write(out.join("bundled_fonts.rs"), code).unwrap();
}
