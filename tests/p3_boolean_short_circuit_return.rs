//! Proved 0/1 short-circuit return chains keep Java's evaluation order as `&&` and `||`.

use jarde::*;
use std::{fs, path::PathBuf, process::Command, slice, time::SystemTime};

const CLASS: &[u8] = include_bytes!("fixtures/p3-boolean-short-circuit-return/v8/BoolValue.class");
const EXPECTED: &str = include_str!("fixtures/p3-boolean-short-circuit-return/expected.txt");
const RUNNER: &str = include_str!("fixtures/p3-boolean-short-circuit-return/Runner.java");

fn budget() -> Budget {
    task_budget(&[]).expect("bounded defaults")
}

fn class_source(bytes: &[u8]) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("BoolValue"),
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
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(task_budget(&[]).expect("bounded defaults").limits().clone()),
        )
        .expect("class-source request runs")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected operation outcome: {other:?}"),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing method {name}"))
}

fn recovered_body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let ClassSourceOutcome::Recovered { report, .. } = &method(report, name).outcome else {
        panic!("{name} did not recover: {:?}", method(report, name).outcome)
    };
    report
}

fn run_class_source(source: &str, label: &str) -> String {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("jarde-bool-return-{label}-{nonce}"));
    let classes = root.join("classes");
    fs::create_dir_all(&classes).unwrap();
    fs::write(root.join("BoolValue.java"), source).unwrap();
    fs::write(root.join("Runner.java"), RUNNER).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(&classes)
        .args(["-d"])
        .arg(&classes)
        .args(["BoolValue.java", "Runner.java"])
        .current_dir(&root)
        .output()
        .expect("javac is installed");
    assert!(
        compile.status.success(),
        "javac failed:\n{}\n{}",
        source,
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&classes)
        .arg("Runner")
        .output()
        .expect("java is installed");
    assert!(
        run.status.success(),
        "java failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let output = String::from_utf8(run.stdout).expect("output is UTF-8");
    let _ = fs::remove_dir_all(PathBuf::from(root));
    output
}

#[test]
fn return_consumers_use_short_circuit_operators_and_keep_all_origins() {
    let first = class_source(CLASS);
    let second = class_source(CLASS);
    assert_eq!(
        first.text, second.text,
        "fresh-budget replay is deterministic"
    );
    for (name, expected, bcis) in [
        ("and", "return arg0 > 0 && arg1 > 0;", [1, 5, 8, 12, 13]),
        ("or", "return arg0 > 0 || arg1 > 0;", [1, 5, 8, 12, 13]),
        (
            "effectfulAnd",
            "return positive(arg0) && positive(arg1);",
            [4, 11, 14, 18, 19],
        ),
        (
            "effectfulOr",
            "return positive(arg0) || positive(arg1);",
            [4, 11, 14, 18, 19],
        ),
    ] {
        let body = recovered_body(&first, name);
        let replay = recovered_body(&second, name);
        assert_eq!(body.text, replay.text, "{name} text changed on replay");
        assert_eq!(
            source_map_shape(&body.source_map),
            source_map_shape(&replay.source_map),
            "{name} mappings changed on replay"
        );
        assert!(body.text.contains(expected), "{name}: {}", body.text);
        assert!(!body.text.contains("?"), "{name}: {}", body.text);
        assert!(!body.text.contains("% 2 != 0"), "{name}: {}", body.text);
        for bci in bcis {
            assert!(
                !body.source_map.of_bci(bci).is_empty(),
                "{name} has no source mapping for BCI {bci}: {:?}",
                body.source_map.segments()
            );
        }
    }
    assert_eq!(run_original_class(), EXPECTED);
    assert_eq!(run_class_source(&first.text, "recovered"), EXPECTED);
}

#[test]
fn integer_return_or_non_boolean_leaves_keep_the_numeric_conversion() {
    let mut int_return = CLASS.to_vec();
    let bool_descriptor = b"(II)Z";
    let positions = int_return
        .windows(bool_descriptor.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == bool_descriptor).then_some(index))
        .collect::<Vec<_>>();
    let [position] = positions.as_slice() else {
        panic!("the `and` method descriptor must occur once: {positions:?}");
    };
    int_return[*position + 4] = b'I';
    let int_report = class_source(&int_return);
    let int_body = recovered_body(&int_report, "and");
    assert!(!int_body.text.contains("&&"), "{}", int_body.text);
    assert!(!int_body.text.contains("||"), "{}", int_body.text);

    // The verifier's int-shaped producer values 2 and 3 both flow into `ireturn Z`. Their low bit
    // still decides the returned boolean, so the exact-0/1 projection must decline this tree.
    let mut non_boolean_leaves = CLASS.to_vec();
    let bytecode = [
        0x1a, 0x9e, 0x00, 0x0b, 0x1b, 0x9e, 0x00, 0x07, 0x04, 0xa7, 0x00, 0x04, 0x03, 0xac,
    ];
    let starts = non_boolean_leaves
        .windows(bytecode.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == bytecode).then_some(index))
        .collect::<Vec<_>>();
    let [start] = starts.as_slice() else {
        panic!("the `and` bytecode must occur once: {starts:?}");
    };
    non_boolean_leaves[*start + 8] = 0x05; // iconst_2
    non_boolean_leaves[*start + 12] = 0x06; // iconst_3
    let raw_report = class_source(&non_boolean_leaves);
    let raw_body = recovered_body(&raw_report, "and");
    assert!(!raw_body.text.contains("&&"), "{}", raw_body.text);
    assert!(!raw_body.text.contains("||"), "{}", raw_body.text);
}

fn source_map_shape(map: &SourceMap) -> Vec<(usize, usize, u32, Vec<u32>)> {
    map.segments()
        .iter()
        .map(|segment| {
            (
                segment.start(),
                segment.end(),
                segment.origin().primary().bci(),
                segment
                    .origin()
                    .derived()
                    .iter()
                    .map(|origin| origin.bci())
                    .collect(),
            )
        })
        .collect()
}

fn run_original_class() -> String {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("jarde-bool-return-original-{nonce}"));
    let classes = root.join("classes");
    fs::create_dir_all(&classes).unwrap();
    fs::write(classes.join("BoolValue.class"), CLASS).unwrap();
    fs::write(root.join("Runner.java"), RUNNER).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(&classes)
        .args(["-d"])
        .arg(&classes)
        .arg("Runner.java")
        .current_dir(&root)
        .output()
        .expect("javac is installed");
    assert!(
        compile.status.success(),
        "javac failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&classes)
        .arg("Runner")
        .output()
        .expect("java is installed");
    assert!(
        run.status.success(),
        "original java failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let output = String::from_utf8(run.stdout).expect("output is UTF-8");
    let _ = fs::remove_dir_all(root);
    output
}
