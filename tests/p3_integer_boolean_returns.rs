//! Java 8 integer-valued code with verifier-valid `ireturn Z` descriptors.
use jarde::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    slice,
    time::{SystemTime, UNIX_EPOCH},
};
const CLASS: &[u8] =
    include_bytes!("fixtures/p3-integer-boolean-returns/v8/IntegerBooleanReturns.class");
const RUNNER: &str =
    include_str!("fixtures/p3-integer-boolean-returns/IntegerBooleanReturnsRunner.java");
const EXPECTED: &str = include_str!("fixtures/p3-integer-boolean-returns/v8/expected.txt");
fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}
fn report(bytes: &[u8]) -> ClassSourceReport {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("IntegerBooleanReturns"),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".into()),
        },
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .unwrap()
    {
        OperationOutcome::Performed(r) => r,
        other => panic!("unexpected selection: {other:?}"),
    }
}
fn member<'a>(r: &'a ClassSourceReport, n: &str) -> &'a ClassSourceMethod {
    r.methods
        .iter()
        .find(|m| m.item.name.raw().0 == n.as_bytes())
        .unwrap()
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "jarde-intbool-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn permanent_class_verifies_and_exposes_all_required_integer_boolean_shapes() {
    let s = Scratch::new();
    let classes = s.path().join("classes");
    fs::create_dir_all(&classes).unwrap();
    fs::write(classes.join("IntegerBooleanReturns.class"), CLASS).unwrap();
    fs::write(s.path().join("IntegerBooleanReturnsRunner.java"), RUNNER).unwrap();
    let c = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(&classes)
        .args(["-d"])
        .arg(&classes)
        .arg("IntegerBooleanReturnsRunner.java")
        .current_dir(s.path())
        .output()
        .unwrap();
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
    let o = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&classes)
        .arg("IntegerBooleanReturnsRunner")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(String::from_utf8(o.stdout).unwrap(), EXPECTED);
    let r = report(CLASS);
    for (name, ret_bci) in [
        ("direct", 1),
        ("once", 5),
        ("post", 11),
        ("pre", 11),
        ("sync", 1),
        ("syncOn", 7),
    ] {
        let m = member(&r, name);
        let ClassSourceOutcome::Recovered { report: b, .. } = &m.outcome else {
            panic!("{name}: {:?}", m.outcome)
        };
        assert_eq!(b.representation, Representation::Java, "{name}: {}", b.text);
        assert_eq!(b.quality, Quality::Structured, "{name}: {}", b.text);
        assert!(!b.text.contains("@bytecode"), "{name}: {}", b.text);
        assert!(
            !b.source_map.direct_of_bci(ret_bci).is_empty(),
            "ireturn BCI {ret_bci} missing in {name}: {:?}",
            b.source_map.segments()
        );
    }
    let once = member(&r, "once");
    let ClassSourceOutcome::Recovered { report: b, .. } = &once.outcome else {
        unreachable!()
    };
    assert!(
        !b.text_of_bci(2).is_empty(),
        "call site origin preserved: {}",
        b.text
    );
    for name in ["post", "pre"] {
        let m = member(&r, name);
        let ClassSourceOutcome::Recovered { report: b, .. } = &m.outcome else {
            unreachable!()
        };
        for bci in [5, 8, 11] {
            assert!(
                !b.text_of_bci(bci).is_empty(),
                "{name} misses field/update/return BCI {bci}: {}",
                b.text
            )
        }
    }
}

#[test]
fn complete_recovered_fixture_is_zero_quote_and_recompiles_to_same_execution() {
    let r = report(CLASS);
    for m in &r.methods {
        let ClassSourceOutcome::Recovered { report: b, .. } = &m.outcome else {
            panic!(
                "{} refused: {:?}",
                String::from_utf8_lossy(&m.item.name.raw().0),
                m.outcome
            )
        };
        assert_eq!(
            b.representation,
            Representation::Java,
            "{}: {}",
            String::from_utf8_lossy(&m.item.name.raw().0),
            b.text
        );
        assert_eq!(b.quality, Quality::Structured);
        assert!(
            !b.text.contains("@bytecode"),
            "quoted method {}: {}",
            String::from_utf8_lossy(&m.item.name.raw().0),
            b.text
        );
    }
    let s = Scratch::new();
    fs::write(s.path().join("IntegerBooleanReturns.java"), &r.text).unwrap();
    fs::write(s.path().join("IntegerBooleanReturnsRunner.java"), RUNNER).unwrap();
    let c = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(s.path().join("classes"))
        .arg("IntegerBooleanReturns.java")
        .current_dir(s.path())
        .output()
        .unwrap();
    assert!(
        c.status.success(),
        "recovered class failed javac: {}\n{}",
        String::from_utf8_lossy(&c.stderr),
        r.text
    );
    let c = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(s.path().join("classes"))
        .args(["-d"])
        .arg(s.path().join("classes"))
        .arg("IntegerBooleanReturnsRunner.java")
        .current_dir(s.path())
        .output()
        .unwrap();
    assert!(
        c.status.success(),
        "runner failed javac: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let o = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(s.path().join("classes"))
        .arg("IntegerBooleanReturnsRunner")
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "recovered JVM failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert_eq!(String::from_utf8(o.stdout).unwrap(), EXPECTED);
}
