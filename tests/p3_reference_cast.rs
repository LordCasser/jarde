//! `recover-explicit-reference-casts`: ordinary `checkcast` instructions stay visible at their
//! existing consumer, including arrays, locals, calls and nested checks.
//!
//! The committed class is javac 23.0.1 output for `--release 8 -g:none`. The positive assertions
//! describe the recovery contract. The ignored JDK check compiles the complete
//! `Engine::class_source` text; its runner is created in a temporary directory and no hand-written
//! replacement body is used.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-reference-cast/v8/ReferenceCastProbe.class");
const SOURCE: &str = include_str!("fixtures/p3-reference-cast/ReferenceCastProbe.java");

const METHODS: [(&[u8], &[u8]); 12] = [
    (b"direct", b"(Ljava/lang/Object;)Ljava/lang/String;"),
    (b"receiver", b"(Ljava/lang/Object;)I"),
    (b"intArrayRead", b"(Ljava/lang/Object;I)I"),
    (b"stringArray", b"(Ljava/lang/Object;)[Ljava/lang/String;"),
    (b"multiArray", b"(Ljava/lang/Object;)[[Ljava/lang/String;"),
    (b"local", b"(Ljava/lang/Object;)Ljava/lang/String;"),
    (
        b"staticParameter",
        b"(Ljava/lang/Object;)Ljava/lang/String;",
    ),
    (b"callOnce", b"()Ljava/lang/String;"),
    (b"nested", b"(Ljava/lang/Object;)Ljava/lang/Number;"),
    (b"nestedCall", b"()Z"),
    (
        b"discardedCastBeforeCall",
        b"(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    (b"source", b"()Ljava/lang/Object;"),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed reference-cast fixture opens as a standalone CLASS")
}

fn class_source_of(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ReferenceCastProbe"),
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
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one committed sample answers one definition, got {} candidate(s)",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one committed sample answers one definition, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

fn member_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the fixture's method table"))
}

fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &member_of(report, name).text
}

fn recovered_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member_of(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` has no recovered body: {other:?}"),
    }
}

fn assert_mapped(report: &ClassSourceReport, name: &str, bcis: &[u32]) {
    let recovered = recovered_of(report, name);
    for bci in bcis {
        assert!(
            !recovered.source_map.of_bci(*bci).is_empty(),
            "`{name}` has no source-map segment for BCI {bci}: {recovered:?}"
        );
    }
}

/// Replaces one same-width instruction sequence in the committed class.  This keeps all BCIs
/// stable while creating a refusal path that Java source cannot spell directly.
fn patched(bytes: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    assert_eq!(needle.len(), replacement.len(), "a patch keeps every BCI");
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0usize;
    let mut sites = 0usize;
    while at < bytes.len() {
        if bytes[at..].starts_with(needle) {
            out.extend_from_slice(replacement);
            at += needle.len();
            sites += 1;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    assert_eq!(
        sites, 1,
        "the fixture holds the sequence this case patches once"
    );
    out
}

fn discarded_store_as_pop(bytes: &[u8]) -> Vec<u8> {
    // discardedCastBeforeCall is `aload_0; checkcast String; astore_1; invokestatic source;
    // areturn`.  The same-width patch makes the cast's value explicitly unused while preserving
    // both the checkcast and the producer call as separate bytecode sources.
    patched(
        bytes,
        &[0x2a, 0xc0, 0x00, 0x07, 0x4c, 0xb8, 0x00, 0x19, 0xb0],
        &[0x2a, 0xc0, 0x00, 0x07, 0x57, 0xb8, 0x00, 0x19, 0xb0],
    )
}

#[test]
fn fixture_declares_the_cast_shapes_and_real_java_source() {
    assert!(SOURCE.contains("(Number) (Runnable) value"));
    let report = class_source_of(&open(FIXTURE));
    for (name, descriptor) in METHODS {
        let method = report
            .methods
            .iter()
            .find(|member| {
                member.item.name.raw().0.as_slice() == name
                    && member.item.descriptor.raw().0.as_slice() == descriptor
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing `{}` {:?}",
                    String::from_utf8_lossy(name),
                    descriptor
                )
            });
        assert!(
            matches!(method.item.body, MemberBodyEvidence::CodeAttribute { .. }),
            "`{}` has a Code attribute",
            String::from_utf8_lossy(name)
        );
    }
}

