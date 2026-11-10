//! Workspace implementation identity, shared by the library build scripts.
//!
//! This is deliberately a conservative source identity, not semantic equivalence or a binary
//! digest. Contract revisions belong to their owning layer; receipts pin the actual binary.
use std::{env, fs, path::Path, process::Command};

fn field(hash: &mut blake3::Hasher, value: &[u8]) {
    hash.update(&(value.len() as u64).to_be_bytes());
    hash.update(value);
}

fn sources(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(directory).expect("read implementation source directory") {
        let path = entry.expect("read implementation source entry").path();
        if path.is_dir() {
            sources(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

pub fn emit(lock: &str) {
    let root = std::path::PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let mut hash = blake3::Hasher::new();
    field(&mut hash, b"jarde-workspace-implementation-v1");
    // Absolute locations and timestamps are not identity. Include our own algorithm as well as
    // each owner's build script, manifest, lock and complete src tree (including inline tests).
    field(&mut hash, include_bytes!("implementation.rs"));
    for path in ["Cargo.toml", "build.rs", lock] {
        println!("cargo:rerun-if-changed={path}");
        field(&mut hash, path.as_bytes());
        field(
            &mut hash,
            &fs::read(root.join(path)).expect("read build identity input"),
        );
    }
    println!("cargo:rerun-if-changed=src");
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    files.sort();
    for path in files {
        let relative = path.strip_prefix(&root).unwrap().to_str().unwrap();
        println!("cargo:rerun-if-changed={relative}");
        field(&mut hash, relative.as_bytes());
        field(
            &mut hash,
            &fs::read(path).expect("read implementation source"),
        );
    }
    // The only current optional library feature is observational. Its presence must not make
    // normal and observed runs disagree about the same artifact. Any future semantic feature is
    // included automatically. Build flags and target/compiler changes conservatively invalidate.
    let mut features = env::vars()
        .filter(|(key, _)| key.starts_with("CARGO_FEATURE_") && key != "CARGO_FEATURE_TEST_SUPPORT")
        .collect::<Vec<_>>();
    features.sort();
    for (key, value) in features {
        field(&mut hash, key.as_bytes());
        field(&mut hash, value.as_bytes());
    }
    for key in ["TARGET", "CARGO_ENCODED_RUSTFLAGS"] {
        println!("cargo:rerun-if-env-changed={key}");
        field(&mut hash, key.as_bytes());
        field(&mut hash, env::var(key).unwrap_or_default().as_bytes());
    }
    let compiler = Command::new(env::var_os("RUSTC").unwrap())
        .arg("-Vv")
        .output()
        .expect("read compiler identity");
    assert!(compiler.status.success(), "compiler identity failed");
    field(&mut hash, &compiler.stdout);
    println!(
        "cargo:rustc-env=JARDE_IMPLEMENTATION_ID=blake3:{}",
        hash.finalize().to_hex()
    );
}
