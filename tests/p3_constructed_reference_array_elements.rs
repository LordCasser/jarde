//! Complete positive integration fixture for inline constructors stored in fresh reference arrays.
//!
//! The frozen class families were built on Corretto 8 and OpenJDK 23. This test analyzes both
//! inputs, then compiles and runs the generated source with the host JDK selected by JARDE_JAVAC23
//! / JARDE_JAVA23 (or PATH); that host check is not a cross-JDK candidate replay.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const STORE: u16 = 0;
static NEXT: AtomicU64 = AtomicU64::new(0);
const SOURCE_NAMES: [&str; 7] = [
    "Main.java",
    "Base.java",
    "Mid.java",
    "DirectA.java",
    "DirectB.java",
    "TwoHop.java",
    "LocalInterface.java",
];
const CLASS_NAMES: [&str; 7] = [
    "Main",
    "Base",
    "Mid",
    "DirectA",
    "DirectB",
    "TwoHop",
    "LocalInterface",
];

const JAVAC8_CLASSES: &[(&str, &[u8])] = &[
    (
        "Main.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/Main.class"
        ),
    ),
    (
        "Base.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/Base.class"
        ),
    ),
    (
        "Mid.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/Mid.class"
        ),
    ),
    (
        "DirectA.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/DirectA.class"
        ),
    ),
    (
        "DirectB.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/DirectB.class"
        ),
    ),
    (
        "TwoHop.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/TwoHop.class"
        ),
    ),
    (
        "LocalInterface.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac8/classes/LocalInterface.class"
        ),
    ),
];
const JAVAC23_CLASSES: &[(&str, &[u8])] = &[
    (
        "Main.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/Main.class"
        ),
    ),
    (
        "Base.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/Base.class"
        ),
    ),
    (
        "Mid.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/Mid.class"
        ),
    ),
    (
        "DirectA.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/DirectA.class"
        ),
    ),
    (
        "DirectB.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/DirectB.class"
        ),
    ),
    (
        "TwoHop.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/TwoHop.class"
        ),
    ),
    (
        "LocalInterface.class",
        include_bytes!(
            "fixtures/p3-constructed-reference-array-elements-v1/javac23/classes/LocalInterface.class"
        ),
    ),
];

const JAVAC8_STDOUT: &[u8] = include_bytes!(
    "fixtures/p3-constructed-reference-array-elements-v1/oracle/javac8/original-main.stdout"
);
const JAVAC8_STDERR: &[u8] = include_bytes!(
    "fixtures/p3-constructed-reference-array-elements-v1/oracle/javac8/original-main.stderr"
);
const JAVAC23_STDOUT: &[u8] = include_bytes!(
    "fixtures/p3-constructed-reference-array-elements-v1/oracle/javac23/original-main.stdout"
);
const JAVAC23_STDERR: &[u8] = include_bytes!(
    "fixtures/p3-constructed-reference-array-elements-v1/oracle/javac23/original-main.stderr"
);

const CONTROL_JAVAC8_NESTED: &[(&str, &[u8])] = &[(
    "NestedControls.class",
    include_bytes!(
        "fixtures/p3-constructed-reference-array-controls-v1/javac8/NestedControls.class"
    ),
)];
const CONTROL_JAVAC23_NESTED: &[(&str, &[u8])] = &[(
    "NestedControls.class",
    include_bytes!(
        "fixtures/p3-constructed-reference-array-controls-v1/javac23/NestedControls.class"
    ),
)];
const CONTROL_JAVAC8_BOUNDARY: &[(&str, &[u8])] = &[(
    "BoundaryControls.class",
    include_bytes!(
        "fixtures/p3-constructed-reference-array-controls-v1/javac8/BoundaryControls.class"
    ),
)];
const CONTROL_JAVAC23_BOUNDARY: &[(&str, &[u8])] = &[(
    "BoundaryControls.class",
    include_bytes!(
        "fixtures/p3-constructed-reference-array-controls-v1/javac23/BoundaryControls.class"
    ),
)];

