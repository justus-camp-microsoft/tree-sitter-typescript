use sha2::{Digest, Sha256};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fingerprint = Sha256::new();
    for path in [
        "Cargo.toml",
        "package.json",
        "package-lock.json",
        "tree-sitter.json",
        "bindings/rust/build.rs",
        "bindings/rust/lib.rs",
        "common/define-grammar.js",
        "common/scanner.h",
        "typescript/grammar.js",
        "typescript/src/parser.c",
        "typescript/src/scanner.c",
        "typescript/src/grammar.json",
        "typescript/src/node-types.json",
        "typescript/src/tree_sitter/alloc.h",
        "typescript/src/tree_sitter/array.h",
        "typescript/src/tree_sitter/parser.h",
        "tsx/grammar.js",
        "tsx/src/parser.c",
        "tsx/src/scanner.c",
        "tsx/src/grammar.json",
        "tsx/src/node-types.json",
        "tsx/src/tree_sitter/alloc.h",
        "tsx/src/tree_sitter/array.h",
        "tsx/src/tree_sitter/parser.h",
    ] {
        println!("cargo:rerun-if-changed={path}");
        fingerprint.update(path.as_bytes());
        fingerprint.update([0]);
        fingerprint.update(std::fs::read(path)?);
        fingerprint.update([0]);
    }
    println!(
        "cargo:rustc-env=EROSION_TYPESCRIPT_FINGERPRINT={:x}",
        fingerprint.finalize()
    );
    let root_dir = std::path::Path::new(".");
    let typescript_dir = root_dir.join("typescript").join("src");
    let tsx_dir = root_dir.join("tsx").join("src");
    let common_dir = root_dir.join("common");

    let mut config = cc::Build::new();
    config.include(&typescript_dir);
    config
        .flag_if_supported("-std=c11")
        .flag_if_supported("-Wno-unused-parameter");

    for path in &[
        typescript_dir.join("parser.c"),
        typescript_dir.join("scanner.c"),
        tsx_dir.join("parser.c"),
        tsx_dir.join("scanner.c"),
    ] {
        config.file(path);
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    }

    println!(
        "cargo:rerun-if-changed={}",
        common_dir.join("scanner.h").to_str().unwrap()
    );

    config.compile("tree-sitter-typescript");
    Ok(())
}