#[test]
fn ordinary_reference_casts_are_written_at_each_consumer() {
    let report = class_source_of(&open(FIXTURE));
    let expected = [
        ("direct", "return (java.lang.String) arg0;"),
        ("receiver", "return ((java.lang.String) arg0).length();"),
        ("intArrayRead", "return ((int[]) arg0)[arg1];"),
        ("stringArray", "return (java.lang.String[]) arg0;"),
        ("multiArray", "return (java.lang.String[][]) arg0;"),
        (
            "local",
            "java.lang.String local1 = (java.lang.String) arg0;",
        ),
        (
            "staticParameter",
            "return consume((java.lang.String) arg0);",
        ),
        ("callOnce", "return (java.lang.String) source();"),
    ];
    for (name, snippet) in expected {
        let text = text_of(&report, name);
        assert!(
            !text.contains("@bytecode"),
            "`{name}` is still quoted:\n{text}"
        );
        assert!(
            text.contains(snippet),
            "`{name}` loses its cast consumer:\n{text}"
        );
    }
    assert_mapped(&report, "direct", &[0, 1, 4]);
}

#[test]
fn nested_casts_keep_the_intermediate_runtime_check() {
    let report = class_source_of(&open(FIXTURE));
    let nested = text_of(&report, "nested");
    let body_start = nested.find("return ").expect("nested has a return");
    let body = &nested[body_start..];
    assert!(
        body.contains("return (java.lang.Number) (java.lang.Runnable) arg0;"),
        "the nested source spelling preserves inner Runnable evaluation before outer Number check: {body}"
    );
    assert!(
        body.contains("return (java.lang.Number)"),
        "the outer Number cast is explicit: {body}"
    );
    assert!(
        body.contains("(java.lang.Runnable)"),
        "the inner Runnable cast is explicit: {body}"
    );
    assert!(
        body.contains("arg0;"),
        "the original operand remains at the leaf: {body}"
    );
    assert_mapped(&report, "nested", &[0, 1, 4, 7]);
}

#[test]
fn discarded_cast_patch_retains_check_and_call_sources() {
    let patched = discarded_store_as_pop(FIXTURE);
    let snapshot = open(&patched);
    let report = class_source_of(&snapshot);
    let text = text_of(&report, "discardedCastBeforeCall");
    assert!(
        text.contains("@bytecode 1"),
        "the discarded checkcast remains named:\n{text}"
    );
    assert!(
        text.contains("@bytecode 4"),
        "the discarded pop remains named:\n{text}"
    );
    assert!(
        text.contains("return source();") && text.matches("source()").count() == 1,
        "the independent static call remains recovered exactly once:\n{text}"
    );
    assert!(
        !text.contains("java.lang.String) arg0).source"),
        "an ordinary discarded cast must not become an invalid static-call qualifier:\n{text}"
    );
    assert_mapped(&report, "discardedCastBeforeCall", &[1, 4, 5]);
}