#[derive(Clone, Copy)]
struct TargetMethod {
    name: &'static str,
    allocations: [(&'static str, u32, u32, u32); 2],
    other_dups: &'static [u32],
    stores: &'static [u32],
}

const TARGETS: &[TargetMethod] = &[
    TargetMethod {
        name: "sequence",
        allocations: [
            ("java/lang/StringBuilder", 6, 9, 15),
            ("java/lang/StringBuffer", 21, 24, 30),
        ],
        other_dups: &[4, 19],
        stores: &[18, 33],
    },
    TargetMethod {
        name: "collections",
        allocations: [
            ("java/util/ArrayList", 6, 9, 25),
            ("java/util/HashSet", 31, 34, 50),
        ],
        other_dups: &[4, 14, 29, 39],
        stores: &[21, 28, 46, 53],
    },
    TargetMethod {
        name: "failures",
        allocations: [
            ("java/lang/IllegalStateException", 6, 9, 15),
            ("java/lang/IllegalArgumentException", 21, 24, 30),
        ],
        other_dups: &[4, 19],
        stores: &[18, 33],
    },
    TargetMethod {
        name: "ownDirect",
        allocations: [("DirectA", 6, 9, 15), ("DirectB", 21, 24, 30)],
        other_dups: &[4, 19],
        stores: &[18, 33],
    },
    TargetMethod {
        name: "ownTwoHop",
        allocations: [("TwoHop", 6, 9, 15), ("DirectB", 21, 24, 30)],
        other_dups: &[4, 19],
        stores: &[18, 33],
    },
    TargetMethod {
        name: "ownInterface",
        allocations: [("DirectA", 6, 9, 15), ("TwoHop", 21, 24, 30)],
        other_dups: &[4, 19],
        stores: &[18, 33],
    },
];

struct Scratch(PathBuf);

impl Scratch {
    fn new(leg: &str) -> Self {
        let nonce = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-constructed-reference-array-{}-{leg}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("private fixture scratch directory is created");
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

fn archive(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in files {
            let (mut entry, config) = writer
                .new_file(EntryPath::verbatim(name.as_bytes().to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("class entry starts");
            let mut stream = config.wrap(&mut entry);
            stream.write_all(bytes).expect("class entry is written");
            let (_, descriptor) = stream.finish().expect("class entry closes");
            entry.finish(descriptor).expect("class entry finishes");
        }
        writer.finish().expect("complete class archive closes");
    }
    output.into_inner()
}

fn request(snapshot: &artifact::ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    }
}

fn class_source(snapshot: &artifact::ArtifactSnapshot, class: &str) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot, class),
            &RecoveryEvidenceRequest::all(),
            &mut task_budget(&[]).expect("bounded task budget"),
        )
        .expect("valid class-source request answers")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("{class} class source did not complete: {other:?}"),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("class has no method `{name}`"))
}

fn recovered_body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &method(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("{name} has no recovered body: {other:?}"),
    }
}

fn save_classes(files: &[(&str, &[u8])], destination: &Path) {
    fs::create_dir_all(destination).expect("class-only directory exists");
    for (name, bytes) in files {
        fs::write(destination.join(name), bytes).expect("frozen class is copied");
    }
}

fn java_tool() -> PathBuf {
    std::env::var_os("JARDE_JAVA23")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("java"))
}

fn javac_tool() -> PathBuf {
    std::env::var_os("JARDE_JAVAC23")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("javac"))
}

fn compile_complete(source_dir: &Path, classes: &Path, empty: &Path) -> Output {
    fs::create_dir_all(classes).expect("isolated candidate class directory exists");
    let mut command = Command::new(javac_tool());
    command
        .args([
            "-source",
            "8",
            "-target",
            "8",
            "-g:none",
            "-Xlint:-options",
            "-classpath",
        ])
        .arg(empty)
        .arg("-sourcepath")
        .arg(empty)
        .arg("-d")
        .arg(classes);
    for name in SOURCE_NAMES {
        command.arg(source_dir.join(name));
    }
    command
        .current_dir(source_dir)
        .output()
        .expect("host javac is available for isolated source compilation")
}

