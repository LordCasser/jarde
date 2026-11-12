// Shared by independently compiled integration targets; each uses a different subset.
#![allow(dead_code)]
#[derive(Debug)]
pub enum Outcome {
    Equal,
    Rejected,
    InvalidOriginal,
    Failed(String),
}
// Complete-class verifier oracle: never strips, edits, or replaces any generated member.
pub fn inspect(label: &str, class: &str, bytes: &[u8], text: &str, replay: &str) -> Outcome {
    use std::{fs, process::Command};
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("jarde-boundary-{label}-{stamp}"));
    let original = base.join("original");
    let generated = base.join("generated");
    for dir in [&original, &generated] {
        fs::create_dir_all(dir.join(class).parent().unwrap()).unwrap();
    }
    fs::write(original.join(format!("{class}.class")), bytes).unwrap();
    fs::write(generated.join(format!("{class}.java")), text).unwrap();
    for dir in [&original, &generated] {
        fs::write(dir.join("Replay.java"), replay).unwrap();
    }
    let javac = std::env::var("JAVAC").unwrap_or_else(|_| "javac".into());
    let java = std::env::var("JAVA").unwrap_or_else(|_| "java".into());
    let compile = |dir: &std::path::Path, recovered: bool| {
        let mut cmd = Command::new(&javac);
        cmd.current_dir(dir).args(["--release", "8", "-d", "."]);
        if recovered {
            cmd.arg(format!("{class}.java"));
        }
        cmd.arg("Replay.java").output().unwrap()
    };
    let c0 = compile(&original, false);
    let c1 = compile(&generated, true);
    fs::write(base.join("original-javac.stderr"), &c0.stderr).unwrap();
    fs::write(base.join("generated-javac.stderr"), &c1.stderr).unwrap();
    eprintln!(
        "ORACLE {label}: {} generated_compile={}",
        base.display(),
        c1.status
    );
    if !c0.status.success() {
        return Outcome::Failed(format!(
            "{label}: original runner did not compile: {}",
            String::from_utf8_lossy(&c0.stderr)
        ));
    }
    let run = |dir: &std::path::Path| {
        Command::new(&java)
            .current_dir(dir)
            .args(["-Xverify:all", "-cp", ".", "Replay"])
            .output()
            .unwrap()
    };
    let a = run(&original);
    fs::write(base.join("original.stdout"), &a.stdout).unwrap();
    fs::write(base.join("original.stderr"), &a.stderr).unwrap();
    if !a.status.success() {
        eprintln!(
            "ORACLE {label}: INVALID/UNEXECUTABLE ORIGINAL {}; no equivalence claim",
            String::from_utf8_lossy(&a.stderr)
        );
        return Outcome::InvalidOriginal;
    }
    if !c1.status.success() {
        eprintln!("ORACLE {label}: generated whole class refuses compilation; recovery unproven");
        fs::remove_dir_all(&base).unwrap();
        return Outcome::Rejected;
    }
    let b = run(&generated);
    fs::write(base.join("generated.stdout"), &b.stdout).unwrap();
    fs::write(base.join("generated.stderr"), &b.stderr).unwrap();
    if !b.status.success() || a.stdout != b.stdout || a.stderr != b.stderr {
        return Outcome::Failed(format!(
            "{label}: semantic mismatch at {}\noriginal={}\ngenerated={}\n{}",
            base.display(),
            String::from_utf8_lossy(&a.stdout),
            String::from_utf8_lossy(&b.stdout),
            String::from_utf8_lossy(&b.stderr)
        ));
    }
    eprintln!("ORACLE {label}: full-class behavioral equality on enumerated inputs");
    fs::remove_dir_all(&base).unwrap();
    Outcome::Equal
}

// Use only for individually adjudicated supported cases; any skipped or rejected
// original/generated class is a hard failure, not a silently passing oracle.
pub fn require_equal(label: &str, class: &str, bytes: &[u8], text: &str, replay: &str) {
    assert!(
        matches!(inspect(label, class, bytes, text, replay), Outcome::Equal),
        "{label}: required a verified original and successful full-class equal replay"
    );
}
// For real safety negatives. Invalid original cannot count as safe rejection.
pub fn require_rejected(label: &str, class: &str, bytes: &[u8], text: &str, replay: &str) {
    assert!(
        matches!(
            inspect(label, class, bytes, text, replay),
            Outcome::Rejected
        ),
        "{label}: required a verified original and uncompileable complete emitted class"
    );
}
