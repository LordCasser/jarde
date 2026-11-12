//! The returned sum of one static int update, over both frozen javac outputs.
//! Whole-class compilation and dual-JVM replay are separate acceptance gates.

use jarde::*;
use std::slice;

const LEGS: &[(&str, &[u8])] = &[
    (
        "javac23 --release8",
        include_bytes!("fixtures/recover-static-field-prefix-return/v8/FO.class"),
    ),
    (
        "javac8",
        include_bytes!("fixtures/recover-static-field-prefix-return/v8-javac8/FO.class"),
    ),
];

fn source(bytes: &[u8]) -> ClassSourceReport {
    source_named(bytes, "FO")
}

fn source_named(bytes: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen physical class must be selected: {other:?}"),
    }
}

fn body<'a>(source: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap();
    assert_eq!(method.item.descriptor.raw().0, b"()I");
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the physical method must have its recovery report: {other:?}"),
    }
}

#[test]
fn static_prefix_returns_the_new_value_on_both_compiler_legs() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let bump = body(&source, "bump");
        assert!(bump.text.contains("return ++FO.i;"), "{leg}: {}", bump.text);
        assert!(!bump.text.contains("@bytecode"), "{leg}: {}", bump.text);
        assert!(!bump.text.contains("FO.i ="), "{leg}: {}", bump.text);
        assert!(!bump.text.contains("FO.i++"), "{leg}: {}", bump.text);
        assert_eq!(bump.content, RecoveryContent::ContainsStatements);
    }
}

#[test]
fn static_prefix_keeps_read_write_and_return_origins() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let bump = body(&source, "bump");
        assert_eq!(
            bump.fields
                .iter()
                .map(|field| (field.bci, field.access, field.presented))
                .collect::<Vec<_>>(),
            [(0, "read", true), (6, "write", true)],
            "{leg}: {}",
            bump.text,
        );
        let read = bump.source_map.direct_of_bci(0);
        assert_eq!(read.len(), 2, "{leg}: {:?}", bump.source_map.segments());
        assert_eq!(
            read.iter()
                .map(|segment| segment.text(&bump.text))
                .collect::<Vec<_>>(),
            ["FO", "FO.i"]
        );
        assert_eq!(
            read[0].origin().primary().provenance(),
            jarde_java::Provenance::Derived
        );
        assert_eq!(
            read[1].origin().primary().provenance(),
            jarde_java::Provenance::Direct
        );
        let physical = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"bump")
            .unwrap()
            .item
            .identity
            .clone();
        assert!(
            bump.source_map
                .segments()
                .iter()
                .flat_map(|segment| std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived()))
                .all(|origin| origin.method() == Some(&physical))
        );
        let write = bump.source_map.direct_of_bci(6);
        assert_eq!(write.len(), 1, "{leg}: {:?}", bump.source_map.segments());
        assert_eq!(write[0].text(&bump.text), "++FO.i");
        for bci in [3, 4, 5] {
            assert!(
                !bump.source_map.derived_of_bci(bci).is_empty(),
                "{leg}: folded producer {bci}: {:?}",
                bump.source_map.segments(),
            );
        }
        assert!(!bump.source_map.direct_of_bci(9).is_empty());
    }
}

#[test]
fn saved_return_still_precedes_the_finally_write() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let saved = body(&source, "evalThenFin");
        assert!(
            saved.text.contains(
                "try {\n        int local0 = bump();\n        return local0;\n    } finally {\n        FO.i = 100;\n    }"
            ),
            "{leg}: {}",
            saved.text,
        );
        assert!(!saved.text.contains("@bytecode"), "{leg}: {}", saved.text);
    }
}

// These classes move the declaration, CP indexes and Code payload without changing the
// accepted six-instruction ownership proof. A wider encoding of the same constant is a
// separate conservative boundary: it shifts BCIs and must keep generic-copy handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CopyCase {
    Prefix,
    WiderOne,
    PlusTwo,
    OtherField,
}

#[derive(Clone, Copy)]
struct CopyNames {
    class: &'static str,
    field: &'static str,
    method: &'static str,
    padding: u16,
}