fn compile_single(source: &Path, classes: &Path, empty: &Path) -> Output {
    fs::create_dir_all(classes).expect("isolated control class directory exists");
    Command::new(javac_tool())
        .args([
            "-source",
            "8",
            "-target",
            "8",
            "-g:none",
            "-Xlint:-options",
            "-classpath",
        ])
        .arg(empty)
        .arg("-sourcepath")
        .arg(empty)
        .arg("-d")
        .arg(classes)
        .arg(source)
        .current_dir(source.parent().expect("source has a parent directory"))
        .output()
        .expect("host javac is available for isolated control compilation")
}

fn run_verified(classes: &Path) -> Output {
    Command::new(java_tool())
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(classes)
        .arg("Main")
        .output()
        .expect("host java is available for verifier execution")
}

fn run_verified_main(classes: &Path, main: &str) -> Output {
    Command::new(java_tool())
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(classes)
        .arg(main)
        .output()
        .expect("host java is available for control verifier execution")
}

fn assert_complete_reports(reports: &[ClassSourceReport]) {
    assert_eq!(reports.len(), CLASS_NAMES.len());
    for (class, report) in CLASS_NAMES.iter().zip(reports) {
        assert_eq!(
            report
                .declaration
                .as_ref()
                .map(|declaration| declaration.name.as_str()),
            Some(*class)
        );
        assert!(
            !report.text.contains("@bytecode") && !report.text.contains("jarde_refused_body"),
            "{class} source retained a refusal marker:\n{}",
            report.text
        );
        for member in &report.methods {
            assert!(
                !member.text.contains("@bytecode") && !member.text.contains("jarde_refused_body"),
                "{class} method {:?} retained a refusal marker:\n{}",
                member.item.name.raw(),
                member.text
            );
        }
    }
}

fn assert_constructor_sites(report: &ClassSourceReport) {
    for target in TARGETS {
        let body = recovered_body(report, target.name);
        assert_eq!(body.quality, Quality::Structured, "{}", target.name);
        assert_eq!(body.representation, Representation::Java, "{}", target.name);
        assert!(
            !body.text.contains("@bytecode") && !body.text.contains("jarde_refused_body"),
            "{} retained a fallback marker:\n{}",
            target.name,
            body.text
        );

        let expected_heads = target
            .allocations
            .iter()
            .map(|(_, head, _, _)| *head)
            .collect::<Vec<_>>();
        let presented = body
            .news
            .iter()
            .filter(|record| record.presented)
            .collect::<Vec<_>>();
        assert_eq!(
            body.news.len(),
            2,
            "{} allocation records: {:?}",
            target.name,
            body.news
        );
        assert_eq!(presented.len(), 2, "{} presented records", target.name);
        let mut actual_heads = presented
            .iter()
            .map(|record| record.head)
            .collect::<Vec<_>>();
        actual_heads.sort_unstable();
        assert_eq!(
            actual_heads, expected_heads,
            "{} allocation heads",
            target.name
        );

        let mut source_bcis = Vec::new();
        for (class, head, dup, constructor) in target.allocations {
            let matching = body
                .news
                .iter()
                .filter(|record| record.head == head)
                .collect::<Vec<_>>();
            assert_eq!(
                matching.len(),
                1,
                "{} new@{head} must have one record",
                target.name
            );
            let record = matching[0];
            assert!(
                record.presented,
                "{} new@{head} was not presented: {record:?}",
                target.name
            );
            assert_eq!(record.class, class, "{} new@{head} class", target.name);
            assert_eq!(record.dup, Some(dup), "{} new@{head} dup", target.name);
            assert_eq!(
                record.constructor,
                Some(constructor),
                "{} new@{head} init",
                target.name
            );
            source_bcis.extend([head, dup, constructor]);
        }
        source_bcis.extend_from_slice(target.other_dups);
        source_bcis.extend_from_slice(target.stores);
        source_bcis.sort_unstable();
        source_bcis.dedup();
        for bci in source_bcis {
            assert!(
                !body.source_map.of_bci(bci).is_empty(),
                "{} omitted javap new/dup/invokespecial/aastore BCI {bci}: {:?}",
                target.name,
                body.source_map.segments()
            );
        }
    }
}