#[test]
fn nested_instanceof_consumer_retains_invocation_and_both_cast_bcis() {
    let report = class_source_of(&open(FIXTURE));
    let text = text_of(&report, "nestedCall");
    assert!(
        !text.contains("@bytecode"),
        "the nested consumer is fully recovered:\n{text}"
    );
    assert!(
        text.contains(
            "return (java.lang.Object) (java.lang.Number) (java.lang.Runnable) source() instanceof java.lang.Comparable;"
        ),
        "the producer, both runtime casts and instanceof remain in evaluation order:\n{text}"
    );
    assert_mapped(&report, "nestedCall", &[0, 3, 6, 9]);
    let recovered = recovered_of(&report, "nestedCall");
    let identity = &member_of(&report, "nestedCall").item.identity;
    for bci in [0, 3, 6, 9] {
        let segments = recovered.source_map.direct_of_bci(bci);
        assert!(
            segments
                .iter()
                .any(|segment| segment.origin().primary().method() == Some(identity)),
            "nestedCall BCI {bci} lacks a direct source segment owned by {identity:?}: {segments:?}"
        );
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-reference-cast-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the JDK comparison directory");
        Self(path)
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

const RUNNER: &str = r#"
public final class ReferenceCastRunner {
    private interface Call { Object run(); }

    private static void print(String name, Call call) {
        try {
            System.out.println(name + "=" + call.run());
        } catch (Throwable error) {
            System.out.println(name + "=" + error.getClass().getName());
        }
    }

    private static final class Both extends Number implements Runnable {
        public void run() {}
        public int intValue() { return 9; }
        public long longValue() { return 9L; }
        public float floatValue() { return 9.0f; }
        public double doubleValue() { return 9.0; }
    }

    public static void main(String[] args) {
        ReferenceCastProbe.calls = 0;
        print("direct", new Call() { public Object run() { return ReferenceCastProbe.direct("ok"); } });
        print("receiver", new Call() { public Object run() { return ReferenceCastProbe.receiver("abcd"); } });
        print("intArrayRead", new Call() { public Object run() { return ReferenceCastProbe.intArrayRead(new int[] {3, 5}, 1); } });
        print("stringArray", new Call() { public Object run() { return ReferenceCastProbe.stringArray(new String[] {"a"}).length; } });
        print("multiArray", new Call() { public Object run() { return ReferenceCastProbe.multiArray(new String[][] { {"a"} }).length; } });
        print("local", new Call() { public Object run() { return ReferenceCastProbe.local("local"); } });
        print("staticParameter", new Call() { public Object run() { return ReferenceCastProbe.staticParameter("parameter"); } });
        print("callOnce", new Call() { public Object run() { return ReferenceCastProbe.callOnce(); } });
        print("calls", new Call() { public Object run() { return ReferenceCastProbe.calls; } });
        ReferenceCastProbe.calls = 0;
        print("nestedCall", new Call() { public Object run() { return ReferenceCastProbe.nestedCall(); } });
        print("nestedCallCalls", new Call() { public Object run() { return ReferenceCastProbe.calls; } });
        print("directNull", new Call() { public Object run() { return ReferenceCastProbe.direct(null); } });
        print("directWrong", new Call() { public Object run() { return ReferenceCastProbe.direct(Integer.valueOf(1)); } });
        print("receiverNull", new Call() { public Object run() { return ReferenceCastProbe.receiver(null); } });
        print("intArrayWrong", new Call() { public Object run() { return ReferenceCastProbe.intArrayRead(new String[] {"wrong"}, 0); } });
        print("stringArrayWrong", new Call() { public Object run() { return ReferenceCastProbe.stringArray(new Object[] {"wrong"}); } });
        print("stringArrayNull", new Call() { public Object run() { return ReferenceCastProbe.stringArray(null); } });
        print("nestedBoth", new Call() { public Object run() { return ReferenceCastProbe.nested(new Both()).intValue(); } });
        print("nestedInteger", new Call() { public Object run() { return ReferenceCastProbe.nested(Integer.valueOf(1)); } });
        print("nestedNull", new Call() { public Object run() { return ReferenceCastProbe.nested(null); } });
    }
}
"#;

fn javac(dir: &Path, files: &[&str]) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-d")
        .arg(dir)
        .args(files)
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling generated recovery failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg("ReferenceCastRunner")
        .current_dir(dir)
        .output()
        .expect("execute the fixture runner");
    assert!(
        output.status.success(),
        "the fixture runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the fixture runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile Engine::class_source output and execute a temporary runner"]
fn recovered_reference_casts_match_original_runtime() {
    let report = class_source_of(&open(FIXTURE));
    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered = scratch.path().join("recovered");
    fs::create_dir_all(&original).expect("create the original comparison directory");
    fs::create_dir_all(&recovered).expect("create the recovered comparison directory");

    fs::write(original.join("ReferenceCastProbe.class"), FIXTURE)
        .expect("write the original fixture class");
    fs::write(original.join("ReferenceCastRunner.java"), RUNNER)
        .expect("write the original runner scaffold");
    javac(&original, &["ReferenceCastRunner.java"]);
    let original_output = run_runner(&original);

    fs::write(recovered.join("ReferenceCastProbe.java"), &report.text)
        .expect("write the generated recovery text");
    fs::write(recovered.join("ReferenceCastRunner.java"), RUNNER)
        .expect("write the recovered runner scaffold");
    javac(
        &recovered,
        &["ReferenceCastProbe.java", "ReferenceCastRunner.java"],
    );
    let recovered_output = run_runner(&recovered);

    for marker in [
        "calls=",
        "nestedCall=java.lang.ClassCastException",
        "nestedCallCalls=1",
        "directNull=null",
        "directWrong=java.lang.ClassCastException",
        "receiverNull=java.lang.NullPointerException",
        "intArrayWrong=java.lang.ClassCastException",
        "stringArrayWrong=java.lang.ClassCastException",
        "stringArrayNull=null",
        "nestedBoth=9",
        "nestedInteger=java.lang.ClassCastException",
        "nestedNull=null",
    ] {
        assert!(
            original_output.contains(marker),
            "the original runner covers `{marker}`:\n{original_output}"
        );
    }
    assert_eq!(
        recovered_output, original_output,
        "the genuine recovered bodies preserve the original runtime output"
    );
}
