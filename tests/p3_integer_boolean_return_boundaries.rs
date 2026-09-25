//! A genuine major-49 operand-stack switch join with a shared `ireturn Z`.
use jarde::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    slice,
    time::{SystemTime, UNIX_EPOCH},
};
const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-integer-boolean-returns/v8/actual-stack-join/ActualStackJoin.class"
);
const RUNNER: &str = include_str!(
    "fixtures/p3-integer-boolean-returns/actual-stack-join/ActualStackJoinRunner.java"
);
const EXPECTED: &str =
    include_str!("fixtures/p3-integer-boolean-returns/v8/actual-stack-join/expected.txt");
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
fn report() -> ClassSourceReport {
    let mut b = Budget::new(limits());
    let s = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut b)
        .unwrap();
    let q = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ActualStackJoin"),
        },
        environment: EnvironmentRequest {
            snapshot: s.id().clone(),
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
            slice::from_ref(&s),
            &q,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .unwrap()
    {
        OperationOutcome::Performed(r) => r,
        x => panic!("{x:?}"),
    }
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "jarde-intbool-stack-{}-{}",
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
fn shared_switch_operand_arms_and_real_ireturn_are_distinct_origins() {
    let s = Scratch::new();
    let cdir = s.path().join("classes");
    fs::create_dir_all(&cdir).unwrap();
    fs::write(cdir.join("ActualStackJoin.class"), CLASS).unwrap();
    fs::write(s.path().join("ActualStackJoinRunner.java"), RUNNER).unwrap();
    let c = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(&cdir)
        .args(["-d"])
        .arg(&cdir)
        .arg("ActualStackJoinRunner.java")
        .current_dir(s.path())
        .output()
        .unwrap();
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
    let o = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&cdir)
        .arg("ActualStackJoinRunner")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(String::from_utf8(o.stdout).unwrap(), EXPECTED);
    let r = report();
    let m = r
        .methods
        .iter()
        .find(|m| m.item.name.raw().0 == b"runByte")
        .unwrap();
    assert_eq!(m.item.descriptor.raw().0, b"(I)Z");
    let ClassSourceOutcome::Recovered { report: b, .. } = &m.outcome else {
        panic!("{:?}", m.outcome)
    };
    assert_eq!(b.representation, Representation::Java, "{}", b.text);
    assert_eq!(b.quality, Quality::Structured, "{}", b.text);
    assert!(!b.text.contains("@bytecode"));
    for bci in [1, 20, 26, 32] {
        assert!(
            !b.text_of_bci(bci).is_empty(),
            "missing origin BCI {bci}: {}",
            b.text
        )
    }
    assert!(
        !b.source_map.direct_of_bci(32).is_empty(),
        "shared real ireturn source absent: {:?}",
        b.source_map.segments()
    );
}