#[test]
fn constructed_reference_array_elements_recover_as_one_complete_same_snapshot_family() {
    for (leg, files, oracle_stdout, oracle_stderr) in [
        ("javac8", JAVAC8_CLASSES, JAVAC8_STDOUT, JAVAC8_STDERR),
        ("javac23", JAVAC23_CLASSES, JAVAC23_STDOUT, JAVAC23_STDERR),
    ] {
        let jar = archive(files);
        let snapshot = Engine::new()
            .open(
                ArtifactInput::bytes(jar),
                &mut task_budget(&[]).expect("bounded open budget"),
            )
            .expect("one complete seven-class family opens");
        let reports = CLASS_NAMES
            .iter()
            .map(|class| class_source(&snapshot, class))
            .collect::<Vec<_>>();
        assert_complete_reports(&reports);
        let main_report = &reports[0];
        assert_constructor_sites(main_report);

        let scratch = Scratch::new(leg);
        let root = scratch.path();
        let source_dir = root.join("candidate-sources");
        let candidate_classes = root.join("candidate-classes");
        let original_classes = root.join("original-classes");
        let empty = root.join("empty-classpath-sourcepath");
        fs::create_dir_all(&source_dir).expect("candidate source directory exists");
        fs::create_dir_all(&empty).expect("empty classpath/sourcepath exists");
        for (class, report) in CLASS_NAMES.iter().zip(&reports) {
            fs::write(source_dir.join(format!("{class}.java")), &report.text)
                .expect("complete class source is written");
        }
        save_classes(files, &original_classes);

        let compile = compile_complete(&source_dir, &candidate_classes, &empty);
        assert!(
            compile.status.success(),
            "{leg} generated seven-class source family did not compile:\n{}\n{}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let original = run_verified(&original_classes);
        assert!(
            original.status.success(),
            "{leg} original verifier run failed:\n{}\n{}",
            String::from_utf8_lossy(&original.stdout),
            String::from_utf8_lossy(&original.stderr)
        );
        assert_eq!(
            original.stdout, oracle_stdout,
            "{leg} original stdout oracle"
        );
        assert_eq!(
            original.stderr, oracle_stderr,
            "{leg} original stderr oracle"
        );

        let recovered = run_verified(&candidate_classes);
        assert!(
            recovered.status.success(),
            "{leg} recovered verifier run failed:\n{}\n{}",
            String::from_utf8_lossy(&recovered.stdout),
            String::from_utf8_lossy(&recovered.stderr)
        );
        assert_eq!(
            recovered.status.code(),
            original.status.code(),
            "{leg} runtime exit"
        );
        assert_eq!(recovered.stdout, original.stdout, "{leg} recovered stdout");
        assert_eq!(recovered.stderr, original.stderr, "{leg} recovered stderr");
        assert_eq!(
            recovered.stdout, oracle_stdout,
            "{leg} recovered stdout oracle"
        );
        assert_eq!(
            recovered.stderr, oracle_stderr,
            "{leg} recovered stderr oracle"
        );
    }
}

#[test]
fn nested_constructor_composition_is_presented_once_and_boundary_controls_remain_honest() {
    for (leg, nested_files, boundary_files) in [
        ("javac8", CONTROL_JAVAC8_NESTED, CONTROL_JAVAC8_BOUNDARY),
        ("javac23", CONTROL_JAVAC23_NESTED, CONTROL_JAVAC23_BOUNDARY),
    ] {
        let nested_snapshot = Engine::new()
            .open(
                ArtifactInput::bytes(archive(nested_files)),
                &mut task_budget(&[]).expect("bounded control open budget"),
            )
            .expect("nested control class opens");
        let nested_report = class_source(&nested_snapshot, "NestedControls");
        assert_eq!(
            nested_report
                .declaration
                .as_ref()
                .map(|declaration| declaration.name.as_str()),
            Some("NestedControls")
        );
        assert!(
            !nested_report.text.contains("@bytecode")
                && !nested_report.text.contains("jarde_refused_body"),
            "{leg} nested control retained a class-level refusal"
        );
        let nested = recovered_body(&nested_report, "nested");
        assert_eq!(nested.quality, Quality::Structured, "{leg} nested quality");
        assert_eq!(
            nested.representation,
            Representation::Java,
            "{leg} nested representation"
        );
        assert_eq!(
            nested.text.matches("mark(\"nested\")").count(),
            1,
            "{leg} nested source must present the side effect once:\n{}",
            nested.text
        );
        assert_eq!(nested.news.len(), 2, "{leg} nested allocation records");
        for (head, dup, constructor, argument) in [(6, 9, 22, 19), (10, 13, 19, 16)] {
            let matching = nested
                .news
                .iter()
                .filter(|record| record.head == head)
                .collect::<Vec<_>>();
            assert_eq!(matching.len(), 1, "{leg} nested new@{head} record");
            let record = matching[0];
            assert_eq!(record.class, "java/lang/StringBuilder");
            assert_eq!(record.dup, Some(dup));
            assert_eq!(record.constructor, Some(constructor));
            assert_eq!(record.arguments.as_slice(), &[argument]);
            assert!(record.presented, "{leg} nested new@{head} was refused");
            assert!(record.refusal.is_none());
        }
        for bci in [6, 9, 10, 13, 19, 22, 25] {
            assert!(
                !nested.source_map.of_bci(bci).is_empty(),
                "{leg} nested source map omitted allocation/dup/init/store BCI {bci}"
            );
        }

        let scratch = Scratch::new(&format!("{leg}-nested-control"));
        let root = scratch.path();
        let sources = root.join("sources");
        let original_classes = root.join("original-classes");
        let candidate_classes = root.join("candidate-classes");
        let empty = root.join("empty-classpath-sourcepath");
        fs::create_dir_all(&sources).expect("nested source directory exists");
        fs::create_dir_all(&empty).expect("empty paths exist");
        fs::write(sources.join("NestedControls.java"), &nested_report.text)
            .expect("only the complete nested report is written");
        save_classes(nested_files, &original_classes);
        let compile = compile_single(
            &sources.join("NestedControls.java"),
            &candidate_classes,
            &empty,
        );
        assert!(
            compile.status.success(),
            "{leg} nested report did not compile in isolation:\n{}\n{}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let original = run_verified_main(&original_classes, "NestedControls");
        let candidate = run_verified_main(&candidate_classes, "NestedControls");
        assert!(
            original.status.success(),
            "{leg} nested original did not verify"
        );
        assert!(
            candidate.status.success(),
            "{leg} nested candidate did not verify"
        );
        assert_eq!(
            original.stdout, candidate.stdout,
            "{leg} nested host-JDK stdout"
        );
        assert_eq!(
            original.stderr, candidate.stderr,
            "{leg} nested host-JDK stderr"
        );
        assert_eq!(
            String::from_utf8_lossy(&candidate.stdout)
                .matches("mark:nested")
                .count(),
            1,
            "{leg} nested runtime must observe mark exactly once"
        );

        let boundary_snapshot = Engine::new()
            .open(
                ArtifactInput::bytes(archive(boundary_files)),
                &mut task_budget(&[]).expect("bounded boundary open budget"),
            )
            .expect("boundary control class opens");
        let boundary = class_source(&boundary_snapshot, "BoundaryControls");
        assert_eq!(
            boundary
                .declaration
                .as_ref()
                .map(|declaration| declaration.name.as_str()),
            Some("BoundaryControls")
        );
        for name in [
            "firstThenUnsupportedStructure",
            "old",
            "repeated",
            "descending",
            "crossBlock",
            "closedNumberBoundary",
        ] {
            let _ = method(&boundary, name);
        }

        let first = recovered_body(&boundary, "firstThenUnsupportedStructure");
        assert_eq!(first.quality, Quality::Fallback, "{leg} first quality");
        assert_eq!(first.representation, Representation::Mixed);
        assert!(first.text.contains("@bytecode"));
        assert_eq!(first.news.len(), 2, "{leg} first allocation records");
        assert!(first.news.iter().all(|record| !record.presented));
        assert_eq!(
            first.news[0].refusal.as_ref().map(|r| r.code),
            Some("jre_new_shape")
        );
        assert_eq!(
            first.news[1].refusal.as_ref().map(|r| r.code),
            Some("jre_new_interleaved_effect")
        );
        assert!(!first.text.contains("new java.lang.Object[]{"));
        assert!(!first.text.contains("new java.lang.StringBuilder("));
        assert!(!first.text.contains("new java.lang.Long("));

        for name in ["old", "repeated", "descending"] {
            let body = recovered_body(&boundary, name);
            assert!(
                !body.text.contains("new java.lang.Object[]{"),
                "{leg} {name} must not claim a composed initializer"
            );
            let mut heads = body
                .news
                .iter()
                .map(|record| record.head)
                .collect::<Vec<_>>();
            heads.sort_unstable();
            heads.dedup();
            assert_eq!(
                heads.len(),
                body.news.len(),
                "{leg} {name} duplicate records"
            );
        }

        let cross = recovered_body(&boundary, "crossBlock");
        assert!(
            !cross.text.contains("new java.lang.Object[]{"),
            "{leg} crossBlock must not claim a cross-block composed initializer"
        );
        assert_eq!(cross.news.len(), 1, "{leg} crossBlock construction count");
        assert_eq!(cross.news[0].head, 5);
        assert!(cross.news[0].presented);
        assert!(cross.text.contains("local1[0] = local2"));

        let closed = recovered_body(&boundary, "closedNumberBoundary");
        assert_eq!(
            closed.quality,
            Quality::Fallback,
            "{leg} closed Number quality"
        );
        assert_eq!(closed.representation, Representation::Mixed);
        assert!(
            closed
                .text
                .contains("no compatible reference fact for a Java initializer")
        );
        assert!(!closed.text.contains("new java.lang.Number[]{"));
        assert_eq!(closed.news.len(), 2, "{leg} Number constructor records");
        for (head, dup, constructor, class, argument) in [
            (6, 9, 15, "java/lang/Integer", 12),
            (21, 24, 30, "java/math/BigDecimal", 27),
        ] {
            let matching = closed
                .news
                .iter()
                .filter(|record| record.head == head)
                .collect::<Vec<_>>();
            assert_eq!(matching.len(), 1, "{leg} closed Number new@{head}");
            let record = matching[0];
            assert_eq!(record.class, class);
            assert_eq!(record.dup, Some(dup));
            assert_eq!(record.constructor, Some(constructor));
            assert_eq!(record.arguments.as_slice(), &[argument]);
            assert!(
                record.presented,
                "{leg} structural site new@{head} evidence"
            );
            assert!(record.refusal.is_none());
        }
        for bci in [1, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 34] {
            assert!(
                !closed.source_map.of_bci(bci).is_empty(),
                "{leg} closed Number source map omitted allocation/constructor/store BCI {bci}"
            );
        }
    }
}