const COPY_NAMES: &[CopyNames] = &[
    CopyNames {
        class: "Counter",
        field: "count",
        method: "next",
        padding: 0,
    },
    CopyNames {
        class: "RenamedLedger",
        field: "amount",
        method: "advance",
        padding: 9,
    },
    CopyNames {
        class: "RelocatedSequence",
        field: "position",
        method: "step",
        padding: 260,
    },
];

fn copy_fixture(names: CopyNames, case: CopyCase) -> (Vec<u8>, Vec<u32>) {
    fn word(bytes: &mut Vec<u8>, n: u16) {
        bytes.extend(n.to_be_bytes());
    }
    fn push(pool: &mut Vec<Vec<u8>>, tag: u8, indexes: &[u16]) -> u16 {
        let mut bytes = vec![tag];
        for index in indexes {
            word(&mut bytes, *index);
        }
        pool.push(bytes);
        pool.len().try_into().unwrap()
    }
    fn utf(pool: &mut Vec<Vec<u8>>, text: &str) -> u16 {
        let mut bytes = vec![1];
        word(&mut bytes, text.len().try_into().unwrap());
        bytes.extend(text.as_bytes());
        pool.push(bytes);
        pool.len().try_into().unwrap()
    }
    let mut pool = Vec::new();
    for index in 0..names.padding {
        utf(&mut pool, &format!("unused_{index}"));
    }
    let class_name = utf(&mut pool, names.class);
    let class = push(&mut pool, 7, &[class_name]);
    let object_name = utf(&mut pool, "java/lang/Object");
    let object = push(&mut pool, 7, &[object_name]);
    let field_name = utf(&mut pool, names.field);
    let peer_name = utf(&mut pool, "peer");
    let int_type = utf(&mut pool, "I");
    let field_type = push(&mut pool, 12, &[field_name, int_type]);
    let peer_type = push(&mut pool, 12, &[peer_name, int_type]);
    let field = push(&mut pool, 9, &[class, field_type]);
    let peer = push(&mut pool, 9, &[class, peer_type]);
    let method_name = utf(&mut pool, names.method);
    let method_type = utf(&mut pool, "()I");
    let code_name = utf(&mut pool, "Code");
    let mut code = vec![0xb2];
    word(&mut code, field);
    let mut bcis = vec![0, code.len() as u32];
    match case {
        CopyCase::WiderOne => code.extend([0x10, 1]), // bipush 1, not iconst_1
        CopyCase::PlusTwo => code.push(0x05),
        _ => code.push(0x04),
    }
    for opcode in [0x60, 0x59, 0xb3] {
        bcis.push(code.len() as u32);
        code.push(opcode);
    }
    word(
        &mut code,
        if case == CopyCase::OtherField {
            peer
        } else {
            field
        },
    );
    bcis.push(code.len() as u32);
    code.push(0xac);

    let mut bytes = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 52];
    word(&mut bytes, (pool.len() + 1).try_into().unwrap());
    for entry in pool {
        bytes.extend(entry);
    }
    for value in [0x0021, class, object, 0, 2] {
        word(&mut bytes, value);
    }
    for name in [field_name, peer_name] {
        for value in [0x0009, name, int_type, 0] {
            word(&mut bytes, value);
        }
    }
    for value in [1, 0x0009, method_name, method_type, 1, code_name] {
        word(&mut bytes, value);
    }
    bytes.extend((12 + code.len() as u32).to_be_bytes());
    word(&mut bytes, 2); // max_stack
    word(&mut bytes, 0); // max_locals
    bytes.extend((code.len() as u32).to_be_bytes());
    bytes.extend(code);
    for _ in 0..3 {
        word(&mut bytes, 0);
    } // handlers, Code attributes, class attributes
    (bytes, bcis)
}

fn assert_copy_sources(report: &ClassSourceReport, name: &str, bcis: &[u32]) {
    let recovered = body(report, name);
    let physical = &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap()
        .item
        .identity;
    for bci in bcis {
        assert!(
            !recovered.source_map.of_bci(*bci).is_empty(),
            "lost BCI {bci}: {}",
            recovered.text
        );
    }
    assert!(
        recovered
            .source_map
            .segments()
            .iter()
            .flat_map(|segment| std::iter::once(segment.origin().primary())
                .chain(segment.origin().derived()))
            .all(|origin| origin.method() == Some(physical))
    );
}

#[test]
fn static_prefix_ownership_survives_names_pool_indexes_and_file_offsets() {
    for names in COPY_NAMES {
        let (bytes, bcis) = copy_fixture(*names, CopyCase::Prefix);
        let report = source_named(&bytes, names.class);
        let recovered = body(&report, names.method);
        let target = format!("{}.{}", names.class, names.field);
        assert!(
            recovered.text.contains(&format!("return ++{target};")),
            "{}",
            recovered.text
        );
        assert!(!recovered.text.contains("@bytecode"), "{}", recovered.text);
        assert_eq!(
            recovered.text.matches(&target).count(),
            1,
            "{}",
            recovered.text
        );
        let read = recovered.source_map.direct_of_bci(bcis[0]);
        assert_eq!(read.len(), 2, "{}", recovered.text);
        assert_eq!(
            read.iter()
                .map(|segment| segment.text(&recovered.text))
                .collect::<Vec<_>>(),
            [names.class, target.as_str()]
        );
        assert_eq!(
            read[0].origin().primary().provenance(),
            jarde_java::Provenance::Derived
        );
        assert_eq!(
            read[1].origin().primary().provenance(),
            jarde_java::Provenance::Direct
        );
        let write = recovered.source_map.direct_of_bci(bcis[4]);
        assert_eq!(write.len(), 1, "{}", recovered.text);
        assert_eq!(write[0].text(&recovered.text), format!("++{target}"));
        for bci in &bcis[1..4] {
            assert!(!recovered.source_map.derived_of_bci(*bci).is_empty());
        }
        assert!(!recovered.source_map.direct_of_bci(bcis[5]).is_empty());
        assert_copy_sources(&report, names.method, &bcis);
    }
}

#[test]
fn an_unproved_prefix_does_not_steal_the_generic_copy_binding() {
    for names in COPY_NAMES {
        for case in [CopyCase::WiderOne, CopyCase::PlusTwo, CopyCase::OtherField] {
            let (bytes, bcis) = copy_fixture(*names, case);
            let report = source_named(&bytes, names.class);
            let recovered = body(&report, names.method);
            assert!(
                !recovered.text.contains("++"),
                "{case:?}: {}",
                recovered.text
            );
            assert!(
                !recovered.text.contains("@bytecode"),
                "{case:?}: {}",
                recovered.text
            );
            let written = if case == CopyCase::OtherField {
                "peer"
            } else {
                names.field
            };
            assert!(
                recovered
                    .text
                    .contains(&format!("{}.{} =", names.class, written)),
                "{case:?}: {}",
                recovered.text
            );
            assert!(
                recovered.text.contains("return saved"),
                "{case:?}: {}",
                recovered.text
            );
            assert_copy_sources(&report, names.method, &bcis);
        }
    }
}

struct JavaScratch(std::path::PathBuf);

impl JavaScratch {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jarde-static-prefix-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for JavaScratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn java_compile(directory: &std::path::Path, files: &[&str]) {
    let mut command = std::process::Command::new(
        std::env::var_os("JARDE_TEST_JAVAC").unwrap_or_else(|| "javac".into()),
    );
    // Set this only for a real Java 8 compiler, which predates --release.
    if std::env::var_os("JARDE_TEST_JAVAC8").is_some() {
        command.args(["-source", "8", "-target", "8"]);
    } else {
        command.args(["--release", "8"]);
    }
    let output = command
        .args(["-Xlint:-options", "-cp"])
        .arg(directory)
        .arg("-d")
        .arg(directory)
        .args(files.iter().map(|file| directory.join(file)))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "javac failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn java_run(directory: &std::path::Path, class: &str, args: &[&str]) -> Vec<u8> {
    let output = std::process::Command::new(
        std::env::var_os("JARDE_TEST_JAVA").unwrap_or_else(|| "java".into()),
    )
    .args(["-Xverify:all", "-cp"])
    .arg(directory)
    .arg(class)
    .args(args)
    .output()
    .unwrap();
    assert!(
        output.status.success(),
        "java failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected java stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
#[ignore = "requires a JDK: compiles whole unedited Engine classes and compares verified JVM behavior"]
fn complete_prefix_classes_and_unowned_controls_match_original_jvm_behavior() {
    const RUNNER: &str = r#"
public class CopyOracle {
    public static void main(String[] args) throws Exception {
        Class<?> type = Class.forName(args[0]);
        java.lang.reflect.Field field = type.getField(args[1]);
        java.lang.reflect.Field peer = type.getField("peer");
        java.lang.reflect.Method step = type.getMethod(args[2]);
        for (int start : new int[]{-1, 7, Integer.MIN_VALUE, Integer.MAX_VALUE}) {
            field.setInt(null, start);
            peer.setInt(null, 91);
            Object result = step.invoke(null);
            System.out.println(result + "/" + field.getInt(null) + "/" + peer.getInt(null));
        }
    }
}
"#;
    for names in COPY_NAMES {
        for case in [
            CopyCase::Prefix,
            CopyCase::WiderOne,
            CopyCase::PlusTwo,
            CopyCase::OtherField,
        ] {
            let (bytes, _) = copy_fixture(*names, case);
            let original = JavaScratch::new();
            let recovered = JavaScratch::new();
            std::fs::write(original.0.join(format!("{}.class", names.class)), &bytes).unwrap();
            let report = source_named(&bytes, names.class);
            std::fs::write(
                recovered.0.join(format!("{}.java", names.class)),
                &report.text,
            )
            .unwrap();
            for directory in [&original.0, &recovered.0] {
                std::fs::write(directory.join("CopyOracle.java"), RUNNER).unwrap();
            }
            java_compile(&original.0, &["CopyOracle.java"]);
            java_compile(
                &recovered.0,
                &[&format!("{}.java", names.class), "CopyOracle.java"],
            );
            let args = [names.class, names.field, names.method];
            let expected = [-1_i32, 7, i32::MIN, i32::MAX]
                .into_iter()
                .map(|start| {
                    let sum = start.wrapping_add(if case == CopyCase::PlusTwo { 2 } else { 1 });
                    let (field, peer) = if case == CopyCase::OtherField {
                        (start, sum)
                    } else {
                        (sum, 91)
                    };
                    format!("{sum}/{field}/{peer}\n")
                })
                .collect::<String>();
            let original_output = java_run(&original.0, "CopyOracle", &args);
            assert_eq!(
                original_output,
                expected.as_bytes(),
                "{} / {case:?}",
                names.class
            );
            assert_eq!(
                java_run(&recovered.0, "CopyOracle", &args),
                original_output,
                "{} / {case:?}: {}",
                names.class,
                report.text
            );
        }
    }
    // The frozen complete FO class includes the finally path and its main, not only bump.
    for (leg, bytes) in LEGS {
        let original = JavaScratch::new();
        let recovered = JavaScratch::new();
        std::fs::write(original.0.join("FO.class"), bytes).unwrap();
        let report = source(bytes);
        std::fs::write(recovered.0.join("FO.java"), &report.text).unwrap();
        for directory in [&original.0, &recovered.0] {
            std::fs::write(
                directory.join("StaticPrefixProbe.java"),
                include_str!("fixtures/recover-static-field-prefix-return/StaticPrefixProbe.java"),
            )
            .unwrap();
        }
        java_compile(&original.0, &["StaticPrefixProbe.java"]);
        java_compile(&recovered.0, &["FO.java", "StaticPrefixProbe.java"]);
        for (main, expected) in [
            (
                "FO",
                include_bytes!("fixtures/recover-static-field-prefix-return/main.stdout")
                    .as_slice(),
            ),
            (
                "StaticPrefixProbe",
                include_bytes!("fixtures/recover-static-field-prefix-return/probe.stdout")
                    .as_slice(),
            ),
        ] {
            let original_output = java_run(&original.0, main, &[]);
            assert_eq!(original_output, expected, "{leg}/{main}");
            assert_eq!(
                java_run(&recovered.0, main, &[]),
                original_output,
                "{leg}/{main}: {}",
                report.text
            );
        }
    }
}
